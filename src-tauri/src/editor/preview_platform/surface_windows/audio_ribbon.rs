// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's audio ribbon, drawn on its own composition visual.

use super::super::audio_ribbon::{AudioRibbonRenderer, RibbonLook};
use super::super::PreviewSurfaceRect;
use crate::editor::recording_preview_player::audio_visualizer::AudioRibbonEnvelopes;
use crate::gpu::surface::Surface;
use windows::Win32::Graphics::DirectComposition::{IDCompositionDevice, IDCompositionVisual};

pub(super) struct AudioRibbon {
  renderer: AudioRibbonRenderer,
  visual: IDCompositionVisual,
  composition: IDCompositionDevice,
  offset: (f32, f32),
  scale: f32,
  neutral: f32,
  playhead: f32,
}

impl AudioRibbon {
  /// The ribbon's visual under `root`, hidden until it has levels. The
  /// caller commits the tree.
  pub(super) fn new(
    shared: &'static crate::gpu::Gpu,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let visual = unsafe { composition.CreateVisual() }.map_err(|e| e.to_string())?;
    unsafe {
      visual
        .SetOffsetX2(-100000.0)
        .and_then(|_| root.AddVisual(&visual, true, None::<&IDCompositionVisual>))
    }
    .map_err(|e| e.to_string())?;
    let surface = Surface::on_visual(shared, &visual)?;
    Ok(Self {
      renderer: AudioRibbonRenderer::new(shared, surface),
      visual,
      composition: composition.clone(),
      offset: (0.0, 0.0),
      scale: 1.0,
      neutral: 1.0,
      playhead: 0.0,
    })
  }

  pub(super) fn set_viewport(
    &mut self,
    rect: PreviewSurfaceRect,
    scale: f64,
    backdrop: [f64; 4],
  ) -> Result<(), String> {
    self.renderer.set_viewport((
      (rect.width * scale).round() as u32,
      (rect.height * scale).round() as u32,
    ));
    self.offset = ((rect.x * scale) as f32, (rect.y * scale) as f32);
    self.scale = scale.max(0.1) as f32;
    let luminance = backdrop[0] * 0.2126 + backdrop[1] * 0.7152 + backdrop[2] * 0.0722;
    self.neutral = if luminance > 0.5 { 0.0 } else { 1.0 };
    self.draw()?;
    self.sync_visibility()
  }

  pub(super) fn set_envelopes(&mut self, values: &AudioRibbonEnvelopes) -> Result<(), String> {
    self.renderer.set_envelopes(values);
    self.draw()?;
    self.sync_visibility()
  }

  pub(super) fn has_envelopes(&self) -> bool {
    self.renderer.has_levels()
  }

  pub(super) fn set_playhead(&mut self, ratio: f64) -> Result<(), String> {
    if !self.renderer.has_levels() {
      return Ok(());
    }
    self.playhead = ratio.clamp(0.0, 1.0) as f32;
    self.draw()?;
    self.sync_visibility()
  }

  fn draw(&mut self) -> Result<(), String> {
    let [red, green, blue] = crate::system_accent::accent_rgb();
    self
      .renderer
      .draw(RibbonLook {
        scale: self.scale,
        playhead: self.playhead,
        color: [red, green, blue, 1.0],
        flat: [self.neutral, self.neutral, self.neutral, 0.25],
      })
      .map(|_| ())
  }

  fn sync_visibility(&self) -> Result<(), String> {
    let x = if self.renderer.has_levels() {
      self.offset.0
    } else {
      -100000.0
    };
    unsafe {
      self
        .visual
        .SetOffsetX2(x)
        .and_then(|_| self.visual.SetOffsetY2(self.offset.1))
        .and_then(|_| self.composition.Commit())
    }
    .map_err(|error| error.to_string())
  }
}
