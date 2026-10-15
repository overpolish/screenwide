// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as an export draws them on each frame, with the
//! matte read alongside the frames as they are drawn.

use crate::camera_effects::gpu::EffectsPass;
use crate::camera_effects::matte_frames::MatteFrames;
use crate::cursor_export::ExportCameraEffects;
use screenwide_annotations::pin::LumaFrame;
use screenwide_compositor::compositor::{Compositor, SourceTexture};
use screenwide_gpu::Gpu;

/// The slot the effects draw the camera into, one frame's camera at a time:
/// the GPU runs submissions in order, so a frame's reads of it come before
/// the next frame writes it.
const SLOT: usize = 0;

pub(crate) struct FrameEffects {
  effects: ExportCameraEffects,
  pass: EffectsPass,
  matte: Option<MatteFrames>,
}

impl FrameEffects {
  pub(crate) fn new(
    effects: Option<&ExportCameraEffects>,
    gpu: &'static Gpu,
  ) -> Result<Option<Self>, String> {
    let Some(effects) = effects else {
      return Ok(None);
    };
    // The export reads the matte at the size it was made.
    let matte =
      (!effects.live.matte.is_empty()).then(|| MatteFrames::open(&effects.live.matte, u32::MAX));
    Ok(Some(Self {
      effects: effects.clone(),
      pass: EffectsPass::new(gpu),
      matte,
    }))
  }

  /// Whether the effects are drawn on the screen, which is the camera.
  pub(crate) fn on_screen(&self) -> bool {
    self.effects.on_screen
  }

  /// `camera`, the camera frame at `ms`, drawn with the effects by the
  /// export's `compositor`.
  pub(crate) fn drawn(
    &mut self,
    compositor: &Compositor,
    camera: &SourceTexture,
    ms: u64,
  ) -> Result<SourceTexture, String> {
    let matte: Option<std::sync::Arc<LumaFrame>> = match self.matte.as_mut() {
      Some(matte) => matte.at(ms)?,
      None => None,
    };
    self.pass.apply(
      compositor,
      (camera, ms),
      matte.as_deref(),
      &self.effects.live,
      SLOT,
    )
  }
}
