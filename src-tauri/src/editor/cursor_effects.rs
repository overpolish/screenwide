// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

#[path = "cursor_effects/geometry_helpers.rs"]
mod geometry_helpers;
#[path = "cursor_effects/record_ingestion.rs"]
mod record_ingestion;

#[cfg(test)]
use geometry_helpers::motion_blur_sample_count;
use geometry_helpers::{
  dwell_anchors, last_at_or_before, output_hotspot, segment_raw_positions, stabilise_positions,
};

use crate::recording::cursor::{
  self, ButtonState, CursorButton, CursorRecord, CursorSource, CursorStyle,
};
use std::path::Path;

mod appearance_timeline;
use appearance_timeline::{normalize_custom_fallback_size, stable_appearances};
#[cfg(target_os = "macos")]
mod gpu_wire;
#[cfg(target_os = "macos")]
pub(crate) use gpu_wire::{NativeGpuArtwork, NativeGpuCursor};
mod raster;
mod settings;
#[cfg(test)]
#[path = "cursor_effects/tests.rs"]
mod tests;
mod timing;
mod visibility;
pub use settings::CursorEffectSettings;

const APPEARANCE_STABILITY_US: u64 = 300_000;
// ScreenCaptureKit's recorded cursor image trails the independently observed
// cursor position slightly, so macOS deliberately samples the sidecar earlier.
// Windows Graphics Capture and GetCursorInfo share the live screen timing; the
// macOS correction would make the baked Windows cursor visibly lag its native
// counterpart even when every cursor effect is disabled.
#[cfg(target_os = "macos")]
const SCREEN_REACTION_US: u64 = 2 * 1_000_000 / 60;
#[cfg(not(target_os = "macos"))]
const SCREEN_REACTION_US: u64 = 0;
const POSITION_SEGMENT_GAP_US: u64 = 100_000;
const POSITION_DWELL_US: u64 = 120_000;
const POSITION_DWELL_SPAN: f64 = 0.015;
const POSITION_DWELL_CORE: f64 = 0.4;
#[cfg(test)]
const MAX_BLUR_DISTANCE: f64 = 80.0;
#[cfg(test)]
const MAX_BLUR_SAMPLES: usize = 48;

#[derive(Clone, Copy)]
struct Appearance {
  height: f64,
  hotspot_x: f64,
  hotspot_y: f64,
  style: CursorStyle,
  timestamp_us: u64,
  width: f64,
}

#[derive(Clone, Copy)]
struct Position {
  segment: u32,
  timestamp_us: u64,
  x: f64,
  y: f64,
}

#[derive(Clone, Copy)]
struct ButtonEvent {
  state: ButtonState,
  timestamp_us: u64,
}

#[derive(Clone, Copy)]
struct DwellAnchor {
  end_us: u64,
  start_us: u64,
  x: f64,
  y: f64,
}

#[derive(Clone, Copy)]
struct EvaluatedCursor {
  appearance: Appearance,
  rotation_degrees: f64,
  scale: f64,
  segment: u32,
  x: f64,
  y: f64,
}

#[derive(Clone, Copy)]
struct OutputCursor {
  cursor: EvaluatedCursor,
  delta_x: f64,
  delta_y: f64,
  height: f64,
  hotspot_x: f64,
  hotspot_y: f64,
  width: f64,
  x: f64,
  y: f64,
}

/// Small, frame-local cursor description consumed by native GPU compositors.
/// Evaluating the event timeline is CPU work over a few numbers; cursor pixels,
/// animation, blur and blending remain entirely in the graphics shader.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuCursor {
  pub opacity: f32,
  pub blur_delta_x: f32,
  pub blur_delta_y: f32,
  pub height: f32,
  pub hotspot_x: f32,
  pub hotspot_y: f32,
  pub rotation_radians: f32,
  pub scale: f32,
  pub style: u32,
  pub width: f32,
  pub x: f32,
  pub y: f32,
  pub clip_at_video_edge: bool,
}

#[derive(Clone)]
pub struct CursorCompositor {
  visibility: Vec<(u64, bool)>,
  appearances: Vec<Appearance>,
  button_events: Vec<ButtonEvent>,
  dwell_anchors: Vec<DwellAnchor>,
  raw_positions: Vec<Position>,
  positions: Vec<Position>,
  source: CursorSource,
}

impl CursorCompositor {
  fn output_cursor(
    &self,
    position_ms: u64,
    width: usize,
    height: usize,
    settings: CursorEffectSettings,
  ) -> Option<OutputCursor> {
    let timestamp_us = position_ms
      .saturating_mul(1_000)
      .saturating_sub(SCREEN_REACTION_US);
    let cursor = self.evaluate(timestamp_us, settings)?;
    let (hotspot_x, hotspot_y) = output_hotspot(cursor.appearance);
    let previous = self.evaluate(timestamp_us.saturating_sub(1_000_000 / 60), settings);
    let (delta_x, delta_y) = previous
      .filter(|previous| previous.segment == cursor.segment)
      .map_or((0.0, 0.0), |previous| {
        (
          (cursor.x - previous.x) / self.source.width * width as f64,
          (cursor.y - previous.y) / self.source.height * height as f64,
        )
      });
    Some(OutputCursor {
      cursor,
      delta_x,
      delta_y,
      height: cursor.appearance.height / self.source.height * height as f64,
      hotspot_x: hotspot_x / self.source.width * width as f64,
      hotspot_y: hotspot_y / self.source.height * height as f64,
      width: cursor.appearance.width / self.source.width * width as f64,
      x: (cursor.x - self.source.x) / self.source.width * width as f64,
      y: (cursor.y - self.source.y) / self.source.height * height as f64,
    })
  }

  /// Evaluates the event timeline for one output frame. The result carries no
  /// pixels: the native compositor scales, rotates, blurs and blends the
  /// style's artwork from these few numbers.
  pub(in crate::editor) fn gpu_cursor(
    &self,
    position_ms: u64,
    source_size: (u32, u32),
    settings: CursorEffectSettings,
  ) -> Option<GpuCursor> {
    let output = self.output_cursor(
      position_ms,
      source_size.0 as usize,
      source_size.1 as usize,
      settings,
    )?;
    // Windows composites from an eight-slice atlas of the standard system
    // cursors; every other platform indexes the shared artwork order.
    #[cfg(target_os = "windows")]
    let artwork = match output.cursor.appearance.style {
      CursorStyle::IBeam => 1,
      CursorStyle::VerticalIBeam => 2,
      CursorStyle::ResizeHorizontal => 3,
      CursorStyle::ResizeVertical => 4,
      CursorStyle::PointingHand | CursorStyle::ClosedHand | CursorStyle::OpenHand => 5,
      CursorStyle::Crosshair => 6,
      CursorStyle::NotAllowed => 7,
      _ => 0,
    };
    #[cfg(not(target_os = "windows"))]
    let artwork = raster::artwork_index(output.cursor.appearance.style);
    #[cfg(target_os = "macos")]
    let rotation_adjustment = raster::gpu_rotation_radians(output.cursor.appearance.style);
    #[cfg(not(target_os = "macos"))]
    let rotation_adjustment = 0.0;
    Some(GpuCursor {
      opacity: self.visibility_opacity(
        position_ms
          .saturating_mul(1_000)
          .saturating_sub(SCREEN_REACTION_US),
      ),
      blur_delta_x: if settings.motion_blur {
        output.delta_x as f32
      } else {
        0.0
      },
      blur_delta_y: if settings.motion_blur {
        output.delta_y as f32
      } else {
        0.0
      },
      height: output.height as f32,
      hotspot_x: output.hotspot_x as f32,
      hotspot_y: output.hotspot_y as f32,
      rotation_radians: output.cursor.rotation_degrees.to_radians() as f32 + rotation_adjustment,
      scale: (output.cursor.scale * settings.size_percent.clamp(50.0, 500.0) / 100.0) as f32,
      style: artwork,
      width: output.width as f32,
      // Keep the artwork origin on the output pixel grid. Fractional cursor
      // event positions otherwise make opposite edges sample different texel
      // coverage, producing the alternating thin/thick outline seen during
      // very small movements.
      x: output.x.round() as f32,
      y: output.y.round() as f32,
      clip_at_video_edge: settings.clip_at_video_edge,
    })
  }

  fn evaluate(&self, timestamp_us: u64, settings: CursorEffectSettings) -> Option<EvaluatedCursor> {
    if self.visibility_opacity(timestamp_us) <= 0.0 {
      return None;
    }
    let appearance = *self.appearances.get(last_at_or_before(
      &self.appearances,
      timestamp_us,
      |appearance| appearance.timestamp_us,
    )?)?;
    let current = self.smoothed_position(timestamp_us, settings.smooth_movement)?;
    let rotation_degrees = if settings.smooth_movement {
      let cursor_size_pixels =
        appearance.width.max(appearance.height) * settings.size_percent.clamp(50.0, 500.0) / 100.0;
      self.motion_lean_degrees(timestamp_us, cursor_size_pixels)
    } else {
      0.0
    };
    Some(EvaluatedCursor {
      appearance,
      rotation_degrees,
      scale: if settings.click_animation {
        self.click_scale(timestamp_us)
      } else {
        1.0
      },
      segment: current.segment,
      x: current.x,
      y: current.y,
    })
  }
}

pub(crate) fn initialize_artwork() {
  raster::initialize_system_artwork();
}

#[cfg(target_os = "macos")]
pub(crate) use raster::GpuArtwork;

/// The style-indexed artwork the native compositor uploads once per export.
#[cfg(target_os = "macos")]
pub(in crate::editor) fn gpu_artworks() -> Vec<GpuArtwork> {
  raster::gpu_artworks()
}
