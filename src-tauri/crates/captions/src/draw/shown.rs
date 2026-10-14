// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a piece is drawn at one moment: moved by the entrance it came on by,
//! and, once its caption starts going off, by the exits it goes off by too.

use crate::motion::{pose, CaptionMotionTrack, CaptionProperty, Mirror, Pose};

/// How a piece is drawn, now and as the frame's exposure began, which the
/// compositor blurs between: how large about its centre, how solid, and how
/// far it is moved, in ems of its type.
#[derive(Clone, Copy)]
pub(super) struct Shown {
  pub(super) scale: [f64; 2],
  pub(super) opacity: [f64; 2],
  pub(super) offset: [[f64; 2]; 2],
}

/// What moves a piece: the tracks it came on by, if any, and how long ago
/// they started; and the tracks it goes off by, each with how long ago its
/// exit started, less than nothing before then: a rolling word goes off
/// before the caption does.
#[derive(Clone, Copy)]
pub(super) struct Moving<'a> {
  pub(super) entrance: Option<(&'a [CaptionMotionTrack], f64)>,
  pub(super) exits: [(&'a [CaptionMotionTrack], f64); 2],
}

impl Moving<'_> {
  /// Every property's value `back_ms` ago. An exit moves nothing before it
  /// starts, though its tracks hold their first keyframe's value then.
  fn pose(&self, back_ms: f64, mirror: Mirror) -> Pose {
    let mut posed = match self.entrance {
      Some((tracks, elapsed_ms)) => pose(tracks, elapsed_ms - back_ms, mirror),
      None => pose(&[], 0.0, mirror),
    };
    for (tracks, elapsed_ms) in self.exits {
      let exit_ms = elapsed_ms - back_ms;
      if exit_ms >= 0.0 {
        posed = posed.stacked(pose(tracks, exit_ms, mirror));
      }
    }
    posed
  }

  /// The piece as it is drawn now and `exposure_ms` before.
  pub(super) fn shown(&self, exposure_ms: f64, mirror: Mirror) -> Shown {
    let [now, before] = [0.0, exposure_ms.max(0.0)].map(|back_ms| self.pose(back_ms, mirror));
    let both = |property| [now.get(property), before.get(property)];
    let offset = |pose: Pose| {
      [
        pose.get(CaptionProperty::OffsetX),
        pose.get(CaptionProperty::OffsetY),
      ]
    };
    Shown {
      scale: both(CaptionProperty::Scale),
      opacity: both(CaptionProperty::Opacity),
      offset: [offset(now), offset(before)],
    }
  }
}
