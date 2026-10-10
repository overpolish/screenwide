// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A whole picture blurred by the Gaussian a blur redaction draws, for the
//! camera's blurred background, so the two blurs look alike.

use super::{Compositor, SourceTexture};
use crate::editor::annotations::redact::records::{RedactRecord, RedactRecords, REDACT_BLUR};

/// Where the redaction copies these blurs leave are kept, clear of the
/// slots the canvas's own redactions take.
const FIRST_SLOT: usize = 1 << 16;

impl Compositor {
  /// `source` blurred whole with a deviation of `deviation` source pixels,
  /// into a copy that stays readable until `slot` is blurred again.
  pub(crate) fn blurred_whole(
    &self,
    source: &SourceTexture,
    deviation: f32,
    slot: usize,
  ) -> Result<wgpu::TextureView, String> {
    let (width, height) = source.size;
    let records = RedactRecords {
      records: vec![RedactRecord {
        bounds: [0, 0, width, height],
        source_width: width,
        mode: REDACT_BLUR,
        size: deviation,
        color: [0.0, 0.0, 0.0, 1.0],
        ..RedactRecord::default()
      }],
      zones: Vec::new(),
    };
    let gpu = self.gpu();
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide whole blur"),
      });
    let view = self
      .redactor
      .apply(
        gpu,
        &mut encoder,
        source,
        &records,
        FIRST_SLOT + slot,
        false,
      )?
      .ok_or_else(|| "The blur covered nothing".to_owned())?;
    gpu.queue.submit([encoder.finish()]);
    Ok(view)
  }
}
