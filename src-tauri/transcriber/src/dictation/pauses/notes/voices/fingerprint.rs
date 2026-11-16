// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A voice's fingerprint: 3D-Speaker's CAM++ speaker-verification model
//! (English, VoxCeleb), as sherpa-onnx exports it, run on the CPU in tract.
//! Two stretches of the same voice give fingerprints that point the same way;
//! different voices, apart.

use std::path::Path;

use tract_onnx::prelude::*;

use super::fbank::{Fbank, DIMENSIONS};

type Runnable = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct Fingerprints {
  model: Runnable,
  fbank: Fbank,
}

impl Fingerprints {
  pub fn load(path: &Path) -> Result<Self, String> {
    let failed = |error: TractError| format!("Could not load the voice model: {error}");
    let mut model = tract_onnx::onnx().model_for_path(path).map_err(failed)?;
    let frames = model.symbol_table.sym("frames");
    model = model
      .with_input_fact(
        0,
        f32::fact([1.to_dim(), frames.to_dim(), DIMENSIONS.to_dim()]).into(),
      )
      .map_err(failed)?;
    let model = model
      .into_optimized()
      .and_then(|model| model.into_runnable())
      .map_err(failed)?;
    Ok(Self {
      model,
      fbank: Fbank::new(),
    })
  }

  /// The fingerprint of `samples`, 16 kHz in [-1, 1], scaled to unit length;
  /// none for a stretch too short to have a frame.
  pub fn of(&self, samples: &[f32]) -> Result<Option<Vec<f32>>, String> {
    let features = self.fbank.features(samples);
    let frames = features.len() / DIMENSIONS;
    if frames == 0 {
      return Ok(None);
    }
    let failed = |error: TractError| format!("The voice model could not run: {error}");
    let input = Tensor::from_shape(&[1, frames, DIMENSIONS], &features).map_err(failed)?;
    let outputs = self.model.run(tvec!(input.into())).map_err(failed)?;
    let embedding = outputs[0].to_array_view::<f32>().map_err(failed)?;
    let mut fingerprint: Vec<f32> = embedding.iter().copied().collect();
    let length = fingerprint
      .iter()
      .map(|value| value * value)
      .sum::<f32>()
      .sqrt();
    if length > 0.0 {
      fingerprint.iter_mut().for_each(|value| *value /= length);
    }
    Ok(Some(fingerprint))
  }
}
