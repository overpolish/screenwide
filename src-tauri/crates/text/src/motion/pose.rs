// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A motion sampled at one moment: every property's value, read off its
//! track's keyframes.

use super::{TextKeyframe, TextMotionTrack, TextProperty};

/// Every property's value at one moment of a motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Pose([f64; TextProperty::ALL.len()]);

impl Pose {
  pub(crate) fn get(&self, property: TextProperty) -> f64 {
    self.0[property.index()]
  }

  /// This pose with `other` moving the piece too.
  pub(crate) fn stacked(self, other: Self) -> Self {
    Self(
      TextProperty::ALL.map(|property| property.stacked(self.get(property), other.get(property))),
    )
  }
}

/// A track's value `elapsed_ms` into its motion.
fn value_at(keyframes: &[TextKeyframe], rest: f64, elapsed_ms: f64) -> f64 {
  let Some(first) = keyframes.first() else {
    return rest;
  };
  let next = keyframes.partition_point(|keyframe| keyframe.at_ms <= elapsed_ms);
  match (
    next.checked_sub(1).map(|at| &keyframes[at]),
    keyframes.get(next),
  ) {
    (None, _) => first.value,
    (Some(last), None) => last.value,
    (Some(from), Some(to)) => {
      let progress = (elapsed_ms - from.at_ms) / (to.at_ms - from.at_ms);
      from.value + (to.value - from.value) * to.easing.eased(progress)
    }
  }
}

/// Which of a caption's offsets turn round where tracks measure them from
/// its edge: across for one at the right, down for one along the top.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Mirror {
  pub(crate) across: bool,
  pub(crate) down: bool,
}

/// Every property's value `elapsed_ms` into `tracks`, a property no track
/// moves at rest, and an offset or a flip measured from the caption's edge
/// turned round as `mirror` says: a flip about the horizontal axis with
/// the caption's place down the picture, and about the vertical one with
/// its place across.
pub(crate) fn pose(tracks: &[TextMotionTrack], elapsed_ms: f64, mirror: Mirror) -> Pose {
  Pose(TextProperty::ALL.map(|property| {
    let rest = property.spec().rest;
    tracks
      .iter()
      .find(|track| track.property == property)
      .map_or(rest, |track| {
        let value = value_at(&track.keyframes, rest, elapsed_ms);
        let turned = track.from_edge
          && match property {
            TextProperty::OffsetX | TextProperty::RotateY => mirror.across,
            TextProperty::OffsetY | TextProperty::RotateX => mirror.down,
            TextProperty::Scale | TextProperty::Opacity | TextProperty::Rotate => false,
          };
        if turned {
          -value
        } else {
          value
        }
      })
  }))
}
