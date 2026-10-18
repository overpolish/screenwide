// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's audio ribbon, drawn on a Metal layer of its own. The twin
//! of the Windows ribbon visual.

use objc2::rc::Retained;
use objc2_quartz_core::CAMetalLayer;

use super::super::audio_ribbon::{AudioRibbonRenderer, RibbonLook};
use super::layers::place;
use super::workspace_view::surface_on;
use crate::editor::recording_preview_player::audio_visualizer::AudioRibbonEnvelopes;
use screenwide_compositor::PreviewSurfaceRect;

pub(super) struct AudioRibbon {
  renderer: AudioRibbonRenderer,
  layer: Retained<CAMetalLayer>,
  /// Where the ribbon sits, in points from the root's top left.
  frame: (f64, f64, f64, f64),
  scale: f32,
  neutral: f32,
  playhead: f32,
}

// Moved only inside transactions and drawn through wgpu.
unsafe impl Send for AudioRibbon {}

impl AudioRibbon {
  /// The ribbon on `layer`, off screen until it has levels.
  ///
  /// # Safety
  /// Main thread only.
  pub(super) unsafe fn new(
    shared: &'static screenwide_gpu::Gpu,
    layer: Retained<CAMetalLayer>,
  ) -> Result<Self, String> {
    let surface = unsafe { surface_on(shared, &layer) }?;
    Ok(Self {
      renderer: AudioRibbonRenderer::new(shared, surface),
      layer,
      frame: (0.0, 0.0, 0.0, 0.0),
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
    self.frame = (rect.x, rect.y, rect.width, rect.height);
    self.scale = scale.max(0.1) as f32;
    let luminance = backdrop[0] * 0.2126 + backdrop[1] * 0.7152 + backdrop[2] * 0.0722;
    self.neutral = if luminance > 0.5 { 0.0 } else { 1.0 };
    self.draw()?;
    self.sync_visibility();
    Ok(())
  }

  pub(super) fn set_envelopes(&mut self, values: &AudioRibbonEnvelopes) -> Result<(), String> {
    self.renderer.set_envelopes(values);
    self.draw()?;
    self.sync_visibility();
    Ok(())
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
    self.sync_visibility();
    Ok(())
  }

  fn draw(&mut self) -> Result<(), String> {
    let [red, green, blue] = screenwide_accent::accent_rgb();
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

  fn sync_visibility(&self) {
    place(
      &self.layer,
      self.renderer.has_levels().then_some(self.frame),
    );
  }
}

impl super::RecordingPreviewSurface {
  pub(crate) fn set_audio_ribbon(&self, envelopes: &AudioRibbonEnvelopes) {
    let Ok(state) = self.inner.state.lock() else {
      return;
    };
    let host = super::mac(&self.inner);
    if let Ok(mut ribbon) = host.gpu.audio_ribbon.lock() {
      let _ = ribbon.set_viewport(state.viewport, state.scale, state.backdrop);
      if let Err(error) = ribbon.set_envelopes(envelopes) {
        eprintln!("Could not upload the audio ribbon: {error}");
      }
    }
  }

  pub(crate) fn set_audio_ribbon_playhead(&self, ratio: f64) {
    let Ok(_state) = self.inner.state.lock() else {
      return;
    };
    let host = super::mac(&self.inner);
    if let Ok(mut ribbon) = host.gpu.audio_ribbon.lock() {
      if ribbon.has_envelopes() {
        let _ = ribbon.set_playhead(ratio);
      }
    }
  }
}
