// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[path = "timing/tests.rs"]
mod tests;

use super::*;

const MOTION_LOOKBACK_US: u64 = 140_000;
const MOTION_LOOKAHEAD_US: u64 = 220_000;
const MOTION_SAMPLES: usize = 24;
const MOTION_WARMUP_US: u64 = 140_000;
const MAX_LEAN_DEGREES: f64 = 14.0;
const FULL_LEAN_DISTANCE: f64 = 0.09;
const FULL_LEAN_SPEED: f64 = 0.65;
const MIN_LEAN_DISTANCE: f64 = 0.006;
const MIN_LEAN_SPEED: f64 = 0.03;
const POSITION_SMOOTHING_RADIUS_US: u64 = 84_000;
const POSITION_SMOOTHING_SIGMA_US: f64 = 28_000.0;
const POSITION_ANCHOR_EASING_US: u64 = 84_000;
const REFERENCE_CURSOR_SIZE: f64 = 32.0;

impl CursorCompositor {
  fn position_at(&self, timestamp_us: u64, stabilised: bool) -> Option<Position> {
    let positions = if stabilised {
      &self.positions
    } else {
      &self.raw_positions
    };
    let index = last_at_or_before(positions, timestamp_us, |position| position.timestamp_us)?;
    let current = positions[index];
    let Some(next) = positions.get(index + 1).copied() else {
      return Some(current);
    };
    let duration = next.timestamp_us.saturating_sub(current.timestamp_us);
    if duration == 0 || current.segment != next.segment {
      return Some(current);
    }
    let progress = timestamp_us.saturating_sub(current.timestamp_us) as f64 / duration as f64;
    Some(Position {
      segment: current.segment,
      timestamp_us,
      x: current.x + (next.x - current.x) * progress,
      y: current.y + (next.y - current.y) * progress,
    })
  }

  pub(super) fn smoothed_position(&self, timestamp_us: u64, enabled: bool) -> Option<Position> {
    let current = self.position_at(timestamp_us, enabled)?;
    if !enabled {
      return Some(current);
    }
    // The complete recording lets us use a centred filter. Unlike a trailing
    // average, it removes capture jitter without making the rendered pointer
    // visibly chase the real one.
    let filtered = self.gaussian_position(
      timestamp_us,
      current,
      POSITION_SMOOTHING_RADIUS_US,
      POSITION_SMOOTHING_SIGMA_US,
    );
    Some(self.anchored_position(timestamp_us, filtered))
  }

  fn anchored_position(&self, timestamp_us: u64, filtered: Position) -> Position {
    let next_index = self
      .dwell_anchors
      .partition_point(|anchor| anchor.start_us <= timestamp_us);
    if let Some(anchor) = next_index
      .checked_sub(1)
      .and_then(|index| self.dwell_anchors.get(index))
    {
      if timestamp_us <= anchor.end_us {
        return Position {
          x: anchor.x,
          y: anchor.y,
          ..filtered
        };
      }
      let elapsed = timestamp_us.saturating_sub(anchor.end_us);
      if elapsed < POSITION_ANCHOR_EASING_US {
        let progress = smoothstep(0.0, POSITION_ANCHOR_EASING_US as f64, elapsed as f64);
        return Position {
          x: anchor.x + (filtered.x - anchor.x) * progress,
          y: anchor.y + (filtered.y - anchor.y) * progress,
          ..filtered
        };
      }
    }
    if let Some(anchor) = self.dwell_anchors.get(next_index) {
      let remaining = anchor.start_us.saturating_sub(timestamp_us);
      if remaining < POSITION_ANCHOR_EASING_US {
        let progress = smoothstep(
          0.0,
          POSITION_ANCHOR_EASING_US as f64,
          POSITION_ANCHOR_EASING_US.saturating_sub(remaining) as f64,
        );
        return Position {
          x: filtered.x + (anchor.x - filtered.x) * progress,
          y: filtered.y + (anchor.y - filtered.y) * progress,
          ..filtered
        };
      }
    }
    filtered
  }

  fn gaussian_position(
    &self,
    timestamp_us: u64,
    current: Position,
    radius_us: u64,
    sigma_us: f64,
  ) -> Position {
    let mut total_weight = 0.0;
    let mut x = 0.0;
    let mut y = 0.0;
    for index in -6_i64..=6 {
      let offset_us = index * radius_us as i64 / 6;
      let sample_us = timestamp_us.saturating_add_signed(offset_us);
      if let Some(sample) = self.position_at(sample_us, true) {
        if sample.segment != current.segment {
          continue;
        }
        let weight = (-0.5 * (offset_us as f64 / sigma_us).powi(2)).exp();
        x += sample.x * weight;
        y += sample.y * weight;
        total_weight += weight;
      }
    }
    if total_weight == 0.0 {
      return current;
    }
    Position {
      segment: current.segment,
      timestamp_us,
      x: x / total_weight,
      y: y / total_weight,
    }
  }

  pub(super) fn motion_lean_degrees(&self, timestamp_us: u64, cursor_size_pixels: f64) -> f64 {
    let Some(current) = self.smoothed_position(timestamp_us, true) else {
      return 0.0;
    };
    let segment_start_us = self
      .positions
      .iter()
      .find(|position| position.segment == current.segment)
      .map_or(timestamp_us, |position| position.timestamp_us);
    let start_us = timestamp_us
      .saturating_sub(MOTION_LOOKBACK_US)
      .max(segment_start_us);
    let end_us = timestamp_us.saturating_add(MOTION_LOOKAHEAD_US);
    let duration_seconds = end_us.saturating_sub(start_us) as f64 / 1_000_000.0;
    if duration_seconds <= 0.0 || self.source.width <= 0.0 {
      return 0.0;
    }

    // Rotation represents momentum along the visible trajectory, not the
    // instantaneous acceleration between two cursor samples. Looking ahead
    // carries a fast gesture naturally into a connected slower section, while
    // the accumulated path distance prevents tiny fast flicks from leaning as
    // much as a deliberate long movement.
    let mut first: Option<Position> = None;
    let mut previous: Option<Position> = None;
    let mut last: Option<Position> = None;
    let mut path_distance = 0.0;
    for step in 0..=MOTION_SAMPLES {
      let sample_us = start_us.saturating_add(
        (end_us.saturating_sub(start_us) as u128 * step as u128 / MOTION_SAMPLES as u128) as u64,
      );
      let Some(sample) = self
        .smoothed_position(sample_us, true)
        .filter(|position| position.segment == current.segment)
      else {
        continue;
      };
      if let Some(previous_position) = previous {
        let delta_x: f64 = sample.x - previous_position.x;
        let delta_y: f64 = sample.y - previous_position.y;
        path_distance += delta_x.hypot(delta_y);
      } else {
        first = Some(sample);
      }
      previous = Some(sample);
      last = Some(sample);
    }
    let (Some(first), Some(last)) = (first, last) else {
      return 0.0;
    };
    if path_distance <= f64::EPSILON {
      return 0.0;
    }
    let normalized_distance = path_distance / self.source.width;
    let normalized_speed = normalized_distance / duration_seconds;
    let distance_weight = smoothstep(MIN_LEAN_DISTANCE, FULL_LEAN_DISTANCE, normalized_distance);
    let speed_weight = smoothstep(MIN_LEAN_SPEED, FULL_LEAN_SPEED, normalized_speed);
    let horizontal_direction = ((last.x - first.x) / path_distance).clamp(-1.0, 1.0);
    // Larger artwork needs more visual inertia. Square-root scaling keeps a
    // giant cursor restrained without making its motion completely rigid.
    let size_weight = (REFERENCE_CURSOR_SIZE / cursor_size_pixels.max(1.0))
      .sqrt()
      .clamp(0.4, 1.4);
    let warmup = smoothstep(
      0.0,
      MOTION_WARMUP_US as f64,
      timestamp_us.saturating_sub(segment_start_us) as f64,
    );
    MAX_LEAN_DEGREES * horizontal_direction * distance_weight * speed_weight * size_weight * warmup
  }

  pub(super) fn click_scale(&self, timestamp_us: u64) -> f64 {
    let Some(index) = last_at_or_before(&self.button_events, timestamp_us, |event| {
      event.timestamp_us
    }) else {
      return 1.0;
    };
    self.click_scale_after(index, timestamp_us)
  }

  fn click_scale_after(&self, index: usize, timestamp_us: u64) -> f64 {
    let event = self.button_events[index];
    let elapsed_seconds = timestamp_us.saturating_sub(event.timestamp_us) as f64 / 1_000_000.0;
    // Once a release has settled it is also a natural history boundary. This
    // keeps evaluation constant-size for long recordings with many clicks.
    if event.state == ButtonState::Up && elapsed_seconds >= 0.5 {
      return 1.0;
    }
    let start = index.checked_sub(1).map_or(1.0, |previous| {
      self.click_scale_after(previous, event.timestamp_us)
    });
    match event.state {
      ButtonState::Down => pressed_scale(elapsed_seconds, start),
      ButtonState::Up => released_scale(elapsed_seconds, start),
    }
  }
}

fn smoothstep(start: f64, end: f64, value: f64) -> f64 {
  let progress = ((value - start) / (end - start)).clamp(0.0, 1.0);
  progress * progress * (3.0 - 2.0 * progress)
}

fn pressed_scale(elapsed_seconds: f64, start: f64) -> f64 {
  let progress = 1.0 - (1.0 + 32.0 * elapsed_seconds) * (-32.0 * elapsed_seconds).exp();
  start + (0.86 - start) * progress.clamp(0.0, 1.0)
}

fn released_scale(elapsed_seconds: f64, released_from: f64) -> f64 {
  if elapsed_seconds >= 0.5 {
    return 1.0;
  }
  let damping = 0.68;
  let frequency = 22.0;
  let damped_frequency = frequency * f64::sqrt(1.0 - damping * damping);
  let response = 1.0
    - (-damping * frequency * elapsed_seconds).exp()
      * ((damped_frequency * elapsed_seconds).cos()
        + damping / f64::sqrt(1.0 - damping * damping)
          * (damped_frequency * elapsed_seconds).sin());
  released_from + (1.0 - released_from) * response
}
