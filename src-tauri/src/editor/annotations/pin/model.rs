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
use ts_rs::TS;

use super::resolve::PinnedPath;

/// How close to a keyframe, in source milliseconds, a moment is still that
/// keyframe's: about half a frame, so the frame the playhead shows is the
/// one a drag there corrects.
pub(crate) const KEYFRAME_SLACK_MS: u64 = 8;

/// Where the annotation is to be at `ms`: moved by `dx` and `dy` source
/// pixels from where it was drawn. A redaction resized away from the frame
/// it was pinned on also says how far each of its edges - left, top, right,
/// bottom - sits outside the box it was drawn as, once moved there.
///
/// An out-of-view keyframe says instead that the content cannot be seen from
/// `ms`: the annotation is hidden until the next keyframe, and nothing is
/// followed in between. Its movement means nothing.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PinKeyframe {
  pub ms: u64,
  pub dx: f64,
  pub dy: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub edges: Option<[f64; 4]>,
  #[serde(default, skip_serializing_if = "is_false")]
  pub out_of_view: bool,
}

fn is_false(value: &bool) -> bool {
  !*value
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
      && self.keyframes.iter().all(|keyframe| {
        keyframe.dx.is_finite()
          && keyframe.dy.is_finite()
          && keyframe
            .edges
            .is_none_or(|edges| edges.iter().all(|edge| edge.is_finite()))
      })
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
  /// adds one. `edges`, where given, becomes its size; a keyframe already
  /// there keeps its own otherwise.
  pub(crate) fn set_keyframe(&mut self, ms: u64, [dx, dy]: [f64; 2], edges: Option<[f64; 4]>) {
    let existing = self
      .keyframes
      .iter()
      .enumerate()
      .filter(|(_, keyframe)| keyframe.ms.abs_diff(ms) <= KEYFRAME_SLACK_MS)
      .min_by_key(|(_, keyframe)| keyframe.ms.abs_diff(ms))
      .map(|(index, _)| index);
    match existing {
      // Put somewhere by the hand, the annotation is in view there.
      Some(index) => {
        let keyframe = &mut self.keyframes[index];
        keyframe.dx = dx;
        keyframe.dy = dy;
        keyframe.edges = edges.or(keyframe.edges);
        keyframe.out_of_view = false;
      }
      None => {
        self.keyframes.push(PinKeyframe {
          ms,
          dx,
          dy,
          edges,
          out_of_view: false,
        });
        self.keyframes.sort_by_key(|keyframe| keyframe.ms);
      }
    }
  }

  /// How far a redaction's edges sit outside its drawn box at `ms`. Only the
  /// keyframes that resized it take part: the size eases from one resize to
  /// the next, holds after the last, and is the size it was drawn at before
  /// the first. A keyframe that only moved it leaves the size as it is, and a
  /// resize late in a clip does not creep in from the frame it was pinned on.
  pub(crate) fn edges_at(&self, ms: u64) -> [f64; 4] {
    let resizes: Vec<(u64, [f64; 4])> = self
      .sorted()
      .into_iter()
      .filter_map(|keyframe| Some((keyframe.ms, keyframe.edges?)))
      .collect();
    if let Some((_, edges)) = resizes
      .iter()
      .find(|(at, _)| at.abs_diff(ms) <= KEYFRAME_SLACK_MS)
    {
      return *edges;
    }
    let before = resizes.iter().rev().find(|(at, _)| *at <= ms);
    let after = resizes.iter().find(|(at, _)| *at > ms);
    match (before, after) {
      (Some((from_ms, from)), Some((to_ms, to))) => {
        let share = (ms - from_ms) as f64 / (to_ms - from_ms).max(1) as f64;
        std::array::from_fn(|side| from[side] + share * (to[side] - from[side]))
      }
      (Some((_, edges)), None) => *edges,
      _ => [0.0; 4],
    }
  }

  /// The stretches the hand said the content is out of view in: from each
  /// out-of-view keyframe to the keyframe after it, or on past the clip.
  pub(crate) fn out_of_view(&self) -> Vec<[u64; 2]> {
    let keyframes = self.sorted();
    keyframes
      .iter()
      .enumerate()
      .filter(|(_, keyframe)| keyframe.out_of_view)
      .map(|(index, keyframe)| {
        let end = keyframes.get(index + 1).map_or(u64::MAX, |next| next.ms);
        [keyframe.ms, end]
      })
      .collect()
  }

  /// Whether the hand said the content is out of view at `ms`.
  pub(crate) fn out_of_view_at(&self, ms: u64) -> bool {
    self
      .out_of_view()
      .iter()
      .any(|[start, end]| (*start..*end).contains(&ms))
  }
}
