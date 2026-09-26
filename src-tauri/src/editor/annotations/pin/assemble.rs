// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A pin's legs joined into one path across its clip.
//!
//! Between two keyframes the target is followed from both, and the two are
//! blended by how near each keyframe is and how well each matched, so a
//! correction pulls the path towards it from both sides and a leg that lost
//! the target is carried by the other. The joined path is smoothed, a stretch
//! off the frame is carried on at the speed it left or came back at, and a
//! redaction is grown over every stretch the content was lost in.

use std::collections::BTreeMap;

use super::leg::{Leg, LegSample};
use super::model::{PinKeyframe, KEYFRAME_SLACK_MS};
use super::path::PinRequest;
use super::resolve::{PinSample, PinnedPath};
use super::smooth::{smooth, Sample, LOG_SCALE, POSITION};
use super::tracker::Status;

/// Runs of samples, and how a path's stretches off the frame and a
/// redaction's doubtful ones are filled in.
#[path = "assemble_stretches.rs"]
mod stretches;
use stretches::{carry_hidden, grown, runs};

/// A frame below this is doubtful: the tracker lost the content there and
/// bridged it, or took it back from a match its keyframe's points could not
/// confirm. It is shown on the timeline for checking, and grown over by a
/// redaction. A frame followed soundly never falls below it.
pub(crate) const WEAK: f32 = 0.5;

#[derive(Clone, Copy)]
struct Raw {
  ms: u64,
  dx: f32,
  dy: f32,
  scale: f32,
  confidence: f32,
  status: Status,
}

impl Raw {
  fn exact(keyframe: &PinKeyframe, ms: u64) -> Self {
    Self {
      ms,
      dx: keyframe.dx as f32,
      dy: keyframe.dy as f32,
      scale: 1.0,
      confidence: 1.0,
      status: Status::Tracked,
    }
  }

  fn from(sample: &LegSample) -> Self {
    Self {
      ms: sample.ms,
      dx: sample.dx,
      dy: sample.dy,
      scale: sample.scale,
      confidence: sample.confidence,
      status: sample.status,
    }
  }
}

/// The frames a leg from each side has for one moment.
#[derive(Default)]
struct Sides {
  forward: Option<(u64, LegSample)>,
  backward: Option<(u64, LegSample)>,
}

/// Blends the leg followed forward from the keyframe before with the one
/// followed back from the keyframe after, `share` of the way from the first
/// to the second.
fn blend(forward: Option<&LegSample>, backward: Option<&LegSample>, share: f32) -> Raw {
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

/// Joins `legs` into `request`'s path. `source` is the recording's size in
/// source pixels, which decides what is on the frame.
pub(crate) fn assemble(
  request: &PinRequest,
  legs: &[(Leg, &[LegSample])],
  source: (u32, u32),
) -> PinnedPath {
  let lower = request.start_ms.saturating_sub(super::leg::LEAD_MS);
  let mut moments: BTreeMap<u64, Sides> = BTreeMap::new();
  for (leg, samples) in legs {
    for sample in samples
      .iter()
      .filter(|sample| sample.ms >= lower && sample.ms < request.end_ms)
    {
      let sides = moments.entry(sample.ms).or_default();
      let slot = if leg.forward {
        &mut sides.forward
      } else {
        &mut sides.backward
      };
      // Where legs meet on a keyframe, the one starting there answers.
      let nearer =
        slot.is_none_or(|(from, _)| from.abs_diff(sample.ms) > leg.from.ms.abs_diff(sample.ms));
      if nearer {
        *slot = Some((leg.from.ms, *sample));
      }
    }
  }

  let keyframes = &request.keyframes;
  let raw: Vec<Raw> = moments
    .iter()
    .map(|(&ms, sides)| {
      if let Some(keyframe) = keyframes
        .iter()
        .find(|keyframe| keyframe.ms.abs_diff(ms) <= KEYFRAME_SLACK_MS)
      {
        return Raw::exact(keyframe, ms);
      }
      let before = keyframes.iter().rev().find(|keyframe| keyframe.ms <= ms);
      let after = keyframes.iter().find(|keyframe| keyframe.ms >= ms);
      let share = match (before, after) {
        (Some(before), Some(after)) if after.ms > before.ms => {
          (ms - before.ms) as f32 / (after.ms - before.ms) as f32
        }
        (Some(_), _) => 0.0,
        _ => 1.0,
      };
      let mut raw = blend(
        sides.forward.as_ref().map(|(_, sample)| sample),
        sides.backward.as_ref().map(|(_, sample)| sample),
        share,
      );
      raw.ms = ms;
      raw
    })
    .collect();

  let axis = |value: &dyn Fn(&Raw) -> f32, tuning| {
    let samples: Vec<Sample> = raw
      .iter()
      .map(|raw| Sample {
        ms: raw.ms,
        value: (raw.status == Status::Tracked).then(|| (value(raw), raw.confidence)),
      })
      .collect();
    smooth(&samples, tuning)
  };
  let mut dx = axis(&|raw| raw.dx, POSITION);
  let mut dy = axis(&|raw| raw.dy, POSITION);
  let scale: Vec<f32> = axis(&|raw| raw.scale.max(1e-3).ln(), LOG_SCALE)
    .into_iter()
    .map(f32::exp)
    .collect();
  carry_hidden(&raw, &mut dx, &mut dy);

  let target = &request.target;
  let (width, height) = (f64::from(source.0), f64::from(source.1));
  let visible: Vec<bool> = (0..raw.len())
    .map(|index| {
      let (x, y) = (f64::from(dx[index]), f64::from(dy[index]));
      if target.redaction {
        let s = f64::from(scale[index]);
        let [ax, ay] = target.anchor;
        let place = |value: f64, anchor: f64, shift: f64| anchor + shift + s * (value - anchor);
        let (x0, y0) = (
          place(target.region[0], ax, x),
          place(target.region[1], ay, y),
        );
        let (x1, y1) = (
          place(target.region[2], ax, x),
          place(target.region[3], ay, y),
        );
        x1 > 0.0 && y1 > 0.0 && x0 < width && y0 < height
      } else {
        let (ax, ay) = (target.anchor[0] + x, target.anchor[1] + y);
        raw[index].status != Status::Hidden && ax >= 0.0 && ay >= 0.0 && ax < width && ay < height
      }
    })
    .collect();

  let span = |(from, to): (usize, usize)| {
    let start = if from == 0 {
      request.start_ms
    } else {
      raw[from].ms
    };
    let end = raw.get(to + 1).map_or(request.end_ms, |next| next.ms);
    [start, end]
  };
  let weak_runs = runs(raw.len(), |index| {
    visible[index] && raw[index].confidence < WEAK
  });
  let growth = if target.redaction {
    grown(&weak_runs, &dx, &dy, target.region)
  } else {
    Vec::new()
  };
  PinnedPath {
    key: request.key(),
    samples: raw
      .iter()
      .enumerate()
      .map(|(index, raw)| PinSample {
        ms: raw.ms,
        dx: dx[index],
        dy: dy[index],
        scale: scale[index],
        confidence: raw.confidence,
        visible: visible[index],
      })
      .collect(),
    growth,
    shown: runs(raw.len(), |index| visible[index])
      .into_iter()
      .map(span)
      .collect(),
    weak: weak_runs.iter().copied().map(span).collect(),
    hidden: runs(raw.len(), |index| raw[index].status == Status::Hidden)
      .into_iter()
      .map(span)
      .collect(),
  }
}
