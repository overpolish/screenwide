// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A pin's legs joined into one path across its clip.
//!
//! Between two keyframes the target is followed from both, and the two are
//! blended by how near each keyframe is and how well each matched, so a
//! correction pulls the path towards it from both sides and a leg that lost
//! the target is carried by the other. The joined path is smoothed, a stretch
//! off the frame or under a still cover is carried on at the speed it left
//! or came back at, a redaction is trimmed clear of the cover, and grown over
//! every other stretch the content was lost in.

use std::collections::BTreeMap;

use super::leg::{Leg, LegSample};
use super::model::KEYFRAME_SLACK_MS;
use super::path::PinRequest;
use super::resolve::{PinSample, PinnedPath};
use super::smooth::{smooth, Sample, LOG_SCALE, POSITION};
use super::tracker::Status;

/// Stretches the content spent under a still cover.
mod covers;
/// What each leg has for one moment, and how the two are blended.
mod moment;
/// Runs of samples, and how a path's stretches off the frame and a
/// redaction's doubtful ones are filled in.
mod stretches;
use covers::under_covers;
use moment::{blend, Raw, Sides};
use stretches::{
  bridge_lost, carry_hidden, grown, hide_unseen, in_sight, out_of_view, runs, unseen_between,
};

/// A frame below this is doubtful: the tracker lost the content there and
/// bridged it, or took it back from a match its keyframe's points could not
/// confirm. It is shown on the timeline for checking, and grown over by a
/// redaction. A frame followed soundly never falls below it.
pub(crate) const WEAK: f32 = 0.5;

/// How long an arrow, counter or text box may have its content out of sight
/// on screen before it is hidden, in milliseconds.
const LOST_GRACE_MS: u64 = 200;

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
  let unseen = unseen_between(&moments, keyframes, LOST_GRACE_MS);
  let mut raw: Vec<Raw> = moments
    .iter()
    .map(|(&ms, sides)| {
      if let Some(keyframe) = keyframes
        .iter()
        .find(|keyframe| keyframe.ms.abs_diff(ms) <= KEYFRAME_SLACK_MS)
      {
        return Raw {
          proven: sides.proven(),
          ..Raw::exact(keyframe, ms)
        };
      }
      let slot = keyframes.iter().rposition(|keyframe| keyframe.ms <= ms);
      let before = slot.map(|index| &keyframes[index]);
      let after = keyframes.iter().find(|keyframe| keyframe.ms >= ms);
      let share = match (
        before,
        after,
        slot.and_then(|index| unseen.get(index).copied().flatten()),
      ) {
        (_, _, Some((start, _))) if ms < start => 0.0,
        (_, _, Some((_, end))) if ms >= end => 1.0,
        (Some(before), Some(after), _) if after.ms > before.ms => {
          (ms - before.ms) as f32 / (after.ms - before.ms) as f32
        }
        (Some(_), _, _) => 0.0,
        _ => 1.0,
      };
      let mut raw = blend(
        sides.forward.as_ref().map(|(_, sample)| sample),
        sides.backward.as_ref().map(|(_, sample)| sample),
        share,
      );
      raw.ms = ms;
      raw.proven = sides.proven();
      raw
    })
    .collect();
  let unseen_by_hand = out_of_view(&mut raw, keyframes);

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
  let mut scale: Vec<f32> = axis(&|raw| raw.scale.max(1e-3).ln(), LOG_SCALE)
    .into_iter()
    .map(f32::exp)
    .collect();
  let target = &request.target;
  under_covers(&mut raw, [&dx, &dy, &scale], target);
  carry_hidden(&raw, &mut dx, &mut dy);
  bridge_lost(&raw, &mut [&mut dx, &mut dy, &mut scale]);

  let mut visible = in_sight(&raw, [&dx, &dy, &scale], target, source);

  let span = |(from, to): (usize, usize)| {
    let start = if from == 0 {
      request.start_ms
    } else {
      raw[from].ms
    };
    let end = raw.get(to + 1).map_or(request.end_ms, |next| next.ms);
    [start, end]
  };
  let covered = hide_unseen(&raw, &mut visible, &unseen_by_hand, target.redaction, &span);
  // Content known to be under a cover is not lost: it is where it went.
  let weak_runs = runs(raw.len(), |index| {
    (visible[index] || covered[index])
      && !unseen_by_hand[index]
      && !raw[index].under
      && raw[index].confidence < WEAK
  });
  let (growth, views) = if target.redaction {
    (
      grown(&weak_runs, &dx, &dy, target.region),
      raw.iter().map(|raw| raw.view).collect(),
    )
  } else {
    (Vec::new(), Vec::new())
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
    views,
    shown: runs(raw.len(), |index| visible[index])
      .into_iter()
      .map(span)
      .collect(),
    weak: weak_runs.iter().copied().map(span).collect(),
    hidden: runs(raw.len(), |index| {
      raw[index].status == Status::Hidden && !unseen_by_hand[index]
    })
    .into_iter()
    .map(span)
    .collect(),
    covered: runs(raw.len(), |index| covered[index])
      .into_iter()
      .map(span)
      .collect(),
    under: runs(raw.len(), |index| {
      raw[index].under && !visible[index] && !unseen_by_hand[index]
    })
    .into_iter()
    .map(span)
    .collect(),
  }
}
