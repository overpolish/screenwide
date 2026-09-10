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
/// `--alpha-content-fg`, the label tier every OSC readout is drawn at.
const CONTENT_ALPHA: f32 = 0.85;
/// A status surface that leaves its material backing untinted.
const TRANSPARENT: [f32; 4] = [0.0; 4];

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

/// OCR highlights follow the system accent, the way every other OSC accent
/// surface does, and the error tier is `--color-error` per appearance. The
/// status pill no longer carries a coloured fill: its material surface is the
/// backing, so the loading and error fills are transparent and only the two
/// foregrounds resolve to a colour.
pub fn ocr_palette(light_appearance: bool) -> OcrPalette {
  let [red, green, blue] = crate::system_accent::accent_rgb();
  let error = if light_appearance {
    [1.0, 56.0 / 255.0, 60.0 / 255.0]
  } else {
    [1.0, 66.0 / 255.0, 69.0 / 255.0]
  };
  // The label tier from `src/index.css`: pure black in light, pure white in
  // dark, both at 85%.
  let label = if light_appearance { 0.0 } else { 1.0 };
  OcrPalette {
    primary_fill: [red, green, blue, 0.15],
    primary_outline: [red, green, blue, 0.50],
    qr_fill: [red, green, blue, 0.25],
    qr_outline: [red, green, blue, 0.80],
    error_fill: [error[0], error[1], error[2], 0.20],
    error_outline: [error[0], error[1], error[2], 0.80],
    selection_fill: [red, green, blue, 0.45],
    selection_outline: [red, green, blue, 0.95],
    loading_fill: TRANSPARENT,
    loading_foreground: [label, label, label, CONTENT_ALPHA],
    status_error_fill: TRANSPARENT,
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
  fn ocr_highlights_follow_the_accent_and_the_status_pill_carries_no_fill() {
    let [red, green, blue] = crate::system_accent::accent_rgb();
    for light_appearance in [true, false] {
      let palette = ocr_palette(light_appearance);
      assert_eq!(palette.primary_fill, [red, green, blue, 0.15]);
      assert_eq!(palette.primary_outline, [red, green, blue, 0.50]);
      assert_eq!(palette.qr_fill, [red, green, blue, 0.25]);
      assert_eq!(palette.qr_outline, [red, green, blue, 0.80]);
      assert_eq!(palette.selection_fill, [red, green, blue, 0.45]);
      assert_eq!(palette.selection_outline, [red, green, blue, 0.95]);
      // The pill is a bare material surface, so neither status state tints it.
      assert_eq!(palette.loading_fill, [0.0; 4]);
      assert_eq!(palette.status_error_fill, [0.0; 4]);
    }

    let light = ocr_palette(true);
    assert_eq!(light.error_fill, [1.0, 56.0 / 255.0, 60.0 / 255.0, 0.20]);
    assert_eq!(light.error_outline, [1.0, 56.0 / 255.0, 60.0 / 255.0, 0.80]);
    assert_eq!(light.loading_foreground, [0.0, 0.0, 0.0, 0.85]);
    assert_eq!(
      light.status_error_foreground,
      [1.0, 56.0 / 255.0, 60.0 / 255.0, 1.0]
    );

    let dark = ocr_palette(false);
    assert_eq!(dark.error_fill, [1.0, 66.0 / 255.0, 69.0 / 255.0, 0.20]);
    assert_eq!(dark.error_outline, [1.0, 66.0 / 255.0, 69.0 / 255.0, 0.80]);
    assert_eq!(dark.loading_foreground, [1.0, 1.0, 1.0, 0.85]);
    assert_eq!(
      dark.status_error_foreground,
      [1.0, 66.0 / 255.0, 69.0 / 255.0, 1.0]
    );
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
