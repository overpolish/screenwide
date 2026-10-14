// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where an overlay drawn over the picture sits, and how large it is drawn.
//! Captions and shortcuts offer the same nine places rather than a free
//! position, so every recording keeps them clear of the edges the same way;
//! shortcuts and the cursor offer the same few sizes rather than a free
//! percentage, so a recording looks right without weighing 140% against
//! 150%.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One of nine places on the picture: along the top, through the middle or
/// along the bottom, at the left, the centre or the right. The picture's own
/// sides, whatever the language's direction.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Placement {
  TopLeft,
  Top,
  TopRight,
  Left,
  Center,
  Right,
  BottomLeft,
  #[default]
  Bottom,
  BottomRight,
}

impl Placement {
  /// How far across and down the picture the overlay is held, each from 0
  /// at the left or top margin to 1 at the right or bottom one. Across, it
  /// is also how the overlay's contents line up: from their left, about
  /// their middle, or from their right.
  pub fn shares(self) -> [f64; 2] {
    match self {
      Self::TopLeft => [0.0, 0.0],
      Self::Top => [0.5, 0.0],
      Self::TopRight => [1.0, 0.0],
      Self::Left => [0.0, 0.5],
      Self::Center => [0.5, 0.5],
      Self::Right => [1.0, 0.5],
      Self::BottomLeft => [0.0, 1.0],
      Self::Bottom => [0.5, 1.0],
      Self::BottomRight => [1.0, 1.0],
    }
  }
}

/// How large an overlay is drawn: the shortcuts or the cursor.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum OverlaySize {
  #[default]
  Standard,
  Large,
  ExtraLarge,
  Huge,
  Giant,
}

impl OverlaySize {
  /// How many times its own size the overlay is drawn at: the size it was
  /// recorded at, or designed at, then 150%, 200%, 300% and 500%.
  pub fn scale(self) -> f64 {
    match self {
      Self::Standard => 1.0,
      Self::Large => 1.5,
      Self::ExtraLarge => 2.0,
      Self::Huge => 3.0,
      Self::Giant => 5.0,
    }
  }
}
