// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::leg::{LegSample, NO_COVER};
use super::super::model::PinKeyframe;
use super::super::tracker::Status;

/// One moment of a path, as the legs have it before it is smoothed.
#[derive(Clone, Copy)]
pub(super) struct Raw {
  pub(super) ms: u64,
  pub(super) dx: f32,
  pub(super) dy: f32,
  pub(super) scale: f32,
  pub(super) confidence: f32,
  pub(super) status: Status,
  /// The covers' edges either leg had proven by this moment, which only a
  /// stretch lost under one acts on: see `covers`.
  pub(super) proven: [f32; 4],
  /// What the box is trimmed to.
  pub(super) view: [f32; 4],
  /// Lost under a still cover.
  pub(super) under: bool,
}

impl Raw {
  pub(super) fn exact(keyframe: &PinKeyframe, ms: u64) -> Self {
    Self {
      ms,
      dx: keyframe.dx as f32,
      dy: keyframe.dy as f32,
      scale: 1.0,
      confidence: 1.0,
      status: Status::Tracked,
      proven: NO_COVER,
      view: NO_COVER,
      under: false,
    }
  }

  pub(super) fn from(sample: &LegSample) -> Self {
    Self {
      ms: sample.ms,
      dx: sample.dx,
      dy: sample.dy,
      scale: sample.scale,
      confidence: sample.confidence,
      status: sample.status,
      proven: sample.proven,
      view: NO_COVER,
      under: false,
    }
  }
}

/// The frames a leg from each side has for one moment.
#[derive(Default)]
pub(super) struct Sides {
  pub(super) forward: Option<(u64, LegSample)>,
  pub(super) backward: Option<(u64, LegSample)>,
}

impl Sides {
  /// The part of the frame neither leg proved a cover over: a cover either
  /// proved is there, the other leg may just not have seen the content move
  /// past it yet.
  pub(super) fn proven(&self) -> [f32; 4] {
    let mut proven = NO_COVER;
    for (_, sample) in self.forward.iter().chain(&self.backward) {
      proven = [
        proven[0].max(sample.proven[0]),
        proven[1].max(sample.proven[1]),
        proven[2].min(sample.proven[2]),
        proven[3].min(sample.proven[3]),
      ];
    }
    proven
  }
}

/// Blends the leg followed forward from the keyframe before with the one
/// followed back from the keyframe after, `share` of the way from the first
/// to the second.
pub(super) fn blend(forward: Option<&LegSample>, backward: Option<&LegSample>, share: f32) -> Raw {
  let tracked = |sample: Option<&LegSample>| {
    sample
      .filter(|sample| sample.status == Status::Tracked)
      .copied()
  };
  match (tracked(forward), tracked(backward)) {
    // The two legs meet because a keyframe was set between them, which is
    // where they are expected to disagree: the blend is the correction.
    (Some(f), Some(b)) => {
      let wf = f.confidence * (1.0 - share) + 1e-3;
      let wb = b.confidence * share + 1e-3;
      let total = wf + wb;
      Raw {
        ms: f.ms,
        dx: (wf * f.dx + wb * b.dx) / total,
        dy: (wf * f.dy + wb * b.dy) / total,
        scale: (wf * f.scale + wb * b.scale) / total,
        confidence: (wf * f.confidence + wb * b.confidence) / total,
        status: Status::Tracked,
        proven: NO_COVER,
        view: NO_COVER,
        under: false,
      }
    }
    (Some(one), None) | (None, Some(one)) => Raw::from(&one),
    (None, None) => {
      let near = if share < 0.5 {
        forward.or(backward)
      } else {
        backward.or(forward)
      };
      let near = near.expect("a moment has a leg from one side at least");
      let hidden = [forward, backward]
        .iter()
        .flatten()
        .any(|sample| sample.status == Status::Hidden);
      Raw {
        confidence: 0.0,
        status: if hidden { Status::Hidden } else { Status::Lost },
        ..Raw::from(near)
      }
    }
  }
}
