// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's audio ribbon, drawn on its own composition visual.

use super::super::PreviewSurfaceRect;
use crate::editor::recording_preview_player::audio_visualizer::AudioRibbonEnvelopes;
use crate::windows::overlay_surface::{Frame, VisualSurface};
use windows::Win32::Graphics::DirectComposition::{IDCompositionDevice, IDCompositionVisual};

/// The twin of `Ribbon` in `audio_ribbon.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Constants {
  color: [f32; 4],
  flat: [f32; 4],
  geometry: [f32; 4],
  style: [f32; 4],
}

pub(super) struct AudioRibbon {
  shared: &'static crate::gpu::Gpu,
  surface: VisualSurface,
  pipeline: wgpu::RenderPipeline,
  layout: wgpu::BindGroupLayout,
  constants: wgpu::Buffer,
  /// The constants and the bucketed levels, once there are levels to draw.
  levels: Option<wgpu::BindGroup>,
  visual: IDCompositionVisual,
  viewport: (u32, u32),
  points: u32,
  playhead: f32,
  composition: IDCompositionDevice,
  offset: (f32, f32),
  scale: f32,
  neutral: f32,
  dirty: bool,
  last_constants: Option<Constants>,
}

#[path = "audio_ribbon/draw.rs"]
mod draw;
#[path = "audio_ribbon/pipeline.rs"]
mod pipeline;

impl AudioRibbon {
  pub(super) fn set_viewport(
    &mut self,
    rect: PreviewSurfaceRect,
    scale: f64,
    backdrop: [f64; 4],
  ) -> Result<(), String> {
    let size = (
      (rect.width * scale).round().max(2.0) as u32,
      (rect.height * scale).round().max(2.0) as u32,
    );
    if size != self.viewport {
      self.surface.resize(self.shared, size);
      self.viewport = size;
      self.dirty = true;
    }
    self.offset = ((rect.x * scale) as f32, (rect.y * scale) as f32);
    self.scale = scale.max(0.1) as f32;
    let luminance = backdrop[0] * 0.2126 + backdrop[1] * 0.7152 + backdrop[2] * 0.0722;
    self.neutral = if luminance > 0.5 { 0.0 } else { 1.0 };
    self.draw()?;
    self.sync_visibility()
  }

  pub(super) fn set_envelopes(&mut self, values: &AudioRibbonEnvelopes) -> Result<(), String> {
    let samples = crate::editor::recording_preview_player::audio_visualizer::bucket_levels(values);
    self.points = values.points;
    self.dirty = true;
    if samples.is_empty() {
      self.levels = None;
      self.surface.resize(self.shared, (2, 2));
      self.viewport = (2, 2);
      return self.sync_visibility();
    }
    let levels = self
      .shared
      .texture_with_pixels(
        "Screenwide audio ribbon levels",
        (samples.len() as u32, 1, 1),
        wgpu::TextureFormat::R32Float,
        bytemuck::cast_slice(&samples),
      )
      .create_view(&Default::default());
    self.levels = Some(self.bindings(&levels));
    self.draw()?;
    self.sync_visibility()
  }

  pub(super) fn has_envelopes(&self) -> bool {
    self.levels.is_some()
  }

  pub(super) fn set_playhead(&mut self, ratio: f64) -> Result<(), String> {
    if self.levels.is_none() {
      return Ok(());
    }
    self.playhead = ratio.clamp(0.0, 1.0) as f32;
    self.draw()?;
    self.sync_visibility()
  }

  fn sync_visibility(&self) -> Result<(), String> {
    let x = if self.levels.is_some() {
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
