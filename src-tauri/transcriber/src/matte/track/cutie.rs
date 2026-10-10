// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cutie-small (Cheng et al., MIT), the video object segmentation model Final
//! Cut's Magnetic Mask resembles, as three ONNX graphs: one that reads a
//! frame, one that finds an object in it from what is remembered, and one
//! that remembers a frame with the object's mask. Each graph has fixed sizes,
//! so CoreML and DirectML can take it: frames of [`HEIGHT`] by [`WIDTH`], and
//! a memory of [`MEMORY_FRAMES`] frames, those not yet filled marked as such.
//!
//! As in Cutie's own inference, the clicked frame is remembered for good and
//! every [`MEMORY_EVERY`]th frame since for a while, the newest replacing the
//! oldest; the object's sensory memory changes every frame.

use std::collections::VecDeque;
use std::path::Path;

use ort::session::{Session, SessionOutputs};
use ort::value::TensorRef;

use super::WHAT;
use crate::session::on_platform_gpu;

/// The size the model sees each frame at: 480 lines, as Cutie works at, by
/// a 16:9 width, in sixteens.
pub(super) const HEIGHT: usize = 480;
pub(super) const WIDTH: usize = 864;
/// The size an object's mask comes out at: a quarter of the frame's.
pub(in crate::matte) const MASK_HEIGHT: usize = HEIGHT / 4;
pub(in crate::matte) const MASK_WIDTH: usize = WIDTH / 4;
const TOKENS: usize = (HEIGHT / 16) * (WIDTH / 16);
const KEY: usize = 64;
const VALUE: usize = 256;
const SENSORY: usize = 256;
const SUMMARIES: usize = 16 * 257;
const MEMORY_FRAMES: usize = 5;
const MEMORY_EVERY: usize = 5;

fn failed(error: ort::Error) -> String {
  format!("{WHAT} could not run: {error}")
}

fn extracted(outputs: &SessionOutputs<'_>, name: &str) -> Result<Vec<f32>, String> {
  Ok(
    outputs[name]
      .try_extract_tensor::<f32>()
      .map_err(failed)?
      .1
      .to_vec(),
  )
}

pub(super) struct Cutie {
  encode: Session,
  read: Session,
  memorise: Session,
}

/// A frame as the model reads it.
pub(super) struct Seen {
  planes: Vec<f32>,
  f8: Vec<f32>,
  f4: Vec<f32>,
  pix: Vec<f32>,
  key: Vec<f32>,
  shrinkage: Vec<f32>,
  selection: Vec<f32>,
}

/// What the model remembers of one object, cloned to follow the frames
/// between two followed ones from the earlier.
#[derive(Clone)]
pub(super) struct Object {
  /// The remembered frames' keys, shrinkage and the object's values; the
  /// first is the clicked frame's, kept for good.
  memory: VecDeque<(Vec<f32>, Vec<f32>, Vec<f32>)>,
  sensory: Vec<f32>,
  summaries: Vec<f32>,
  /// The object's probability in the frame last found, full size.
  last: Vec<f32>,
  since_memory: usize,
}

impl Object {
  /// The object's probability in the frame last found, full size.
  pub(super) fn last(&self) -> &[f32] {
    &self.last
  }
}

impl Cutie {
  /// The three graphs, `encode`, `read` and `memorise`, on the GPU where it
  /// can be, CoreML keeping its compiled models in `cache`.
  pub(super) fn load(files: [&Path; 3], cache: &Path) -> Result<Self, String> {
    Ok(Self {
      encode: on_platform_gpu(files[0], WHAT, cache)?,
      read: on_platform_gpu(files[1], WHAT, cache)?,
      memorise: on_platform_gpu(files[2], WHAT, cache)?,
    })
  }

  /// The 8-bit RGB frame `rgb`, [`WIDTH`] by [`HEIGHT`], read.
  pub(super) fn see(&mut self, rgb: &[u8]) -> Result<Seen, String> {
    let plane = WIDTH * HEIGHT;
    let mut planes = vec![0.0_f32; 3 * plane];
    for (index, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
      for (channel, &byte) in pixel.iter().enumerate() {
        planes[channel * plane + index] = f32::from(byte) / 255.0;
      }
    }
    let image = TensorRef::from_array_view(([1_usize, 3, HEIGHT, WIDTH], planes.as_slice()))
      .map_err(failed)?;
    let outputs = self
      .encode
      .run(ort::inputs!["image" => image])
      .map_err(failed)?;
    let seen = Seen {
      f8: extracted(&outputs, "f8")?,
      f4: extracted(&outputs, "f4")?,
      pix: extracted(&outputs, "pix")?,
      key: extracted(&outputs, "key")?,
      shrinkage: extracted(&outputs, "shrinkage")?,
      selection: extracted(&outputs, "selection")?,
      planes: Vec::new(),
    };
    drop(outputs);
    Ok(Seen { planes, ..seen })
  }

  /// An object starting from `mask`, its probability in the clicked frame
  /// `seen`, full size.
  pub(super) fn start(&mut self, seen: &Seen, mask: Vec<f32>) -> Result<Object, String> {
    let mut object = Object {
      memory: VecDeque::with_capacity(MEMORY_FRAMES),
      sensory: vec![0.0; SENSORY * TOKENS],
      summaries: vec![0.0; SUMMARIES],
      last: mask,
      since_memory: 0,
    };
    self.remember(seen, &mut object)?;
    Ok(object)
  }

  /// `object` found in `seen`, answering its probability at
  /// [`MASK_WIDTH`] by [`MASK_HEIGHT`]. Remembers the frame when it is due,
  /// unless `remember` is false, for a frame followed only to fill a gap.
  pub(super) fn find(
    &mut self,
    seen: &Seen,
    object: &mut Object,
    remember: bool,
  ) -> Result<Vec<f32>, String> {
    let length = MEMORY_FRAMES * TOKENS;
    let mut keys = vec![0.0_f32; KEY * length];
    let mut shrinkage = vec![1.0_f32; length];
    let mut valid = vec![0.0_f32; length];
    let mut values = vec![0.0_f32; VALUE * length];
    for (slot, (key, shrink, value)) in object.memory.iter().enumerate() {
      let at = slot * TOKENS;
      for channel in 0..KEY {
        keys[channel * length + at..][..TOKENS].copy_from_slice(&key[channel * TOKENS..][..TOKENS]);
      }
      for channel in 0..VALUE {
        values[channel * length + at..][..TOKENS]
          .copy_from_slice(&value[channel * TOKENS..][..TOKENS]);
      }
      shrinkage[at..at + TOKENS].copy_from_slice(shrink);
      valid[at..at + TOKENS].fill(1.0);
    }
    let selector = [1.0_f32];
    let (h, w) = (HEIGHT / 16, WIDTH / 16);
    let outputs = self
      .read
      .run(ort::inputs![
        "key" => TensorRef::from_array_view(([1, KEY, h, w], seen.key.as_slice())).map_err(failed)?,
        "selection" => TensorRef::from_array_view(([1, KEY, h, w], seen.selection.as_slice())).map_err(failed)?,
        "memory_key" => TensorRef::from_array_view(([1, KEY, length], keys.as_slice())).map_err(failed)?,
        "memory_shrinkage" => TensorRef::from_array_view(([1, 1, length], shrinkage.as_slice())).map_err(failed)?,
        "memory_valid" => TensorRef::from_array_view(([1, 1, length], valid.as_slice())).map_err(failed)?,
        "memory_value" => TensorRef::from_array_view(([1, 1, VALUE, length], values.as_slice())).map_err(failed)?,
        "pix" => TensorRef::from_array_view(([1, 256, h, w], seen.pix.as_slice())).map_err(failed)?,
        "sensory" => TensorRef::from_array_view(([1, 1, SENSORY, h, w], object.sensory.as_slice())).map_err(failed)?,
        "last_mask" => TensorRef::from_array_view(([1, 1, HEIGHT, WIDTH], object.last.as_slice())).map_err(failed)?,
        "object_memory" => TensorRef::from_array_view(([1, 1, 16, 257], object.summaries.as_slice())).map_err(failed)?,
        "selector" => TensorRef::from_array_view(([1, 1, 1, 1], &selector[..])).map_err(failed)?,
        "f8" => TensorRef::from_array_view(([1, 128, h * 2, w * 2], seen.f8.as_slice())).map_err(failed)?,
        "f4" => TensorRef::from_array_view(([1, 64, h * 4, w * 4], seen.f4.as_slice())).map_err(failed)?,
      ])
      .map_err(failed)?;
    object.sensory = extracted(&outputs, "sensory_out")?;
    let quarter = extracted(&outputs, "prob")?;
    drop(outputs);
    let (mask, last) = weighed(&quarter);
    object.last = last;
    object.since_memory += 1;
    if remember && object.since_memory >= MEMORY_EVERY {
      self.remember(seen, object)?;
    }
    Ok(mask)
  }

  /// Remembers `seen` with the object as last found in it.
  fn remember(&mut self, seen: &Seen, object: &mut Object) -> Result<(), String> {
    let (h, w) = (HEIGHT / 16, WIDTH / 16);
    let outputs = self
      .memorise
      .run(ort::inputs![
        "image" => TensorRef::from_array_view(([1_usize, 3, HEIGHT, WIDTH], seen.planes.as_slice())).map_err(failed)?,
        "pix" => TensorRef::from_array_view(([1, 256, h, w], seen.pix.as_slice())).map_err(failed)?,
        "sensory" => TensorRef::from_array_view(([1, 1, SENSORY, h, w], object.sensory.as_slice())).map_err(failed)?,
        "mask" => TensorRef::from_array_view(([1, 1, HEIGHT, WIDTH], object.last.as_slice())).map_err(failed)?,
      ])
      .map_err(failed)?;
    let value = extracted(&outputs, "value")?;
    object.sensory = extracted(&outputs, "sensory_out")?;
    // The object's summaries add up over every remembered frame, the last
    // of each being how much of the object it saw.
    for (total, part) in object
      .summaries
      .iter_mut()
      .zip(extracted(&outputs, "summaries")?)
    {
      *total += part;
    }
    drop(outputs);
    object
      .memory
      .push_back((seen.key.clone(), seen.shrinkage.clone(), value));
    if object.memory.len() > MEMORY_FRAMES {
      // The clicked frame stays; the oldest since goes.
      object.memory.remove(1);
    }
    object.since_memory = 0;
    Ok(())
  }
}

/// One object's probability at a quarter of the size, weighed against the
/// room as Cutie weighs objects: its odds against the room's become logits,
/// stretched to full size, through a softmax of the two. Answers the
/// probability at a quarter of the size and at full size.
fn weighed(quarter: &[f32]) -> (Vec<f32>, Vec<f32>) {
  let logits = quarter
    .iter()
    .map(|&p| {
      let p = p.clamp(1e-7, 1.0 - 1e-7);
      (p / (1.0 - p)).ln()
    })
    .collect::<Vec<_>>();
  // The room's logit is the object's negated, so the softmax of the two is
  // the sigmoid of twice the object's.
  let sure = |logit: f32| 1.0 / (1.0 + (-2.0 * logit).exp());
  let mask = logits.iter().map(|&logit| sure(logit)).collect();
  let mut full = vec![0.0_f32; WIDTH * HEIGHT];
  for y in 0..HEIGHT {
    let sy = ((y as f32 + 0.5) / 4.0 - 0.5).clamp(0.0, (MASK_HEIGHT - 1) as f32);
    let (y0, fy) = (sy.floor() as usize, sy.fract());
    let y1 = (y0 + 1).min(MASK_HEIGHT - 1);
    for x in 0..WIDTH {
      let sx = ((x as f32 + 0.5) / 4.0 - 0.5).clamp(0.0, (MASK_WIDTH - 1) as f32);
      let (x0, fx) = (sx.floor() as usize, sx.fract());
      let x1 = (x0 + 1).min(MASK_WIDTH - 1);
      let at = |row: usize, column: usize| logits[row * MASK_WIDTH + column];
      let upper = at(y0, x0) * (1.0 - fx) + at(y0, x1) * fx;
      let lower = at(y1, x0) * (1.0 - fx) + at(y1, x1) * fx;
      full[y * WIDTH + x] = sure(upper * (1.0 - fy) + lower * fy);
    }
  }
  (mask, full)
}

/// The clicked object's probability at full size from SAM's logits,
/// `side` square, stretched over the frame: one on the object, zero off it,
/// as Cutie takes a given mask.
pub(super) fn from_logits(logits: &[f32], side: usize) -> Vec<f32> {
  let mut mask = vec![0.0_f32; WIDTH * HEIGHT];
  let last = (side - 1) as f32;
  for y in 0..HEIGHT {
    let sy = ((y as f32 + 0.5) * side as f32 / HEIGHT as f32 - 0.5).clamp(0.0, last);
    let (y0, fy) = (sy.floor() as usize, sy.fract());
    let y1 = (y0 + 1).min(side - 1);
    for x in 0..WIDTH {
      let sx = ((x as f32 + 0.5) * side as f32 / WIDTH as f32 - 0.5).clamp(0.0, last);
      let (x0, fx) = (sx.floor() as usize, sx.fract());
      let x1 = (x0 + 1).min(side - 1);
      let upper = logits[y0 * side + x0] * (1.0 - fx) + logits[y0 * side + x1] * fx;
      let lower = logits[y1 * side + x0] * (1.0 - fx) + logits[y1 * side + x1] * fx;
      mask[y * WIDTH + x] = f32::from(upper * (1.0 - fy) + lower * fy > 0.0);
    }
  }
  mask
}
