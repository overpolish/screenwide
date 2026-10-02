// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recording's audio ribbon: a row of rounded bars drawn by
//! `audio_ribbon.wgsl` into a surface the platform places. The platform
//! supplies the colours and the backing scale; everything measured here is
//! in drawable pixels.

mod pipeline;
#[cfg(test)]
mod tests;

use pipeline::Painter;

use crate::editor::recording_preview_player::audio_visualizer::{
  bucket_levels, AudioRibbonEnvelopes,
};
use crate::gpu::surface::{Frame, Surface};
use crate::gpu::Gpu;

/// Two points of bar with a three-point gap.
const BAR_WIDTH_POINTS: f32 = 2.0;
const BAR_PITCH_POINTS: f32 = 5.0;
/// A bar reaches this share of the viewport's height at most, half of it
/// either side of the centre line.
const HEIGHT_SHARE: f32 = 0.30;

/// The twin of `Ribbon` in `audio_ribbon.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Constants {
  color: [f32; 4],
  flat: [f32; 4],
  geometry: [f32; 4],
  style: [f32; 4],
}

/// What one frame is drawn with besides the envelopes.
#[derive(Clone, Copy)]
pub(crate) struct RibbonLook {
  /// Drawable pixels per point.
  pub(crate) scale: f32,
  /// Where the playhead stands through the recording, 0 to 1.
  pub(crate) playhead: f32,
  /// The recording's bars, and the bars before its start and after its end.
  pub(crate) color: [f32; 4],
  pub(crate) flat: [f32; 4],
}

pub(crate) struct AudioRibbonRenderer {
  gpu: &'static Gpu,
  surface: Surface,
  painter: Painter,
  /// The constants and the bucketed levels, once there are levels to draw.
  levels: Option<wgpu::BindGroup>,
  points: u32,
  dirty: bool,
  last_constants: Option<Constants>,
}

impl AudioRibbonRenderer {
  /// A renderer drawing into `surface`, empty until it has levels.
  pub(crate) fn new(gpu: &'static Gpu, surface: Surface) -> Self {
    Self {
      gpu,
      surface,
      painter: Painter::new(&gpu.device),
      levels: None,
      points: 0,
      dirty: true,
      last_constants: None,
    }
  }

  /// Matches the surface to `size` drawable pixels.
  pub(crate) fn set_viewport(&mut self, size: (u32, u32)) {
    let size = (size.0.max(2), size.1.max(2));
    if size != self.surface.size() {
      self.surface.resize(self.gpu, size);
      self.dirty = true;
    }
  }

  pub(crate) fn set_envelopes(&mut self, envelopes: &AudioRibbonEnvelopes) {
    let samples = bucket_levels(envelopes);
    self.points = envelopes.points;
    self.dirty = true;
    if samples.is_empty() {
      self.levels = None;
      self.surface.resize(self.gpu, (2, 2));
      return;
    }
    let levels = levels_view(self.gpu, &samples);
    self.levels = Some(self.painter.bindings(&self.gpu.device, &levels));
  }

  #[cfg_attr(target_os = "macos", allow(dead_code))]
  pub(crate) fn has_levels(&self) -> bool {
    self.levels.is_some()
  }

  /// Presents one frame, unless it would be identical to the last one: a
  /// paused waveform needs no repeated presents, while new envelopes or a
  /// resize still redraw it. Returns whether a frame was submitted.
  pub(crate) fn draw(&mut self, look: RibbonLook) -> Result<bool, String> {
    let Some(levels) = self.levels.as_ref() else {
      return Ok(false);
    };
    let (width, height) = self.surface.size();
    let constants = Constants {
      color: look.color,
      flat: look.flat,
      geometry: [
        width as f32,
        height as f32,
        BAR_PITCH_POINTS * look.scale,
        BAR_WIDTH_POINTS * look.scale,
      ],
      style: [
        height as f32 * HEIGHT_SHARE,
        look.scale,
        look.playhead.clamp(0.0, 1.0),
        self.points as f32,
      ],
    };
    if !self.dirty && self.last_constants == Some(constants) {
      return Ok(false);
    }
    let Frame::Ready(frame) = self.surface.acquire(self.gpu)? else {
      return Ok(false);
    };
    let target = frame.texture.create_view(&Default::default());
    self.painter.encode(self.gpu, levels, constants, &target);
    self.gpu.queue.present(frame);
    self.dirty = false;
    self.last_constants = Some(constants);
    Ok(true)
  }
}

/// One R32Float texel per bucket.
fn levels_view(gpu: &Gpu, samples: &[f32]) -> wgpu::TextureView {
  gpu
    .texture_with_pixels(
      "Screenwide audio ribbon levels",
      (samples.len() as u32, 1, 1),
      wgpu::TextureFormat::R32Float,
      bytemuck::cast_slice(samples),
    )
    .create_view(&Default::default())
}
