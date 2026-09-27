// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding a pinned annotation's content again on one frame, where the hand
//! says it is back in view: by how it looked on a keyframe it was plainly
//! seen on, near where it was last shown.

use super::leg::{Follower, Leg};
use super::luma::{LumaFrame, Pyramid};
use super::model::{PinKeyframe, KEYFRAME_SLACK_MS};
use super::path::FrameSource;
use super::target::PinTarget;

/// How far before a moment its frame is looked for. A recording that stands
/// still has no new frame for a while, so the frame showing may be well
/// before it.
const LOOK_BACK_MS: u64 = 1_000;

/// The frame showing at `ms`.
fn frame_at(source: &mut dyn FrameSource, ms: u64) -> Result<Option<LumaFrame>, String> {
  let mut showing = None;
  source.read(
    ms.saturating_sub(LOOK_BACK_MS),
    ms + KEYFRAME_SLACK_MS + 1,
    &mut |frame| {
      showing = Some(frame);
      true
    },
  )?;
  Ok(showing)
}

/// Where the content `target` follows is at `at_ms`, as a movement from
/// where the annotation was drawn: looked for near `near`, the movement it
/// was last shown with, by its look on `keyframe`'s frame. `None` when the
/// look is not found there with its keyframe's points agreeing.
pub(crate) fn find_again(
  source: &mut dyn FrameSource,
  target: PinTarget,
  keyframe: PinKeyframe,
  near: [f64; 2],
  at_ms: u64,
) -> Result<Option<[f64; 2]>, String> {
  let (Some(seen), Some(now)) = (frame_at(source, keyframe.ms)?, frame_at(source, at_ms)?) else {
    return Ok(None);
  };
  let leg = Leg {
    from: keyframe,
    until_ms: at_ms,
    forward: at_ms >= keyframe.ms,
    target,
  };
  let mut follower = Follower::new(&seen, &leg, source.source_size().0);
  let around = [
    ((near[0] - keyframe.dx) * follower.factor) as f32,
    ((near[1] - keyframe.dy) * follower.factor) as f32,
  ];
  Ok(
    follower
      .tracker
      .find(&Pyramid::new(&now), around)
      .map(|found| {
        let sample = follower.sample(found);
        [f64::from(sample.dx), f64::from(sample.dy)]
      }),
  )
}
