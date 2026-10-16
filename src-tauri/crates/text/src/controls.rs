// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Control points: points a template declares, each a grip a text's own
//! hand moves and something one of the template's elements is drawn by.
//! The template says where a point starts and how far it may go; where it is
//! now is the text's own, kept by id, so the same text keeps its points when
//! it is restyled in a template that declares them too. Captions are placed
//! by their template rather than by hand, and draw no control points.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The longest an id is, so a hand-edited manifest stays readable.
const MAX_ID: usize = 40;
/// The furthest past its box a point may reach, in ems of the text's type.
pub const MAX_REACH: f64 = 200.0;

/// What a control point is held against once it has been moved.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum ControlSpace {
  /// The text's box: the point keeps its place on the box however the text
  /// is moved, retyped or resized.
  #[default]
  Box,
  /// The picture: once moved, the point stays on what it was put on when the
  /// text moves. Until then it follows its default against the box.
  Picture,
}

/// A point held against a text's box. `along` is where it sits against the
/// box in each axis, as a share of the box's half size from its centre: -1
/// at the left or top edge, 1 at the right or bottom. `reach` is how far past
/// that edge it goes in each axis, in ems of the text's type, and is zero in
/// an axis the point is inside the box in.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct BoxPoint {
  pub along: [f64; 2],
  pub reach: [f64; 2],
}

impl BoxPoint {
  /// Tucked into the middle of the bottom edge, where a tail is most often
  /// drawn out from.
  pub const TUCKED: Self = Self {
    along: [0.0, 1.0],
    reach: [0.0, 0.0],
  };

  /// Whether the point reaches past the box in either axis.
  pub fn is_out(&self) -> bool {
    self.reach.iter().any(|reach| *reach > 0.0)
  }

  /// The point with `along` on the box and `reach` within `limits`.
  pub fn within(&self, limits: &ControlLimits) -> Self {
    let finite = |value: f64, fallback: f64| if value.is_finite() { value } else { fallback };
    Self {
      along: self.along.map(|along| finite(along, 0.0).clamp(-1.0, 1.0)),
      reach: self
        .reach
        .map(|reach| finite(reach, 0.0).clamp(limits.reach[0], limits.reach[1])),
    }
  }
}

/// How far a control point may reach past its box, in ems: at least the
/// first, at most the second, in either axis.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct ControlLimits {
  pub reach: [f64; 2],
}

impl Default for ControlLimits {
  fn default() -> Self {
    Self {
      reach: [0.0, MAX_REACH],
    }
  }
}

impl ControlLimits {
  fn normalized(&self) -> Self {
    let [low, high] = self.reach.map(|reach| {
      if reach.is_finite() {
        reach.clamp(0.0, MAX_REACH)
      } else {
        0.0
      }
    });
    Self {
      reach: [low.min(high), high.max(low)],
    }
  }
}

/// A control point a template declares.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct TextControl {
  /// What the template's elements and a text's own points name it by.
  pub id: String,
  #[serde(default)]
  pub space: ControlSpace,
  /// Where the point starts, against the box.
  #[serde(default = "tucked")]
  pub default: BoxPoint,
  #[serde(default)]
  pub limits: ControlLimits,
}

fn tucked() -> BoxPoint {
  BoxPoint::TUCKED
}

impl TextControl {
  pub(crate) fn has_valid_id(&self) -> bool {
    !self.id.is_empty()
      && self.id.len() <= MAX_ID
      && self
        .id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-')
  }

  pub(crate) fn normalized(&self) -> Self {
    let limits = self.limits.normalized();
    Self {
      id: self.id.clone(),
      space: self.space,
      default: self.default.within(&limits),
      limits,
    }
  }
}
