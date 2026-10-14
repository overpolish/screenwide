// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a piece is drawn at one moment: moved by the entrance it came on by,
//! and, once its caption starts going off, by the exits it goes off by too.

use super::CaptionLineTurn;
use crate::motion::{pose, CaptionMotionTrack, CaptionProperty, Mirror, Pose};

/// Whose motion a track is: the caption's, which turns the whole line about
/// its middle, or a word's, which turns the word about its own.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Level {
  Caption,
  Word,
}

/// How a piece is drawn, now and as the frame's exposure began, which the
/// compositor blurs between: how large about its centre, how solid, how far
/// it is moved, in ems of its type, and how far it is turned about its own
/// middle and with the line about the caption's, in degrees about the
/// horizontal, vertical and depth axes, with how far the caption's own
/// motion moves that line, in ems.
#[derive(Clone, Copy)]
pub(super) struct Shown {
  pub(super) scale: [f64; 2],
  pub(super) opacity: [f64; 2],
  pub(super) offset: [[f64; 2]; 2],
  pub(super) own_turn: [[f64; 3]; 2],
  pub(super) line_turn: [[f64; 3]; 2],
  pub(super) line_offset: [[f64; 2]; 2],
}

/// `at` moved `offset` ems of type `size` across and down.
pub(super) fn moved(at: [f64; 2], offset: [f64; 2], size: f64) -> [f64; 2] {
  [at[0] + offset[0] * size, at[1] + offset[1] * size]
}

impl Shown {
  /// How the caption turns as one, if it does, about the middle of `line`,
  /// its lines' centre and block, moved as the caption's own motion moves
  /// it in ems of its type, `size`.
  pub(super) fn line(
    &self,
    line: Option<([f64; 2], [f64; 2])>,
    size: f64,
  ) -> Option<CaptionLineTurn> {
    let (pivot, block) = line.filter(|_| self.line_turn != [[0.0; 3]; 2])?;
    Some(CaptionLineTurn {
      turn: self.line_turn,
      pivot: self.line_offset.map(|offset| moved(pivot, offset, size)),
      reach: block[0].max(block[1]) / 2.0,
    })
  }
}

/// What moves a piece: the tracks it came on by, if any, how long ago they
/// started and whose they are; and the tracks it goes off by, each with how
/// long ago its exit started, less than nothing before then, and whose they
/// are: a rolling word goes off before the caption does.
#[derive(Clone, Copy)]
pub(super) struct Moving<'a> {
  pub(super) entrance: Option<(&'a [CaptionMotionTrack], f64, Level)>,
  pub(super) exits: [(&'a [CaptionMotionTrack], f64, Level); 2],
}

/// A pose's turns about the horizontal, vertical and depth axes.
fn turn(pose: Pose) -> [f64; 3] {
  [
    pose.get(CaptionProperty::RotateX),
    pose.get(CaptionProperty::RotateY),
    pose.get(CaptionProperty::Rotate),
  ]
}

fn offset(pose: Pose) -> [f64; 2] {
  [
    pose.get(CaptionProperty::OffsetX),
    pose.get(CaptionProperty::OffsetY),
  ]
}

/// The motions moving a piece at one moment: every one stacked, and the
/// turns of the word's and the caption's apart, with the caption's offset.
struct Posed {
  all: Pose,
  own_turn: [f64; 3],
  line_turn: [f64; 3],
  line_offset: [f64; 2],
}

impl Moving<'_> {
  /// Every property's value `back_ms` ago. An exit moves nothing before it
  /// starts, though its tracks hold their first keyframe's value then.
  fn pose(&self, back_ms: f64, mirror: Mirror) -> Posed {
    let mut posed = Posed {
      all: pose(&[], 0.0, mirror),
      own_turn: [0.0; 3],
      line_turn: [0.0; 3],
      line_offset: [0.0; 2],
    };
    let parts = self
      .entrance
      .map(|(tracks, elapsed_ms, level)| (tracks, elapsed_ms - back_ms, level))
      .into_iter()
      .chain(
        self
          .exits
          .into_iter()
          .map(|(tracks, elapsed_ms, level)| (tracks, elapsed_ms - back_ms, level))
          .filter(|(_, exit_ms, _)| *exit_ms >= 0.0),
      );
    for (tracks, elapsed_ms, level) in parts {
      let part = pose(tracks, elapsed_ms, mirror);
      posed.all = posed.all.stacked(part);
      let add = |into: &mut [f64], from: &[f64]| {
        for (into, from) in into.iter_mut().zip(from) {
          *into += from;
        }
      };
      if level == Level::Caption {
        add(&mut posed.line_turn, &turn(part));
        add(&mut posed.line_offset, &offset(part));
      } else {
        add(&mut posed.own_turn, &turn(part));
      }
    }
    posed
  }

  /// The piece as it is drawn now and `exposure_ms` before.
  pub(super) fn shown(&self, exposure_ms: f64, mirror: Mirror) -> Shown {
    let [now, before] = [0.0, exposure_ms.max(0.0)].map(|back_ms| self.pose(back_ms, mirror));
    let both = |property| [now.all.get(property), before.all.get(property)];
    Shown {
      scale: both(CaptionProperty::Scale),
      opacity: both(CaptionProperty::Opacity),
      offset: [offset(now.all), offset(before.all)],
      own_turn: [now.own_turn, before.own_turn],
      line_turn: [now.line_turn, before.line_turn],
      line_offset: [now.line_offset, before.line_offset],
    }
  }
}
