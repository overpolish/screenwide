// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A pin as the document stores it: where the annotation was pinned and the
//! places it was put by hand since.
//!
//! The annotation's own geometry is where it was drawn. Each keyframe says
//! where it is to be at one moment, as a movement from that geometry, and the
//! tracker fills in every other frame from the keyframes on either side. The
//! pinned frame is a keyframe like any other; it is only remembered so that
//! clearing the corrections leaves the pin itself behind.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::resolve::PinnedPath;

/// How close to a keyframe, in source milliseconds, a moment is still that
/// keyframe's: about half a frame, so the frame the playhead shows is the
/// one a drag there corrects.
pub(crate) const KEYFRAME_SLACK_MS: u64 = 8;

/// Where the annotation is to be at `ms`: moved by `dx` and `dy` source
/// pixels from where it was drawn.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinKeyframe {
  pub ms: u64,
  pub dx: f64,
  pub dy: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationPin {
  pub pinned_ms: u64,
  pub keyframes: Vec<PinKeyframe>,
  /// The tracked path these keyframes resolve to, attached by whoever has
  /// one to hand and never stored. It may be the path of keyframes that have
  /// since changed while the new one is worked out; [`PinnedPath::key`] says
  /// which.
  #[serde(skip)]
  pub(crate) path: Option<Arc<PinnedPath>>,
}

/// Two pins are the same pin when the document says the same thing; which
/// path either has to hand is not part of it.
impl PartialEq for AnnotationPin {
  fn eq(&self, other: &Self) -> bool {
    self.pinned_ms == other.pinned_ms && self.keyframes == other.keyframes
  }
}

impl AnnotationPin {
  pub(crate) fn is_valid(&self) -> bool {
    !self.keyframes.is_empty()
      && self
        .keyframes
        .iter()
        .all(|keyframe| keyframe.dx.is_finite() && keyframe.dy.is_finite())
  }

  /// The keyframes in time order, one to a moment.
  pub(crate) fn sorted(&self) -> Vec<PinKeyframe> {
    let mut keyframes = self.keyframes.clone();
    keyframes.sort_by_key(|keyframe| keyframe.ms);
    keyframes.dedup_by_key(|keyframe| keyframe.ms);
    keyframes
  }

  /// The keyframe `ms` falls on, if it falls on one.
  pub(crate) fn keyframe_at(&self, ms: u64) -> Option<&PinKeyframe> {
    self
      .keyframes
      .iter()
      .filter(|keyframe| keyframe.ms.abs_diff(ms) <= KEYFRAME_SLACK_MS)
      .min_by_key(|keyframe| keyframe.ms.abs_diff(ms))
  }

  /// The keyframe nearest `ms`, which is where an annotation waits before its
  /// path has been worked out.
  pub(crate) fn nearest(&self, ms: u64) -> Option<&PinKeyframe> {
    self
      .keyframes
      .iter()
      .min_by_key(|keyframe| keyframe.ms.abs_diff(ms))
  }

  /// Puts the annotation at `[dx, dy]` at `ms`: moves the keyframe there, or
  /// adds one.
  pub(crate) fn set_keyframe(&mut self, ms: u64, [dx, dy]: [f64; 2]) {
    let existing = self
      .keyframes
      .iter()
      .enumerate()
      .filter(|(_, keyframe)| keyframe.ms.abs_diff(ms) <= KEYFRAME_SLACK_MS)
      .min_by_key(|(_, keyframe)| keyframe.ms.abs_diff(ms))
      .map(|(index, _)| index);
    match existing {
      Some(index) => {
        self.keyframes[index].dx = dx;
        self.keyframes[index].dy = dy;
      }
      None => {
        self.keyframes.push(PinKeyframe { ms, dx, dy });
        self.keyframes.sort_by_key(|keyframe| keyframe.ms);
      }
    }
  }
}
