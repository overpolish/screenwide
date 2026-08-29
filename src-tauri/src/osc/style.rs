// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// Native controls reuse the inverse tooltip palette: tooltip background for
/// the fill and tooltip text for the outline. Keeping this platform-neutral
/// lets later ruler, OCR, and Windows compositors share the exact same tokens.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlPalette {
  pub fill: [f32; 4],
  pub outline: [f32; 4],
}

const NEUTRAL_800: f32 = 38.0 / 255.0;
const WHITE: f32 = 1.0;

pub const fn control_palette(light_appearance: bool) -> ControlPalette {
  let dark = [NEUTRAL_800, NEUTRAL_800, NEUTRAL_800, 1.0];
  let light = [WHITE, WHITE, WHITE, 1.0];
  if light_appearance {
    ControlPalette {
      fill: dark,
      outline: light,
    }
  } else {
    ControlPalette {
      fill: light,
      outline: dark,
    }
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_control_palette(light_mode: u32) -> ControlPalette {
  control_palette(light_mode != 0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn controls_follow_the_inverse_tooltip_tokens() {
    let light = control_palette(true);
    assert_eq!(light.fill, [NEUTRAL_800, NEUTRAL_800, NEUTRAL_800, 1.0]);
    assert_eq!(light.outline, [WHITE; 4]);

    let dark = control_palette(false);
    assert_eq!(dark.fill, [WHITE; 4]);
    assert_eq!(dark.outline, [NEUTRAL_800, NEUTRAL_800, NEUTRAL_800, 1.0]);
  }
}
