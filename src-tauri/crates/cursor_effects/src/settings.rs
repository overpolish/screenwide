// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_placement::OverlaySize;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CursorEffectSettings {
  pub bake: bool,
  pub click_animation: bool,
  #[serde(default)]
  pub clip_at_video_edge: bool,
  pub motion_blur: bool,
  pub smooth_movement: bool,
  #[serde(default)]
  pub size: OverlaySize,
}

impl Default for CursorEffectSettings {
  fn default() -> Self {
    Self {
      bake: true,
      click_animation: true,
      clip_at_video_edge: false,
      motion_blur: true,
      smooth_movement: true,
      size: OverlaySize::Standard,
    }
  }
}
