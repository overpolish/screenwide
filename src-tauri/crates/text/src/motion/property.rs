// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a motion track can animate. Each property is one entry here: its
//! name in a template, the range a keyframe may set it in, and its value at
//! rest. Reading a template, keeping keyframes in range and sampling a track
//! all go by these, so a new property is a new entry, and whatever the
//! renderer must do to draw it.

use serde::{de, Deserialize, Deserializer, Serialize};
use ts_rs::TS;

/// A property of a caption's piece that a motion track animates.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TextProperty {
  /// How large it is drawn about its centre, 1 at its own size.
  Scale,
  /// How solid it is, from 0 unseen to 1.
  Opacity,
  /// How far it is moved across and down, in ems of its type.
  OffsetX,
  OffsetY,
  /// How far it is turned, in degrees: flat on the picture, clockwise, and
  /// about its horizontal and vertical axes, as a card flips, its top or its
  /// right edge going away at positive turns.
  Rotate,
  RotateX,
  RotateY,
}

pub(crate) struct PropertySpec {
  pub(crate) name: &'static str,
  pub(crate) range: (f64, f64),
  pub(crate) rest: f64,
}

impl TextProperty {
  pub(crate) const ALL: [Self; 7] = [
    Self::Scale,
    Self::Opacity,
    Self::OffsetX,
    Self::OffsetY,
    Self::Rotate,
    Self::RotateX,
    Self::RotateY,
  ];

  pub(crate) fn spec(self) -> PropertySpec {
    let (name, range, rest) = match self {
      Self::Scale => ("scale", (0.0, 4.0), 1.0),
      Self::Opacity => ("opacity", (0.0, 1.0), 1.0),
      Self::OffsetX => ("offsetX", (-2.0, 2.0), 0.0),
      Self::OffsetY => ("offsetY", (-2.0, 2.0), 0.0),
      Self::Rotate => ("rotate", (-720.0, 720.0), 0.0),
      Self::RotateX => ("rotateX", (-720.0, 720.0), 0.0),
      Self::RotateY => ("rotateY", (-720.0, 720.0), 0.0),
    };
    PropertySpec { name, range, rest }
  }

  /// Its place among [`Self::ALL`], which a pose keeps its values by.
  pub(crate) fn index(self) -> usize {
    self as usize
  }

  /// `a` and `b` together, as two motions moving one piece at once, such
  /// as the caption's going off and its word's: sizes and how solid it is
  /// multiply, offsets and turns add, so either at rest leaves the other as
  /// it is.
  pub(crate) fn stacked(self, a: f64, b: f64) -> f64 {
    match self {
      Self::Scale | Self::Opacity => a * b,
      Self::OffsetX | Self::OffsetY | Self::Rotate | Self::RotateX | Self::RotateY => a + b,
    }
  }
}

impl<'de> Deserialize<'de> for TextProperty {
  /// A property this app can draw, by name. Any other is turned away by
  /// name, so a template made for a later version says what it needs
  /// rather than being drawn without it.
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let name = String::deserialize(deserializer)?;
    Self::ALL
      .into_iter()
      .find(|property| property.spec().name == name)
      .ok_or_else(|| {
        de::Error::custom(format!(
          "it animates `{name}`, which this version of Screenwide can't draw"
        ))
      })
  }
}
