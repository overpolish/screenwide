// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Recording-wide keyboard bounds shared by every compositor consumer.

use super::state::VisualKey;

pub(super) const DESIGN_HEIGHT: f64 = 20.0;
pub(super) const GAP: f64 = 4.0;
const BASE_HEIGHT_FRACTION: f64 = 60.0 / 1080.0;
const EDGE_MARGIN_FRACTION: f64 = 0.055;
const ANIMATION_EXTENT: f64 = 1.12;

fn is_modifier(code: u16) -> bool {
  matches!(code, 54 | 55 | 56 | 58 | 59 | 60 | 61 | 62 | 63)
}

fn key_width(code: u16) -> f64 {
  // Windows draws these keys as text rather than the macOS glyphs, so their
  // bounds differ per platform. Slight overestimates are safe: the fitting
  // bound stops growth early instead of overflowing the edge margins.
  #[cfg(target_os = "windows")]
  {
    let text_label = match code {
      51 => 76.0,                // Backspace
      57 | 121 => 72.0,          // Caps Lock, Page Down
      71 => 66.0,                // Num Lock
      114 => 50.0,               // Insert
      36 | 76 => 48.0,           // Enter
      56 | 60 => 46.0,           // Shift
      54 | 55 | 59 | 62 => 38.0, // Win, Ctrl
      48 => 36.0,                // Tab
      58 | 61 => 34.0,           // Alt
      117 => 30.0,               // Del
      _ => 0.0,
    };
    if text_label > 0.0 {
      return text_label;
    }
  }
  match code {
    54..=62 => 24.0,
    63 => 28.0,
    49 => 52.0,
    71 | 114 | 115 => 52.0,
    116 | 121 => 62.0,
    53 | 119 => 36.0,
    96..=113 | 118 | 120 | 122 => 36.0,
    0..=126 => 22.0,
    _ => 72.0,
  }
}

fn legacy_width(key: &VisualKey) -> f64 {
  let modifier_count = (key.modifier_mask & 0x1f).count_ones() as usize;
  let modifier_width = modifier_count as f64 * 24.0;
  key_width(key.key_code) + modifier_width + GAP * modifier_count as f64
}

pub(super) fn maximum_width(
  visuals: &[VisualKey],
  slots: &[u32],
  legacy_modifier_expansion: bool,
) -> f64 {
  let width = slots
    .iter()
    .map(|slot| {
      visuals
        .iter()
        .filter(|visual| visual.slot_id == *slot)
        .map(|visual| {
          if legacy_modifier_expansion && !is_modifier(visual.key_code) {
            legacy_width(visual)
          } else {
            key_width(visual.key_code)
          }
        })
        .fold(0.0, f64::max)
    })
    .sum::<f64>();
  width + GAP * slots.len().saturating_sub(1) as f64
}

pub(super) fn maximum_size_percent(maximum_width: f64, width: u32, height: u32) -> f64 {
  if maximum_width <= 0.0 || width == 0 || height == 0 {
    return 500.0;
  }
  let available_width = f64::from(width) * (1.0 - EDGE_MARGIN_FRACTION * 2.0);
  let width_at_unit_scale =
    f64::from(height) * BASE_HEIGHT_FRACTION * maximum_width / DESIGN_HEIGHT;
  let exact = available_width / (width_at_unit_scale * ANIMATION_EXTENT) * 100.0;
  ((exact / 5.0).floor() * 5.0).clamp(5.0, 500.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bounds_a_long_chord_with_margin_and_animation_headroom() {
    assert_eq!(maximum_size_percent(138.0, 1_920, 1_080), 365.0);
  }

  #[test]
  fn keeps_the_product_ceiling_when_the_chord_has_room() {
    assert_eq!(maximum_size_percent(50.0, 3_840, 2_160), 500.0);
  }
}
