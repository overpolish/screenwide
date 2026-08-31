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

/// Shared desktop shade for every capture-style OSC. Tools vary their chrome,
/// but the indication that the desktop is under an active overlay remains
/// identical across Region, Quick Screenshot, OCR, and future tools.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayPalette {
  pub shade: [f32; 4],
}

/// Shared ruler crosshair token. The callout itself resolves through the OSC
/// compact neutral control tokens so it cannot drift from native buttons.
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

pub const fn ruler_palette(light_appearance: bool) -> RulerPalette {
  if light_appearance {
    RulerPalette {
      primary: [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0, 1.0],
      info: [0.0, 104.0 / 255.0, 201.0 / 255.0, 1.0],
    }
  } else {
    RulerPalette {
      primary: [1.0, 41.0 / 255.0, 112.0 / 255.0, 1.0],
      info: [102.0 / 255.0, 183.0 / 255.0, 1.0, 1.0],
    }
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
  fn controls_follow_the_inverse_tooltip_tokens() {
    let light = control_palette(true);
    assert_eq!(light.fill, [NEUTRAL_800, NEUTRAL_800, NEUTRAL_800, 1.0]);
    assert_eq!(light.outline, [WHITE; 4]);

    let dark = control_palette(false);
    assert_eq!(dark.fill, [WHITE; 4]);
    assert_eq!(dark.outline, [NEUTRAL_800, NEUTRAL_800, NEUTRAL_800, 1.0]);
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
  fn ruler_crosshair_uses_the_shared_primary_accent() {
    let light = ruler_palette(true);
    assert_eq!(
      light.primary,
      [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0, 1.0]
    );
    let dark = ruler_palette(false);
    assert_eq!(dark.primary, [1.0, 41.0 / 255.0, 112.0 / 255.0, 1.0]);
    assert_eq!(light.info, [0.0, 104.0 / 255.0, 201.0 / 255.0, 1.0]);
    assert_eq!(dark.info, [102.0 / 255.0, 183.0 / 255.0, 1.0, 1.0]);
  }
}
