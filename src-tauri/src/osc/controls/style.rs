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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ControlVisual {
  /// The exact CSS semantic fill. Renderers preserve its alpha so controls
  /// layer over native materials and captured content like their React peers.
  pub fill: [f32; 4],
  pub foreground: [f32; 4],
}

impl ControlVisual {
  pub fn mix(self, target: Self, amount: f32) -> Self {
    let mix = |from: [f32; 4], to: [f32; 4]| {
      std::array::from_fn(|index| from[index] + (to[index] - from[index]) * amount)
    };
    Self {
      fill: mix(self.fill, target.fill),
      foreground: mix(self.foreground, target.foreground),
    }
  }
}

/// Label ladder from `src/index.css`: pure black in light, pure white in
/// dark, carried at the tier's alpha. Tertiary is also the disabled tier.
const CONTENT_ALPHA: f32 = 0.85;
const DISABLED_ALPHA: f32 = 0.25;
const WHITE: [f32; 4] = [1.0; 4];

/// `--color-error`, the AppKit system red measured for each appearance.
const ERROR_LIGHT: [f32; 4] = [1.0, 56.0 / 255.0, 60.0 / 255.0, 1.0];
const ERROR_DARK: [f32; 4] = [1.0, 66.0 / 255.0, 69.0 / 255.0, 1.0];

/// The fill ladder is black in light and white in dark, so every neutral fill
/// and every label tier is one channel value with a tier alpha.
const fn content_channel(appearance: Appearance) -> f32 {
  match appearance {
    Appearance::Light => 0.0,
    Appearance::Dark => 1.0,
  }
}

const fn content_color(appearance: Appearance, alpha: f32) -> [f32; 4] {
  let channel = content_channel(appearance);
  [channel, channel, channel, alpha]
}

/// AppKit's system fills, `--color-fill` and friends: 10 / 8 / 5 / 3%. A
/// bezeled control does not react to hover, so hovered repeats the resting
/// fill; pressed is the resting fill with a second 8% layer over it, which
/// composites to 0.10 + 0.08 * (1 - 0.10).
///
/// On the Windows skin these are Fluent's control fills, `--color-control-fill`
/// and its states: white at 70% resting in light appearance, a near-white at
/// 50% under the pointer and 30% pressed; white at 6 / 8.4 / 3.3% in dark.
/// Disabled is the pressed tier in light and 4.2% white in dark.
const fn neutral_fill(appearance: Appearance, interaction: Interaction) -> [f32; 4] {
  if WINDOWS_SKIN {
    const NEAR_WHITE: f32 = 249.0 / 255.0;
    return match (appearance, interaction) {
      (Appearance::Light, Interaction::Normal) => [1.0, 1.0, 1.0, 0.70],
      (Appearance::Light, Interaction::Hovered) => [NEAR_WHITE, NEAR_WHITE, NEAR_WHITE, 0.50],
      (Appearance::Light, Interaction::Pressed | Interaction::Disabled) => {
        [NEAR_WHITE, NEAR_WHITE, NEAR_WHITE, 0.30]
      }
      (Appearance::Dark, Interaction::Normal) => [1.0, 1.0, 1.0, 0.06],
      (Appearance::Dark, Interaction::Hovered) => [1.0, 1.0, 1.0, 0.084],
      (Appearance::Dark, Interaction::Pressed) => [1.0, 1.0, 1.0, 0.033],
      (Appearance::Dark, Interaction::Disabled) => [1.0, 1.0, 1.0, 0.042],
    };
  }
  let alpha = match interaction {
    Interaction::Normal | Interaction::Hovered => 0.10,
    Interaction::Pressed => 0.172,
    Interaction::Disabled => 0.03,
  };
  content_color(appearance, alpha)
}

/// `--color-primary-surface` and its hover and pressed states: the accent
/// opaque, then mixed 10% and 20% toward black. On the Windows skin the fill
/// is the OS accent's tone for the appearance, hover and press are that fill
/// at 90% and 80%, and disabled is Fluent's accent-disabled fill.
fn primary_fill(appearance: Appearance, interaction: Interaction) -> [f32; 4] {
  if WINDOWS_SKIN {
    if interaction == Interaction::Disabled {
      return match appearance {
        Appearance::Light => [0.0, 0.0, 0.0, 0.216],
        Appearance::Dark => [1.0, 1.0, 1.0, 0.158],
      };
    }
    let alpha = match interaction {
      Interaction::Hovered => 0.90,
      Interaction::Pressed => 0.80,
      _ => 1.0,
    };
    let [red, green, blue] = crate::system_accent::accent_fill_rgb(appearance == Appearance::Light);
    return [red, green, blue, alpha];
  }
  if interaction == Interaction::Disabled {
    return neutral_fill(appearance, Interaction::Disabled);
  }
  let shade = match interaction {
    Interaction::Hovered => 0.90,
    Interaction::Pressed => 0.80,
    _ => 1.0,
  };
  let [red, green, blue] = crate::system_accent::accent_rgb();
  [red * shade, green * shade, blue * shade, 1.0]
}

/// The label's alpha at its tier: 85% and 25% disabled on macOS; on the
/// Windows skin Fluent's text at 89.6% black in light and pure white in dark,
/// and disabled text at 36%.
const fn content_alpha(appearance: Appearance, disabled: bool) -> f32 {
  if WINDOWS_SKIN {
    return match (appearance, disabled) {
      (Appearance::Light, false) => 0.896,
      (Appearance::Dark, false) => 1.0,
      (Appearance::Light, true) => 0.361,
      (Appearance::Dark, true) => 0.363,
    };
  }
  if disabled {
    DISABLED_ALPHA
  } else {
    CONTENT_ALPHA
  }
}

/// A bezel's hairline stroke, the web skin's `control-stroke`: Fluent's
/// control stroke on the Windows skin. macOS bezels have no stroke, and only
/// the Windows chrome draws it, as the plate's outline.
#[cfg(target_os = "windows")]
pub const fn control_stroke(appearance: Appearance) -> [f32; 4] {
  match appearance {
    Appearance::Light => [0.0, 0.0, 0.0, 0.058],
    Appearance::Dark => [1.0, 1.0, 1.0, 0.07],
  }
}

pub fn control_visual(
  style: ControlStyle,
  interaction: Interaction,
  appearance: Appearance,
) -> ControlVisual {
  let foreground = if interaction == Interaction::Disabled {
    content_color(appearance, content_alpha(appearance, true))
  } else {
    match (style.color, appearance) {
      (ControlColor::Primary, _) if WINDOWS_SKIN => {
        crate::system_accent::text_on_accent_fill(appearance == Appearance::Light)
      }
      (ControlColor::Primary, _) => WHITE,
      (ControlColor::Error, Appearance::Light) => ERROR_LIGHT,
      (ControlColor::Error, Appearance::Dark) => ERROR_DARK,
      (ControlColor::Neutral, _) => content_color(appearance, content_alpha(appearance, false)),
    }
  };
  let fill = match style.color {
    ControlColor::Neutral => neutral_fill(appearance, interaction),
    ControlColor::Primary => primary_fill(appearance, interaction),
    ControlColor::Error => neutral_fill(appearance, interaction),
  };
  ControlVisual { fill, foreground }
}
