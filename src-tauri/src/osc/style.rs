// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// The bounding box, its handles, and the loupe border: a white fill with a
/// dark hairline outline in both appearances. Capture chrome sits over
/// arbitrary desktop content rather than over a window material, so it does
/// not follow the appearance the way window controls do. Keeping this
/// platform-neutral lets the ruler, OCR, and Windows compositors share the
/// exact same tokens.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlPalette {
  pub fill: [f32; 4],
  pub outline: [f32; 4],
}

/// Shared desktop shade for every capture-style OSC. Tools vary their chrome,
/// but the indication that the desktop is under an active overlay remains
/// identical across Region, Quick Screenshot, OCR, and future tools.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayPalette {
  pub shade: [f32; 4],
}

/// Shared ruler crosshair token. The callout itself resolves through the OSC
/// neutral control tokens so it cannot drift from native buttons.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulerPalette {
  pub primary: [f32; 4],
  pub info: [f32; 4],
}

pub const OVERLAY_SHADE_OPACITY: f32 = 0.48;

pub const fn overlay_palette() -> OverlayPalette {
  OverlayPalette {
    shade: [0.0, 0.0, 0.0, OVERLAY_SHADE_OPACITY],
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_overlay_palette() -> OverlayPalette {
  overlay_palette()
}

pub fn ruler_palette(light_appearance: bool) -> RulerPalette {
  let [red, green, blue] = crate::system_accent::accent_rgb();
  let info = if light_appearance {
    [0.0, 136.0 / 255.0, 1.0, 1.0]
  } else {
    [0.0, 145.0 / 255.0, 1.0, 1.0]
  };
  RulerPalette {
    primary: [red, green, blue, 1.0],
    info,
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_ruler_palette(light_mode: u32) -> RulerPalette {
  ruler_palette(light_mode != 0)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OcrPalette {
  pub primary_fill: [f32; 4],
  pub primary_outline: [f32; 4],
  pub qr_fill: [f32; 4],
  pub qr_outline: [f32; 4],
  pub error_fill: [f32; 4],
  pub error_outline: [f32; 4],
  pub selection_fill: [f32; 4],
  pub selection_outline: [f32; 4],
  pub loading_fill: [f32; 4],
  pub loading_foreground: [f32; 4],
  pub status_error_fill: [f32; 4],
  pub status_error_foreground: [f32; 4],
}

const _: () = assert!(std::mem::size_of::<OcrPalette>() == 192);

const NEUTRAL_800: f32 = 38.0 / 255.0;
const WHITE: f32 = 1.0;

pub const fn control_palette(light_appearance: bool) -> ControlPalette {
  let _ = light_appearance;
  ControlPalette {
    fill: [WHITE, WHITE, WHITE, 1.0],
    outline: [NEUTRAL_800, NEUTRAL_800, NEUTRAL_800, 1.0],
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_control_palette(light_mode: u32) -> ControlPalette {
  control_palette(light_mode != 0)
}

pub const fn ocr_palette(light_appearance: bool) -> OcrPalette {
  let (primary, error, loading, status_error) = if light_appearance {
    (
      [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0],
      [215.0 / 255.0, 0.0, 21.0 / 255.0],
      [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0, 0.85],
      [1.0, 59.0 / 255.0, 48.0 / 255.0, 0.18],
    )
  } else {
    (
      [1.0, 41.0 / 255.0, 112.0 / 255.0],
      [1.0, 105.0 / 255.0, 97.0 / 255.0],
      [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.55],
      [1.0, 69.0 / 255.0, 58.0 / 255.0, 0.30],
    )
  };
  OcrPalette {
    primary_fill: [primary[0], primary[1], primary[2], 0.30],
    primary_outline: [primary[0], primary[1], primary[2], 0.65],
    qr_fill: [primary[0], primary[1], primary[2], 0.50],
    qr_outline: [primary[0], primary[1], primary[2], 0.95],
    error_fill: [error[0], error[1], error[2], 0.20],
    error_outline: [error[0], error[1], error[2], 0.80],
    selection_fill: [primary[0], primary[1], primary[2], 0.45],
    selection_outline: [primary[0], primary[1], primary[2], 0.95],
    loading_fill: loading,
    loading_foreground: [1.0; 4],
    status_error_fill: status_error,
    status_error_foreground: [error[0], error[1], error[2], 1.0],
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_ocr_palette(light_mode: u32) -> OcrPalette {
  ocr_palette(light_mode != 0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn capture_chrome_is_white_over_a_dark_hairline_in_both_appearances() {
    let outline = [NEUTRAL_800, NEUTRAL_800, NEUTRAL_800, 1.0];
    for light_appearance in [true, false] {
      let palette = control_palette(light_appearance);
      assert_eq!(palette.fill, [WHITE; 4]);
      assert_eq!(palette.outline, outline);
    }
  }

  #[test]
  fn every_capture_tool_uses_the_shared_overlay_shade() {
    assert_eq!(overlay_palette().shade, [0.0, 0.0, 0.0, 0.48]);
  }

  #[test]
  fn ocr_palette_matches_the_primary_loading_and_error_tokens() {
    let light = ocr_palette(true);
    assert_eq!(
      light.primary_outline,
      [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0, 0.65]
    );
    assert_eq!(
      light.loading_fill,
      [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0, 0.85]
    );
    assert_eq!(light.error_fill, [215.0 / 255.0, 0.0, 21.0 / 255.0, 0.20]);
    assert_eq!(light.primary_fill[3], 0.30);
    assert_eq!(light.qr_fill[3], 0.50);
    assert_eq!(light.qr_outline[3], 0.95);

    let dark = ocr_palette(false);
    assert_eq!(dark.primary_fill, [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.30]);
    assert_eq!(dark.loading_fill, [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.55]);
    assert_eq!(dark.error_outline, [1.0, 105.0 / 255.0, 97.0 / 255.0, 0.80]);
    assert_eq!(dark.qr_fill, [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.50]);
    assert_eq!(dark.qr_outline[3], 0.95);
  }

  #[test]
  fn ruler_crosshair_follows_the_system_accent_and_the_info_token() {
    let [red, green, blue] = crate::system_accent::accent_rgb();
    let light = ruler_palette(true);
    let dark = ruler_palette(false);
    assert_eq!(light.primary, [red, green, blue, 1.0]);
    assert_eq!(dark.primary, [red, green, blue, 1.0]);
    assert_eq!(light.info, [0.0, 136.0 / 255.0, 1.0, 1.0]);
    assert_eq!(dark.info, [0.0, 145.0 / 255.0, 1.0, 1.0]);
  }
}
