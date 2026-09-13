// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn output_hotspot(appearance: Appearance) -> (f64, f64) {
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

pub(super) fn median(values: impl Iterator<Item = f64>) -> f64 {
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
pub(super) fn stabilise_positions(positions: &[Position], source_width: f64) -> Vec<Position> {
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

pub(super) fn segment_raw_positions(positions: &mut [Position]) {
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

pub(super) fn dwell_anchors(positions: &[Position]) -> Vec<DwellAnchor> {
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

pub(super) fn last_at_or_before<T>(
  values: &[T],
  timestamp_us: u64,
  timestamp: impl Fn(&T) -> u64,
) -> Option<usize> {
  let index = values.partition_point(|value| timestamp(value) <= timestamp_us);
  index.checked_sub(1)
}

#[cfg(test)]
pub(super) fn motion_blur_sample_count(distance: f64) -> usize {
  ((distance / 2.0).ceil() as usize + 1).clamp(8, MAX_BLUR_SAMPLES)
}
