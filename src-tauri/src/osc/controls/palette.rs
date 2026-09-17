// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The semantic colours a control resolves to: the fill ladder, the label
//! tiers, and the accent, each measured for both appearances. `style` owns
//! what a control is and how large it is; this owns what it looks like.

use super::style::{Appearance, ControlColor, ControlStyle, Interaction, WINDOWS_SKIN};

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
