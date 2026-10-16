// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A shadow a caption's words cast: the outline's shape, the type and the
//! band around it, set off in a direction by a distance and blurred by a
//! softness, both in ems of the word's type. Soft, it is a drop shadow; hard
//! and short, the outline offset to one side.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::style::{clamped, rgba};

/// The farthest a shadow falls and the softest it is, in ems.
pub const MAX_SHADOW_DISTANCE: f64 = 0.5;
pub const MAX_SHADOW_SOFTNESS: f64 = 0.5;

/// Black, a little see-through, as a shadow most often is.
const SHADOW_COLOR: &str = "#00000099";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
#[ts(export)]
pub struct TextShadow {
  /// As `#rrggbb` or `#rrggbbaa`.
  pub color: String,
  /// Which way it falls, in degrees clockwise from straight up: 180 is
  /// straight down.
  pub angle: f64,
  /// How far it falls, in ems of the word's type.
  pub distance: f64,
  /// How far its edge spreads, in ems: none for a hard edge.
  pub softness: f64,
}

impl Default for TextShadow {
  fn default() -> Self {
    Self {
      color: SHADOW_COLOR.to_owned(),
      angle: 180.0,
      distance: 0.08,
      softness: 0.12,
    }
  }
}

impl TextShadow {
  /// The shadow with every number in its range, its angle within a turn,
  /// and its colour readable.
  pub(crate) fn normalized(&self) -> Self {
    Self {
      color: if rgba(&self.color).is_some() {
        self.color.clone()
      } else {
        SHADOW_COLOR.to_owned()
      },
      angle: if self.angle.is_finite() {
        self.angle.rem_euclid(360.0)
      } else {
        180.0
      },
      distance: clamped(self.distance, (0.0, MAX_SHADOW_DISTANCE), 0.0),
      softness: clamped(self.softness, (0.0, MAX_SHADOW_SOFTNESS), 0.0),
    }
  }

  /// How far it falls across and down, in ems.
  pub fn offset(&self) -> [f64; 2] {
    let turn = self.angle.to_radians();
    [turn.sin() * self.distance, -turn.cos() * self.distance]
  }
}
