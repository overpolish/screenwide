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

/// Shared ruler line token: the emphasis colour for what the Ruler measures
/// and `--color-info` for guides. The labels and the callout draw in the
/// island's black (`RULER_LABEL_FILLS`), so nothing here tints them.
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

/// A status token from `src/styles/colors.css` in sRGB, light then dark:
/// the OKLCH values resolved once, since the shader has no colour space to
/// mix them in.
const fn token(light_appearance: bool, light: [u8; 3], dark: [u8; 3]) -> [f32; 4] {
  let [red, green, blue] = if light_appearance { light } else { dark };
  [
    red as f32 / 255.0,
    green as f32 / 255.0,
    blue as f32 / 255.0,
    1.0,
  ]
}

/// `--color-info`: `oklch(58% 0.17 250)` light, `oklch(74% 0.13 250)` dark.
const fn info(light_appearance: bool) -> [f32; 4] {
  token(light_appearance, [0, 124, 217], [103, 176, 249])
}

/// `--color-success`: `oklch(60% 0.16 150)` light, `oklch(76% 0.15 150)` dark.
const fn success(light_appearance: bool) -> [f32; 4] {
  token(light_appearance, [19, 153, 72], [97, 203, 124])
}

/// `--color-error`: `oklch(58% 0.21 25)` light, `oklch(70% 0.19 25)` dark.
const fn error(light_appearance: bool) -> [f32; 4] {
  token(light_appearance, [219, 43, 51], [255, 100, 95])
}

pub fn ruler_palette(light_appearance: bool) -> RulerPalette {
  let [red, green, blue] = screenwide_accent::accent_rgb();
  RulerPalette {
    primary: [red, green, blue, 1.0],
    info: info(light_appearance),
  }
}

/// The Ruler's labels and callout: the island's black (`--color-island`)
/// with the dark appearance's ink on it (`--color-fg`, `oklch(98% 0.005 47.6)`
/// at 88%), in either appearance, so they read as the island does over
/// whatever the desktop shows. Like the island they carry no outline; their
/// shadow is their edge.
pub const RULER_LABEL_FILLS: [[f32; 4]; 2] = [
  [0.0, 0.0, 0.0, 1.0],
  [251.0 / 255.0, 247.0 / 255.0, 246.0 / 255.0, 0.88],
];

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
}

const _: () = assert!(std::mem::size_of::<OcrPalette>() == 128);

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

/// OCR highlights in `--color-emphasis`, the colour of where you are, and
/// unreadable QR codes in `--color-error`. Recognised text is a faint wash
/// and selected text a stronger one, as text selection is, neither with a
/// border: its outline is its fill. QR codes, which a click opens, carry an
/// outline, as does the area read.
pub fn ocr_palette(light_appearance: bool) -> OcrPalette {
  let [red, green, blue] = screenwide_accent::accent_rgb();
  let [error_red, error_green, error_blue, _] = error(light_appearance);
  let line = [red, green, blue, 0.14];
  let selection = [red, green, blue, 0.40];
  OcrPalette {
    primary_fill: line,
    primary_outline: line,
    qr_fill: [red, green, blue, 0.16],
    qr_outline: [red, green, blue, 0.9],
    error_fill: [error_red, error_green, error_blue, 0.16],
    error_outline: [error_red, error_green, error_blue, 0.9],
    selection_fill: selection,
    selection_outline: selection,
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_ocr_palette(light_mode: u32) -> OcrPalette {
  ocr_palette(light_mode != 0)
}

/// An object picked in the camera, kept with the person in
/// `--color-success` and removed with the room in `--color-error`, per
/// appearance.
pub fn picked_palette(light_appearance: bool) -> [[f32; 4]; 2] {
  [success(light_appearance), error(light_appearance)]
}

/// The rotation gizmo's rings, in the colours every 3D tool gives its axes: X
/// in `--color-error`'s red, Y in `--color-success`'s green and Z in
/// `--color-info`'s blue, per appearance.
pub fn turn_palette(light_appearance: bool) -> [[f32; 4]; 3] {
  [
    error(light_appearance),
    success(light_appearance),
    info(light_appearance),
  ]
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
  fn ocr_text_is_tinted_without_a_border_and_only_qr_codes_are_outlined() {
    let [red, green, blue] = screenwide_accent::accent_rgb();
    for light_appearance in [true, false] {
      let palette = ocr_palette(light_appearance);
      assert_eq!(palette.primary_fill, palette.primary_outline);
      assert_eq!(palette.selection_fill, palette.selection_outline);
      assert!(palette.selection_fill[3] > palette.primary_fill[3]);
      assert_eq!(palette.qr_outline, [red, green, blue, 0.9]);
      let [red, green, blue, _] = error(light_appearance);
      assert_eq!(palette.error_outline, [red, green, blue, 0.9]);
    }
  }
}
