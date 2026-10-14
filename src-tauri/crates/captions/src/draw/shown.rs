// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a piece is drawn at one moment: moved by the entrance it came on by,
//! and, once its caption's time is over, by the exits it goes off by too.

use super::FRAME_MS;
use crate::motion::{pose, CaptionMotionTrack, CaptionProperty, Mirror, Pose};

/// How a piece is drawn at one moment of its motion: how large about its
/// centre and how solid, now and a frame ago, which the compositor blurs
/// between, and how far it is moved, in ems of its type.
#[derive(Clone, Copy)]
pub(super) struct Shown {
  pub(super) scale: [f64; 2],
  pub(super) opacity: [f64; 2],
  pub(super) offset: [f64; 2],
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

  /// The piece as it is drawn now. A growing piece stays where it settles,
  /// and the size a frame ago is the size now, so growth is drawn sharp
  /// rather than blurred as an annotation's is; how solid it is blurs
  /// across the frame.
  pub(super) fn shown(&self, mirror: Mirror) -> Shown {
    let now = self.pose(0.0, mirror);
    let before = self.pose(FRAME_MS, mirror);
    Shown {
      scale: [now.get(CaptionProperty::Scale); 2],
      opacity: [
        now.get(CaptionProperty::Opacity),
        before.get(CaptionProperty::Opacity),
      ],
      offset: [
        now.get(CaptionProperty::OffsetX),
        now.get(CaptionProperty::OffsetY),
      ],
    }
  }
}
