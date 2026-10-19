// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the face is in a camera recording, found by BlazeFace short range
//! (Google MediaPipe, Apache-2.0): a small model made for faces near a
//! webcam, which takes about a millisecond a frame on one CPU thread. It
//! looks five times a second, which follows a person as closely as the
//! smoothed crop needs; the app smooths what it finds into the path a scene
//! follows. FFmpeg decodes the camera, already shrunk to the model's size.

use std::path::Path;

use ort::session::Session;
use ort::value::TensorRef;
use screenwide_transcriber::FaceTrack;

use crate::matte::decode::Decoder;
use crate::session::on_cpu_threads;

/// The feature's name in what the helper reports.
const WHAT: &str = "Face tracking";
/// The model's square input, in pixels.
const SIDE: usize = 128;
/// How often a frame is looked at, in camera milliseconds.
const EVERY_MS: f64 = 200.0;
/// How sure the model must be that a face is there.
const LEAST_SCORE: f32 = 0.5;

/// The faces in the camera at `camera`, `size` pixels, written to `output`
/// as a [`FaceTrack`], with the model at `model` and `ffmpeg` decoding.
/// Tells `progress` how far it has got out of about `duration_ms`.
pub fn find_faces(
  (model, ffmpeg): (&Path, &Path),
  camera: &Path,
  size: (u32, u32),
  duration_ms: u64,
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  if size.0 == 0 || size.1 == 0 {
    return Err("The camera recording has no frames to read".to_owned());
  }
  let aspect = f64::from(size.0) / f64::from(size.1);
  // The picture fills the model's square across its longer side, centred,
  // with black bars along the other, as the model was trained.
  let (width, height) = if aspect >= 1.0 {
    (
      SIDE,
      ((SIDE as f64 / aspect / 2.0).round() as usize * 2).max(2),
    )
  } else {
    (
      ((SIDE as f64 * aspect / 2.0).round() as usize * 2).max(2),
      SIDE,
    )
  };
  let offset = ((SIDE - width) / 2, (SIDE - height) / 2);
  let mut detector = Detector::new(on_cpu_threads(model, WHAT, 1)?, offset);
  let mut decoder = Decoder::start(ffmpeg, camera, 0, (width as u32, height as u32))?;
  let mut rgb = vec![0_u8; width * height * 3];
  let mut frames = Vec::new();
  let mut next_ms = 0.0;
  let mut last_ms = 0.0;
  while let Some(ms) = decoder.next(&mut rgb)? {
    last_ms = ms;
    if ms + 1.0 < next_ms {
      continue;
    }
    next_ms = ms + EVERY_MS;
    let face = detector
      .detect(&rgb, (width, height))?
      .map(|[x1, y1, x2, y2]| {
        let to_x = |x: f32| (x - offset.0 as f32) / width as f32;
        let to_y = |y: f32| (y - offset.1 as f32) / height as f32;
        [
          (to_x(x1) + to_x(x2)) / 2.0,
          (to_y(y1) + to_y(y2)) / 2.0,
          to_y(y2) - to_y(y1),
        ]
      });
    frames.push((ms, face));
    if duration_ms > 0 {
      progress((ms / duration_ms as f64).clamp(0.0, 1.0) as f32);
    }
  }
  let track = FaceTrack {
    format: FaceTrack::FORMAT,
    aspect,
    duration_ms: last_ms.max(duration_ms as f64),
    frames,
  };
  let bytes = serde_json::to_vec(&track).map_err(|error| error.to_string())?;
  let partial = output.with_extension("partial.json");
  let failed = |error: std::io::Error| format!("Could not keep where the face is: {error}");
  std::fs::write(&partial, bytes).map_err(failed)?;
  std::fs::rename(&partial, output).map_err(failed)
}

struct Detector {
  session: Session,
  /// Where the picture sits in the model's square.
  offset: (usize, usize),
  planes: Vec<f32>,
  anchors: Vec<[f32; 2]>,
}

impl Detector {
  fn new(session: Session, offset: (usize, usize)) -> Self {
    Self {
      session,
      offset,
      planes: vec![0.0; SIDE * SIDE * 3],
      anchors: anchors(),
    }
  }

  /// The surest face in `rgb`, a picture `size` pixels, as its box in the
  /// model's square: left, top, right, bottom.
  fn detect(
    &mut self,
    rgb: &[u8],
    (width, height): (usize, usize),
  ) -> Result<Option<[f32; 4]>, String> {
    let area = SIDE * SIDE;
    // Black bars around the picture, -1 in the model's range.
    self.planes.fill(-1.0);
    for y in 0..height {
      for x in 0..width {
        let from = (y * width + x) * 3;
        let to = (y + self.offset.1) * SIDE + x + self.offset.0;
        for channel in 0..3 {
          self.planes[channel * area + to] = f32::from(rgb[from + channel]) / 127.5 - 1.0;
        }
      }
    }
    let failed = |error: ort::Error| format!("{WHAT} could not run: {error}");
    let input = TensorRef::from_array_view(([1_usize, 3, SIDE, SIDE], self.planes.as_slice()))
      .map_err(failed)?;
    let outputs = self
      .session
      .run(ort::inputs!["input" => input])
      .map_err(failed)?;
    let (_, boxes) = outputs["regressors"]
      .try_extract_tensor::<f32>()
      .map_err(failed)?;
    let (_, scores) = outputs["classificators"]
      .try_extract_tensor::<f32>()
      .map_err(failed)?;
    let side = SIDE as f32;
    // The surest face: with one person on camera, the one to follow.
    let surest = scores
      .iter()
      .enumerate()
      .map(|(index, &logit)| (index, 1.0 / (1.0 + (-logit.clamp(-100.0, 100.0)).exp())))
      .filter(|&(_, score)| score >= LEAST_SCORE)
      .max_by(|a, b| a.1.total_cmp(&b.1));
    Ok(surest.map(|(index, _)| {
      let [dx, dy, w, h] = [0, 1, 2, 3].map(|at| boxes[index * 16 + at] / side);
      let [ax, ay] = self.anchors[index];
      let (x, y) = (ax + dx, ay + dy);
      [x - w / 2.0, y - h / 2.0, x + w / 2.0, y + h / 2.0].map(|v| v * side)
    }))
  }
}

/// The short range model's anchors: a 16 by 16 grid with two to a cell,
/// then an 8 by 8 grid with six, each at its cell's middle as a share of
/// the square.
fn anchors() -> Vec<[f32; 2]> {
  let mut anchors = Vec::with_capacity(896);
  for (grid, each) in [(16_usize, 2_usize), (8, 6)] {
    for y in 0..grid {
      for x in 0..grid {
        let middle = [
          (x as f32 + 0.5) / grid as f32,
          (y as f32 + 0.5) / grid as f32,
        ];
        anchors.extend(std::iter::repeat_n(middle, each));
      }
    }
  }
  anchors
}
