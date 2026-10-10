// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! SAM 2.1 Hiera-Tiny (Meta, Apache-2.0) on the frame clicked: its image
//! encoder and mask decoder turn a click into the object's mask, which Cutie
//! then follows. Its own video memory is not used.

use std::path::Path;

use ort::session::Session;
use ort::value::TensorRef;

use super::WHAT;
use crate::session::on_cpu;

/// The side of the square the model sees the frame at.
pub(super) const SIDE: usize = 1024;
/// The side of the square the mask's logits come out at.
pub(super) const MASK_SIDE: usize = 256;
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const DEVIATION: [f32; 3] = [0.229, 0.224, 0.225];

pub(super) struct Sam {
  vision: Session,
  decoder: Session,
}

fn failed(error: ort::Error) -> String {
  format!("{WHAT} could not run: {error}")
}

impl Sam {
  /// The image encoder at `vision` and the mask decoder at `decoder`, on
  /// the CPU: they read a single frame, in less time than loading the image
  /// encoder onto the GPU takes.
  pub(super) fn load(vision: &Path, decoder: &Path) -> Result<Self, String> {
    Ok(Self {
      vision: on_cpu(vision, WHAT)?,
      decoder: on_cpu(decoder, WHAT)?,
    })
  }

  /// The logits of the object at `point`, 0 to 1 across and down the 8-bit
  /// RGB frame `rgb`, [`SIDE`] square: [`MASK_SIDE`] square, positive on
  /// the object.
  /// The mask of the object at `points`, each 0 to 1 across and down the
  /// frame `rgb`: a click's one, or several along a painted line.
  pub(super) fn mask(&mut self, rgb: &[u8], points: &[[f32; 2]]) -> Result<Vec<f32>, String> {
    let plane = SIDE * SIDE;
    let mut planes = vec![0.0_f32; 3 * plane];
    for (index, pixel) in rgb.as_chunks::<3>().0.iter().enumerate() {
      for (channel, &byte) in pixel.iter().enumerate() {
        planes[channel * plane + index] =
          (f32::from(byte) / 255.0 - MEAN[channel]) / DEVIATION[channel];
      }
    }
    let pixels =
      TensorRef::from_array_view(([1_usize, 3, SIDE, SIDE], planes.as_slice())).map_err(failed)?;
    let seen = self
      .vision
      .run(ort::inputs!["pixel_values" => pixels])
      .map_err(failed)?;
    let feature = |name: &str| -> Result<Vec<f32>, String> {
      Ok(
        seen[name]
          .try_extract_tensor::<f32>()
          .map_err(failed)?
          .1
          .to_vec(),
      )
    };
    let (feats0, feats1, feats2) = (
      feature("feats0")?,
      feature("feats1")?,
      feature("feats2_no_mem")?,
    );
    drop(seen);
    let side = SIDE as f32;
    let count = points.len();
    let labels = vec![1_i32; count];
    let points = points
      .iter()
      .flat_map(|point| [point[0] * side, point[1] * side])
      .collect::<Vec<_>>();
    let outputs = self
      .decoder
      .run(ort::inputs![
        "feats0" => TensorRef::from_array_view(([1_usize, 32, 256, 256], feats0.as_slice())).map_err(failed)?,
        "feats1" => TensorRef::from_array_view(([1_usize, 64, 128, 128], feats1.as_slice())).map_err(failed)?,
        "feats2_cond" => TensorRef::from_array_view(([1_usize, 256, 64, 64], feats2.as_slice())).map_err(failed)?,
        "input_points" => TensorRef::from_array_view(([1_usize, 1, count, 2], &points[..])).map_err(failed)?,
        "input_labels" => TensorRef::from_array_view(([1_usize, 1, count], &labels[..])).map_err(failed)?,
      ])
      .map_err(failed)?;
    Ok(
      outputs["low_res_mask"]
        .try_extract_tensor::<f32>()
        .map_err(failed)?
        .1
        .to_vec(),
    )
  }
}
