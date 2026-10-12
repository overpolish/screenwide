// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
  Light = 0,
  Dark = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlKind {
  Button = 0,
  IconButton = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlColor {
  Neutral = 0,
  Primary = 1,
  Error = 2,
}

/// One control size, matching the regular AppKit control height. Native OSC
/// chrome has no second size, so this exists to keep the metric lookup and the
/// C ABI explicit rather than to offer a choice.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlSize {
  Regular = 0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interaction {
  Normal,
  Hovered,
  Pressed,
  Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlStyle {
  pub kind: ControlKind,
  pub color: ControlColor,
  pub size: ControlSize,
  pub disabled: bool,
}

impl ControlStyle {
  pub const fn button(color: ControlColor, size: ControlSize) -> Self {
    Self {
      kind: ControlKind::Button,
      color,
      size,
      disabled: false,
    }
  }

  pub const fn icon_button(color: ControlColor, size: ControlSize) -> Self {
    Self {
      kind: ControlKind::IconButton,
      color,
      size,
      disabled: false,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlMetrics {
  pub height: f64,
  pub radius: f64,
  pub padding_x: f64,
  pub gap: f64,
  pub icon_size: f64,
  /// A control label: the body role, `--text-body`.
  pub font_size: f64,
  pub line_height: f64,
  /// A numeric readout assembled from the glyph atlas, such as the ruler's
  /// callout and its measurement labels: the subheadline role,
  /// `--text-subheadline`. Secondary to the label beside it, and the tier the
  /// ruler was measured at on screen.
  pub readout_font_size: f64,
  pub readout_line_height: f64,
  /// The ruler callout is a `ToggleMenuButton` at its capture size: a glyph
  /// slot beside two stacked readout lines, so it is taller and more rounded
  /// than the one-row control it otherwise shares its metrics with.
  pub callout_height: f64,
  pub callout_radius: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlSpacing {
  pub tight: f64,
  pub control: f64,
  pub control_inset: f64,
  pub section: f64,
  /// Separation between major layout areas, `--spacing-layout`.
  pub layout: f64,
  /// Inset from a window edge to its content, `--spacing-window-inset`.
  pub window_inset: f64,
}

pub const fn control_spacing() -> ControlSpacing {
  ControlSpacing {
    tight: 2.0,
    control: 4.0,
    control_inset: 8.0,
    section: 12.0,
    layout: 24.0,
    window_inset: 14.0,
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_control_spacing() -> ControlSpacing {
  control_spacing()
}

/// Whether the Windows skin's values apply. The same tokens the page reads
/// from `src/index.css` under `[data-platform="windows"]`: Fluent's 32px
/// control on a 4px corner and body text at 14/20, with the taller callout
/// keeping the control corner, as `--radius-capture` does. The readouts keep
/// the label's 14px size on a tighter 18px line, so the callout's two lines
/// still sit inside its 40px with a little room, as they do on macOS.
pub(crate) const WINDOWS_SKIN: bool = cfg!(target_os = "windows");

pub const fn control_metrics(kind: ControlKind, size: ControlSize) -> ControlMetrics {
  let ControlSize::Regular = size;
  if WINDOWS_SKIN {
    return match kind {
      ControlKind::Button => ControlMetrics {
        height: 32.0,
        radius: 4.0,
        padding_x: 12.0,
        gap: 4.0,
        icon_size: 16.0,
        font_size: 14.0,
        line_height: 20.0,
        readout_font_size: 14.0,
        readout_line_height: 18.0,
        callout_height: 40.0,
        callout_radius: 4.0,
      },
      ControlKind::IconButton => ControlMetrics {
        height: 32.0,
        radius: 4.0,
        padding_x: 8.0,
        gap: 0.0,
        icon_size: 16.0,
        font_size: 0.0,
        line_height: 20.0,
        readout_font_size: 14.0,
        readout_line_height: 18.0,
        callout_height: 40.0,
        callout_radius: 4.0,
      },
    };
  }
  match kind {
    ControlKind::Button => ControlMetrics {
      height: 24.0,
      radius: 8.0,
      padding_x: 12.0,
      gap: 4.0,
      icon_size: 16.0,
      font_size: 13.0,
      line_height: 16.0,
      readout_font_size: 11.0,
      readout_line_height: 14.0,
      callout_height: 40.0,
      callout_radius: 11.0,
    },
    ControlKind::IconButton => ControlMetrics {
      height: 24.0,
      radius: 8.0,
      padding_x: 4.0,
      gap: 0.0,
      icon_size: 16.0,
      font_size: 0.0,
      line_height: 16.0,
      readout_font_size: 11.0,
      readout_line_height: 14.0,
      callout_height: 40.0,
      callout_radius: 11.0,
    },
  }
}
