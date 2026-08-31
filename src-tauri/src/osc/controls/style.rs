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

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlSize {
  Compact = 0,
  Default = 1,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlMetrics {
  pub height: f64,
  pub radius: f64,
  pub padding_x: f64,
  pub gap: f64,
  pub icon_size: f64,
  pub font_size: f64,
  pub line_height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlSpacing {
  pub tight: f64,
  pub control: f64,
  pub control_inset: f64,
  pub section: f64,
  pub window_inset: f64,
}

pub const fn control_spacing() -> ControlSpacing {
  ControlSpacing {
    tight: 2.0,
    control: 4.0,
    control_inset: 8.0,
    section: 12.0,
    window_inset: 24.0,
  }
}

#[no_mangle]
pub extern "C" fn screenwide_osc_control_spacing() -> ControlSpacing {
  control_spacing()
}

pub const fn control_metrics(kind: ControlKind, size: ControlSize) -> ControlMetrics {
  match (kind, size) {
    (ControlKind::Button, ControlSize::Compact) => ControlMetrics {
      height: 24.0,
      radius: 8.0,
      padding_x: 8.0,
      gap: 8.0,
      icon_size: 14.0,
      font_size: 12.0,
      line_height: 16.0,
    },
    (ControlKind::Button, ControlSize::Default) => ControlMetrics {
      height: 36.0,
      radius: 12.0,
      padding_x: 12.0,
      gap: 8.0,
      icon_size: 18.0,
      font_size: 14.0,
      line_height: 20.0,
    },
    (ControlKind::IconButton, ControlSize::Compact) => ControlMetrics {
      height: 24.0,
      radius: 8.0,
      padding_x: 4.0,
      gap: 0.0,
      icon_size: 14.0,
      font_size: 0.0,
      line_height: 16.0,
    },
    (ControlKind::IconButton, ControlSize::Default) => ControlMetrics {
      height: 36.0,
      radius: 12.0,
      padding_x: 6.0,
      gap: 0.0,
      icon_size: 18.0,
      font_size: 0.0,
      line_height: 24.0,
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

const CONTENT_DARK: [f32; 4] = [38.0 / 255.0, 38.0 / 255.0, 38.0 / 255.0, 1.0];
const WHITE: [f32; 4] = [1.0; 4];
const DISABLED_LIGHT: [f32; 4] = [168.0 / 255.0, 168.0 / 255.0, 168.0 / 255.0, 1.0];
const DISABLED_DARK: [f32; 4] = [119.0 / 255.0, 119.0 / 255.0, 119.0 / 255.0, 1.0];
const ERROR_LIGHT: [f32; 4] = [215.0 / 255.0, 0.0, 21.0 / 255.0, 1.0];
const ERROR_DARK: [f32; 4] = [1.0, 105.0 / 255.0, 97.0 / 255.0, 1.0];

const fn neutral_fill(appearance: Appearance, interaction: Interaction) -> [f32; 4] {
  match appearance {
    Appearance::Light => match interaction {
      Interaction::Normal => [0.0, 0.0, 0.0, 0.09],
      Interaction::Hovered => [0.0, 0.0, 0.0, 0.13],
      Interaction::Pressed => [0.0, 0.0, 0.0, 0.17],
      Interaction::Disabled => [0.0, 0.0, 0.0, 0.05],
    },
    Appearance::Dark => match interaction {
      Interaction::Normal => [1.0, 1.0, 1.0, 0.12],
      Interaction::Hovered => [1.0, 1.0, 1.0, 0.17],
      Interaction::Pressed => [1.0, 1.0, 1.0, 0.22],
      Interaction::Disabled => [1.0, 1.0, 1.0, 0.07],
    },
  }
}

const fn primary_fill(appearance: Appearance, interaction: Interaction) -> [f32; 4] {
  match appearance {
    Appearance::Light => match interaction {
      Interaction::Normal => [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0, 0.85],
      Interaction::Hovered => [194.0 / 255.0, 24.0 / 255.0, 91.0 / 255.0, 0.90],
      Interaction::Pressed => [173.0 / 255.0, 20.0 / 255.0, 87.0 / 255.0, 0.95],
      Interaction::Disabled => [0.0, 0.0, 0.0, 0.05],
    },
    Appearance::Dark => match interaction {
      Interaction::Normal => [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.55],
      Interaction::Hovered => [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.65],
      Interaction::Pressed => [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.75],
      Interaction::Disabled => [1.0, 1.0, 1.0, 0.07],
    },
  }
}

pub fn control_visual(
  style: ControlStyle,
  interaction: Interaction,
  appearance: Appearance,
) -> ControlVisual {
  let foreground = if interaction == Interaction::Disabled {
    match appearance {
      Appearance::Light => DISABLED_LIGHT,
      Appearance::Dark => DISABLED_DARK,
    }
  } else {
    match (style.color, appearance) {
      (ControlColor::Primary, _) => WHITE,
      (ControlColor::Error, Appearance::Light) => ERROR_LIGHT,
      (ControlColor::Error, Appearance::Dark) => ERROR_DARK,
      (ControlColor::Neutral, Appearance::Light) => CONTENT_DARK,
      (ControlColor::Neutral, Appearance::Dark) => WHITE,
    }
  };
  let fill = match style.color {
    ControlColor::Neutral => neutral_fill(appearance, interaction),
    ControlColor::Primary => primary_fill(appearance, interaction),
    ControlColor::Error => neutral_fill(appearance, interaction),
  };
  ControlVisual { fill, foreground }
}
