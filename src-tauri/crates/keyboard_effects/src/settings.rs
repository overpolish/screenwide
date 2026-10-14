// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_placement::{OverlaySize, Placement};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, rename = "KeyboardEffectAnimation")]
pub enum KeyboardAnimation {
  Pop,
  Fade,
  None,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, rename = "KeyboardEffectAppearance")]
pub enum KeyboardAppearance {
  Dark,
  Light,
}

/// How every shortcut in the recording is drawn. There is one size and one
/// place for all of them: a shortcut is never placed on its own.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct KeyboardEffectSettings {
  pub bake: bool,
  pub animation: KeyboardAnimation,
  pub appearance: KeyboardAppearance,
  pub size: OverlaySize,
  pub position: Placement,
}

impl Default for KeyboardEffectSettings {
  fn default() -> Self {
    Self {
      bake: true,
      animation: KeyboardAnimation::Pop,
      appearance: KeyboardAppearance::Light,
      size: OverlaySize::Standard,
      position: Placement::Bottom,
    }
  }
}
