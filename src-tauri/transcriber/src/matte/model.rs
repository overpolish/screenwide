// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Robust Video Matting run frame after frame, carrying its four recurrent
//! states from each frame into the next.

use ort::session::Session;
use ort::value::TensorRef;

use super::WHAT;

/// The longest side RVM's backbone sees, as its authors advise: the frame
/// is shrunk to this for finding the person, and the edge refined at the
/// matte's size.
const BACKBONE_SIDE: f32 = 512.0;

pub(super) struct Matting {
  session: Session,
  ratio: f32,
  states: [(Vec<usize>, Vec<f32>); 4],
  /// The frame as the model takes it, kept between frames.
  planes: Vec<f32>,
}

impl Matting {
  pub(super) fn new(session: Session, (width, height): (usize, usize)) -> Self {
    // The model starts from nothing seen, which it takes as a single zero.
    let empty = || (vec![1, 1, 1, 1], vec![0.0]);
    Self {
      session,
      ratio: (BACKBONE_SIDE / width.max(height) as f32).min(1.0),
      states: [empty(), empty(), empty(), empty()],
      planes: vec![0.0; width * height * 3],
    }
  }

  /// The alpha of the 8-bit RGB frame `rgb`, 0 to 255, into `alpha`.
  pub(super) fn run(
    &mut self,
    rgb: &[u8],
    (width, height): (usize, usize),
    alpha: &mut [u8],
  ) -> Result<(), String> {
    planar(rgb, width * height, &mut self.planes);
    let failed = |error: ort::Error| format!("{WHAT} could not run: {error}");
    fn state((shape, values): &(Vec<usize>, Vec<f32>)) -> ort::Result<TensorRef<'_, f32>> {
      TensorRef::from_array_view((shape.as_slice(), values.as_slice()))
    }
    let ratio = [self.ratio];
    let source = TensorRef::from_array_view(([1_usize, 3, height, width], self.planes.as_slice()));
    let inputs = ort::inputs![
      "src" => source.map_err(failed)?,
      "r1i" => state(&self.states[0]).map_err(failed)?,
      "r2i" => state(&self.states[1]).map_err(failed)?,
      "r3i" => state(&self.states[2]).map_err(failed)?,
      "r4i" => state(&self.states[3]).map_err(failed)?,
      "downsample_ratio" => TensorRef::from_array_view(([1_usize], &ratio[..])).map_err(failed)?,
    ];
    let outputs = self.session.run(inputs).map_err(failed)?;
    let (_, values) = outputs["pha"].try_extract_tensor::<f32>().map_err(failed)?;
    quantise(values, alpha);
    for (index, name) in ["r1o", "r2o", "r3o", "r4o"].into_iter().enumerate() {
      let (shape, values) = outputs[name].try_extract_tensor::<f32>().map_err(failed)?;
      let held = &mut self.states[index];
      held.0.clear();
      held
        .0
        .extend(shape.iter().map(|&side| side.max(0) as usize));
      held.1.clear();
      held.1.extend_from_slice(values);
    }
    Ok(())
  }
}

/// Interleaved 8-bit RGB as the model's planar 0-to-1 channels, laid out
/// `[1, 3, height, width]` for a frame of `plane` pixels.
pub(super) fn planar(rgb: &[u8], plane: usize, planes: &mut [f32]) {
  for (index, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
    for (channel, &value) in pixel.iter().enumerate() {
      planes[channel * plane + index] = f32::from(value) / 255.0;
    }
  }
}

pub(super) fn quantise(values: &[f32], into: &mut [u8]) {
  for (value, byte) in values.iter().zip(into.iter_mut()) {
    *byte = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
  }
}
