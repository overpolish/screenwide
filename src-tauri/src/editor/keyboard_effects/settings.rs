// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KeyboardAnimation {
  Pop,
  Fade,
  None,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KeyboardAppearance {
  Dark,
  Light,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeyboardEffectSettings {
  pub bake: bool,
  pub animation: KeyboardAnimation,
  pub appearance: KeyboardAppearance,
  pub size_percent: f64,
  #[serde(default)]
  pub position_x_percent: Option<f64>,
  #[serde(default)]
  pub position_y_percent: Option<f64>,
}

impl Default for KeyboardEffectSettings {
  fn default() -> Self {
    Self {
      bake: true,
      animation: KeyboardAnimation::Pop,
      appearance: KeyboardAppearance::Light,
      size_percent: 100.0,
      position_x_percent: None,
      position_y_percent: None,
    }
  }
}

impl KeyboardEffectSettings {
  pub(crate) fn normalized(self) -> Self {
    Self {
      size_percent: if self.size_percent.is_finite() {
        self.size_percent.clamp(5.0, 500.0)
      } else {
        100.0
      },
      position_x_percent: self
        .position_x_percent
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.0, 100.0)),
      position_y_percent: self
        .position_y_percent
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.0, 100.0)),
      ..self
    }
  }
}
