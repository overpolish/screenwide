// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

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

fn output_hotspot(appearance: Appearance) -> (f64, f64) {
  if appearance.style == CursorStyle::Custom {
    // The sidecar does not carry custom cursor pixels yet, so a custom cursor
    // stands in the system arrow: the recorded hotspot addresses artwork that
    // is not being drawn and is dropped here. The stand-in is anchored by its
    // own hotspot instead, which travels with the artwork rather than the
    // cursor frame (`custom_gpu_artwork`'s `origin`, cursor_effects/raster.rs),
    // exactly as the Windows compositor anchors its atlas entries by
    // `native_cursor_hotspots` (preview_platform/surface_windows
    // /compositor.rs:118). A zero here puts that origin at the recorded
    // position.
    return (0.0, 0.0);
  }
  (appearance.hotspot_x, appearance.hotspot_y)
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

fn median(values: impl Iterator<Item = f64>) -> f64 {
  let mut values = values.collect::<Vec<_>>();
  values.sort_by(f64::total_cmp);
  let middle = values.len() / 2;
  if values.len() % 2 == 0 {
    (values[middle - 1] + values[middle]) * 0.5
  } else {
    values[middle]
  }
}

/// Collapses a settled cloud of tiny OS cursor corrections into one held
/// position. Fast travel cannot qualify because the cloud must remain inside
/// the spatial radius for long enough to be a deliberate dwell.
fn stabilise_positions(positions: &[Position], source_width: f64) -> Vec<Position> {
  if positions.len() < 2 {
    return positions.to_vec();
  }
  let radius = source_width.max(1.0) * POSITION_DWELL_SPAN;
  let mut stable = Vec::with_capacity(positions.len());
  let mut start = 0;
  while start < positions.len() {
    let mut end = start + 1;
    let mut min_x = positions[start].x;
    let mut max_x = positions[start].x;
    let mut min_y = positions[start].y;
    let mut max_y = positions[start].y;
    while end < positions.len() {
      let next_min_x = min_x.min(positions[end].x);
      let next_max_x = max_x.max(positions[end].x);
      let next_min_y = min_y.min(positions[end].y);
      let next_max_y = max_y.max(positions[end].y);
      if (next_max_x - next_min_x).hypot(next_max_y - next_min_y) > radius {
        break;
      }
      min_x = next_min_x;
      max_x = next_max_x;
      min_y = next_min_y;
      max_y = next_max_y;
      end += 1;
    }
    let cloud = &positions[start..end];
    let x = median(cloud.iter().map(|position| position.x));
    let y = median(cloud.iter().map(|position| position.y));
    let core_radius = radius * POSITION_DWELL_CORE;
    let mut core_start = None;
    let mut core_end = None;
    for (offset, position) in cloud.iter().enumerate() {
      if (position.x - x).hypot(position.y - y) <= core_radius {
        core_start.get_or_insert(start + offset);
        core_end = Some(start + offset);
      }
    }
    if let (Some(core_start), Some(core_end)) = (core_start, core_end) {
      if positions[core_end]
        .timestamp_us
        .saturating_sub(positions[core_start].timestamp_us)
        < POSITION_DWELL_US
      {
        stable.push(positions[start]);
        start += 1;
        continue;
      }
      stable.extend_from_slice(&positions[start..core_start]);
      stable.push(Position {
        x,
        y,
        ..positions[core_start]
      });
      if core_end != core_start {
        stable.push(Position {
          x,
          y,
          ..positions[core_end]
        });
      }
      start = core_end + 1;
    } else {
      stable.push(positions[start]);
      start += 1;
    }
  }
  let mut segment = 0_u32;
  for index in 1..stable.len() {
    let previous = stable[index - 1];
    let current = &mut stable[index];
    let distance = (current.x - previous.x).hypot(current.y - previous.y);
    if current.timestamp_us.saturating_sub(previous.timestamp_us) > POSITION_SEGMENT_GAP_US
      && distance > radius
    {
      segment = segment.saturating_add(1);
    }
    current.segment = segment;
  }
  stable
}

fn segment_raw_positions(positions: &mut [Position]) {
  let mut segment = 0_u32;
  for index in 1..positions.len() {
    if positions[index]
      .timestamp_us
      .saturating_sub(positions[index - 1].timestamp_us)
      > POSITION_SEGMENT_GAP_US
    {
      segment = segment.saturating_add(1);
    }
    positions[index].segment = segment;
  }
}

fn dwell_anchors(positions: &[Position]) -> Vec<DwellAnchor> {
  positions
    .windows(2)
    .filter_map(|pair| {
      let current = pair[0];
      let next = pair[1];
      let held_us = next.timestamp_us.saturating_sub(current.timestamp_us);
      (held_us >= POSITION_DWELL_US).then_some(DwellAnchor {
        end_us: next.timestamp_us,
        start_us: current.timestamp_us,
        x: current.x,
        y: current.y,
      })
    })
    .collect()
}

fn last_at_or_before<T>(
  values: &[T],
  timestamp_us: u64,
  timestamp: impl Fn(&T) -> u64,
) -> Option<usize> {
  let index = values.partition_point(|value| timestamp(value) <= timestamp_us);
  index.checked_sub(1)
}

#[cfg(test)]
fn motion_blur_sample_count(distance: f64) -> usize {
  ((distance / 2.0).ceil() as usize + 1).clamp(8, MAX_BLUR_SAMPLES)
}

impl CursorCompositor {
  pub fn open(path: &Path) -> Result<Self, String> {
    let records = cursor::read(path)?;
    Self::from_records(&records)
  }

  fn from_records(records: &[CursorRecord]) -> Result<Self, String> {
    let source = records
      .iter()
      .find_map(|record| match record {
        CursorRecord::Header { source, .. } => Some(source.clone()),
        _ => None,
      })
      .ok_or_else(|| "The cursor recording has no source".to_owned())?;
    let mut appearances: Vec<_> = records
      .iter()
      .filter_map(|record| match record {
        CursorRecord::Appearance {
          height,
          hotspot_x,
          hotspot_y,
          style,
          timestamp_us,
          width,
        } => Some(Appearance {
          height: *height,
          hotspot_x: *hotspot_x,
          hotspot_y: *hotspot_y,
          style: *style,
          timestamp_us: *timestamp_us,
          width: *width,
        }),
        _ => None,
      })
      .collect();
    appearances.sort_by_key(|appearance| appearance.timestamp_us);
    normalize_custom_fallback_size(&mut appearances);
    let visibility = visibility::events(records);
    let mut raw_positions: Vec<_> = records
      .iter()
      .filter_map(|record| match record {
        CursorRecord::Visibility {
          timestamp_us, x, y, ..
        }
        | CursorRecord::Position { timestamp_us, x, y }
        | CursorRecord::Button {
          timestamp_us, x, y, ..
        } => Some(Position {
          segment: 0,
          timestamp_us: *timestamp_us,
          x: *x,
          y: *y,
        }),
        _ => None,
      })
      .collect();
    raw_positions.sort_by_key(|position| position.timestamp_us);
    // macOS's cursor smoothing is already tuned and shipped. Windows polling
    // exposes explicit held intervals that must remain exact anchors.
    let dwell_anchors = if visibility.is_empty() && cfg!(target_os = "windows") {
      dwell_anchors(&raw_positions)
    } else {
      Vec::new()
    };
    segment_raw_positions(&mut raw_positions);
    let mut positions = if visibility.is_empty() && cfg!(target_os = "windows") {
      stabilise_positions(&raw_positions, source.width)
    } else {
      // Event-driven macOS cursor positions are already the canonical path.
      // Keep its proven smoothing input free of Windows polling heuristics.
      raw_positions.clone()
    };
    visibility::segment(&mut raw_positions, &visibility);
    visibility::segment(&mut positions, &visibility);
    let recording_end_us = raw_positions
      .last()
      .map_or(0, |position| position.timestamp_us);
    let stable = stable_appearances(&appearances, recording_end_us);
    let mut raw_button_events = records
      .iter()
      .filter_map(|record| match record {
        CursorRecord::Button {
          button,
          state,
          timestamp_us,
          ..
        } => Some((*timestamp_us, *button, *state)),
        _ => None,
      })
      .collect::<Vec<_>>();
    raw_button_events.sort_by_key(|(timestamp_us, ..)| *timestamp_us);
    let mut pressed = Vec::<CursorButton>::new();
    let mut button_events = Vec::new();
    for (timestamp_us, button, state) in raw_button_events {
      match state {
        ButtonState::Down if !pressed.contains(&button) => {
          if pressed.is_empty() {
            button_events.push(ButtonEvent {
              state,
              timestamp_us,
            });
          }
          pressed.push(button);
        }
        ButtonState::Up if pressed.contains(&button) => {
          pressed.retain(|pressed_button| *pressed_button != button);
          if pressed.is_empty() {
            button_events.push(ButtonEvent {
              state,
              timestamp_us,
            });
          }
        }
        _ => {}
      }
    }
    Ok(Self {
      visibility,
      appearances: stable,
      button_events,
      dwell_anchors,
      raw_positions,
      positions,
      source,
    })
  }

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
  pub(in crate::exports) fn gpu_cursor(
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
pub(in crate::exports) fn gpu_artworks() -> Vec<GpuArtwork> {
  raster::gpu_artworks()
}
