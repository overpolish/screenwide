// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! CPU mirror of the keyboard shader's animated key geometry for hit-testing.

use super::{KeyboardConstants, MAX_KEYS};

fn motion_spring(progress: f32) -> f32 {
  let t = progress.clamp(0.0, 1.0);
  let phase = 6.0 * t;
  if t >= 1.0 {
    1.0
  } else {
    1.0 - (-5.0 * t).exp() * (phase.cos() + (5.0 / 6.0) * phase.sin())
  }
}

fn slot_width(values: &KeyboardConstants, slot: u32) -> f32 {
  (0..values.dimensions[2].min(MAX_KEYS as u32) as usize)
    .filter(|index| values.key_geometry[*index][3] == slot)
    .map(|index| values.key_geometry[index][1] as f32)
    .fold(0.0, f32::max)
}

fn slot_left(values: &KeyboardConstants, slot: u32, mask: u32) -> f32 {
  let count = values.dimensions[2].min(MAX_KEYS as u32) as usize;
  let slots = (0..count)
    .map(|index| values.key_geometry[index][3] + 1)
    .max()
    .unwrap_or(0);
  let gap = (1..count)
    .filter_map(|index| {
      let current = values.key_geometry[index][0];
      let previous = values.key_geometry[index - 1];
      (current > previous[0] + previous[1]).then_some((current - previous[0] - previous[1]) as f32)
    })
    .fold(f32::INFINITY, f32::min);
  let gap = if gap.is_finite() { gap } else { 0.0 };
  let mut total = gap * mask.count_ones().saturating_sub(1) as f32;
  for candidate in 0..slots {
    if mask & (1u32 << candidate) != 0 {
      total += slot_width(values, candidate);
    }
  }
  let mut left = (values.dimensions[0] as f32 - total) * 0.5;
  for candidate in 0..slot {
    if mask & (1u32 << candidate) != 0 {
      left += slot_width(values, candidate) + gap;
    }
  }
  left
}

pub(super) fn calculate(values: &KeyboardConstants, canvas: (u32, u32)) -> Option<[f64; 4]> {
  if values.dimensions[0] == 0 || values.dimensions[1] == 0 || canvas.0 == 0 || canvas.1 == 0 {
    return None;
  }
  let requested = if values.animation[3] > 0.0 {
    values.animation[3]
  } else {
    values.animation[0]
  };
  let effective = if values.animation[2] > 0.0 {
    let available = canvas.0 as f32 * (1.0 - 0.055 * 2.0);
    let unit_width = canvas.1 as f32 * (60.0 / 1080.0) * values.animation[2] / 20.0;
    requested.min(available / (unit_width * 1.12).max(0.0001))
  } else {
    requested
  };
  let row_height = canvas.1 as f32 * (60.0 / 1080.0) * effective;
  let row_width = row_height * values.dimensions[0] as f32 / values.dimensions[1] as f32;
  // A key's group centre on one axis: non-negative is explicit, -1 follows
  // the overlay centre, and at or below -1.5 the key keeps the default.
  let key_axis = |key_value: f32, overlay_value: f32| {
    if key_value >= 0.0 {
      key_value
    } else if key_value > -1.5 {
      overlay_value
    } else {
      -1.0
    }
  };
  let mut bounds = [
    f32::INFINITY,
    f32::INFINITY,
    f32::NEG_INFINITY,
    f32::NEG_INFINITY,
  ];
  for index in 0..values.dimensions[2].min(MAX_KEYS as u32) as usize {
    let geometry = values.key_geometry[index];
    let motion = values.key_motion[index];
    if geometry[2] == 0 || motion[0] <= 0.002 {
      continue;
    }
    let animation_scale = motion[1] / requested.max(0.001);
    if animation_scale <= 0.002 {
      continue;
    }
    let ratio = if values.key_position[index][2] > 0.0 {
      values.key_position[index][2]
    } else {
      1.0
    };
    let key_row_height = row_height * ratio;
    let key_row_width = row_width * ratio;
    let position_x = key_axis(values.key_position[index][0], values.position[0]);
    let position_y = key_axis(values.key_position[index][1], values.position[1]);
    let center_x = if position_x >= 0.0 {
      position_x * canvas.0 as f32
    } else {
      canvas.0 as f32 * 0.5
    };
    let center_y = if position_y >= 0.0 {
      position_y * canvas.1 as f32
    } else {
      canvas.1 as f32 * (1.0 - 0.055) - key_row_height * 0.5
    };
    let slot = geometry[3];
    let masks = values.key_masks[index];
    let from = slot_left(values, slot, masks[0]) + slot_width(values, slot) * 0.5;
    let to = slot_left(values, slot, masks[1]) + slot_width(values, slot) * 0.5;
    let target = from + (to - from) * motion_spring(motion[3]);
    let offset = (target - (geometry[0] as f32 + geometry[1] as f32 * 0.5)) * key_row_width
      / values.dimensions[0] as f32;
    let key_width = key_row_height * geometry[1] as f32 / values.dimensions[1] as f32;
    let key_center = center_x - key_row_width * 0.5
      + key_row_width * (geometry[0] as f32 + geometry[1] as f32 * 0.5)
        / values.dimensions[0] as f32
      + offset;
    let half_width = key_width * animation_scale * 0.5;
    let half_height = key_row_height * animation_scale * 0.5;
    bounds[0] = bounds[0].min(key_center - half_width);
    bounds[1] = bounds[1].min(center_y - half_height);
    bounds[2] = bounds[2].max(key_center + half_width);
    bounds[3] = bounds[3].max(center_y + half_height);
  }
  (bounds[0].is_finite() && bounds[2] > bounds[0] && bounds[3] > bounds[1]).then_some([
    f64::from(bounds[0]) / f64::from(canvas.0),
    f64::from(bounds[1]) / f64::from(canvas.1),
    f64::from(bounds[2] - bounds[0]) / f64::from(canvas.0),
    f64::from(bounds[3] - bounds[1]) / f64::from(canvas.1),
  ])
}

#[cfg(test)]
mod tests {
  use super::*;

  fn two_key_constants() -> KeyboardConstants {
    let mut values = KeyboardConstants {
      dimensions: [100, 20, 2, 0],
      animation: [1.0, 1.0, 100.0, 1.0],
      position: [0.5, 0.5, 0.0, 0.0],
      ..Default::default()
    };
    values.key_geometry[0] = [0, 40, 1, 0];
    values.key_motion[0] = [1.0, 1.0, 1.0, 1.0];
    values.key_masks[0] = [1, 1, 0, 0];
    values.key_geometry[1] = [60, 40, 1, 1];
    values.key_motion[1] = [0.0, 1.0, 1.0, 1.0];
    values.key_masks[1] = [2, 2, 0, 0];
    values
  }

  #[test]
  fn bounds_only_include_keys_visible_at_the_current_frame() {
    let bounds = calculate(&two_key_constants(), (1_000, 1_000)).unwrap();

    assert!((bounds[0] - 0.444_444).abs() < 0.000_01);
    assert!((bounds[1] - 0.472_222).abs() < 0.000_01);
    assert!((bounds[2] - 0.111_111).abs() < 0.000_01);
    assert!((bounds[3] - 0.055_556).abs() < 0.000_01);
  }

  #[test]
  fn bounds_disappear_when_no_key_is_visible() {
    let mut values = two_key_constants();
    values.key_motion[0][0] = 0.0;

    assert_eq!(calculate(&values, (1_000, 1_000)), None);
  }
}
