// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One leg of a pin: the target followed from one keyframe, forward or
//! backward, as far as the next keyframe or the end of the clip.
//!
//! Keyframes split a pin into legs that know nothing of each other, so moving
//! one keyframe only asks for the legs that start or end on it again.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};

use super::geometry::Rect;
use super::luma::{LumaFrame, Pyramid};
use super::model::PinKeyframe;
use super::path::FrameSource;
use super::target::PinTarget;
use super::tracker::{Observation, Status, Target, Tracker};

/// How much of a backward leg is decoded at a time. A decoder only runs
/// forward, so a backward leg is read in chunks from its keyframe back and
/// each chunk is tracked last frame first. Half a second is one keyframe
/// interval of a working recording, so a chunk costs no decode it does not
/// use, and holds about thirty tracking frames.
const CHUNK_MS: u64 = 500;
/// The preview shows a frame up to two milliseconds early, which a keyframe's
/// frame follows.
const SLACK_MS: u64 = 2;
/// How early a frame may start and still be the one showing at a moment: a
/// frame at 30 fps lasts 33 ms.
pub(crate) const LEAD_MS: u64 = 40;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Leg {
  /// Where the leg starts, and where the annotation is there.
  pub(crate) from: PinKeyframe,
  /// How far it runs: forward, the frames before this; backward, the frames
  /// from this on.
  pub(crate) until_ms: u64,
  pub(crate) forward: bool,
  /// What is followed, where the annotation was drawn.
  pub(crate) target: PinTarget,
}

/// One frame of a leg: where the annotation's anchor has moved to from where
/// it was drawn, in source pixels, and how far the frame is to be trusted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LegSample {
  pub(crate) ms: u64,
  pub(crate) dx: f32,
  pub(crate) dy: f32,
  pub(crate) scale: f32,
  pub(crate) confidence: f32,
  pub(crate) status: Status,
}

impl Leg {
  /// A number shared by every leg that starts from the same keyframe in the
  /// same direction after the same target, however far it runs. A leg is
  /// followed one frame after the next, so a longer leg from the same origin
  /// holds this one's frames exactly: see [`Leg::cut_from`].
  pub(crate) fn origin(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.from.ms.hash(&mut hasher);
    self.from.dx.to_bits().hash(&mut hasher);
    self.from.dy.to_bits().hash(&mut hasher);
    self.forward.hash(&mut hasher);
    self.target.hash_into(&mut hasher);
    hasher.finish()
  }

  /// Whether a leg from the same origin running to `until_ms` covers this
  /// one.
  pub(crate) fn within(&self, until_ms: u64) -> bool {
    if self.forward {
      until_ms >= self.until_ms
    } else {
      until_ms <= self.until_ms
    }
  }

  /// This leg's frames out of `longer`, a leg from the same origin that
  /// covers it.
  pub(crate) fn cut_from(&self, longer: &[LegSample]) -> Vec<LegSample> {
    longer
      .iter()
      .filter(|sample| {
        if self.forward {
          sample.ms < self.until_ms.max(self.from.ms + SLACK_MS + 1)
        } else {
          sample.ms >= self.until_ms
        }
      })
      .copied()
      .collect()
  }

  /// How many frames the leg spans at `fps`, for progress.
  pub(crate) fn frames(&self, fps: f64) -> u64 {
    (self.from.ms.abs_diff(self.until_ms) as f64 * fps / 1_000.0).ceil() as u64 + 1
  }

  /// How far the leg's frames are from `ms`, for doing the nearest first.
  pub(crate) fn distance_from(&self, ms: u64) -> u64 {
    let (low, high) = if self.forward {
      (self.from.ms, self.until_ms)
    } else {
      (self.until_ms, self.from.ms)
    };
    if ms < low {
      low - ms
    } else {
      ms.saturating_sub(high)
    }
  }
}

/// Follows `leg` through `source`, calling `progress` once a frame. `None`
/// once `stop` is set.
pub(crate) fn track_leg(
  source: &mut dyn FrameSource,
  leg: &Leg,
  stop: &AtomicBool,
  progress: &mut dyn FnMut(),
) -> Result<Option<Vec<LegSample>>, String> {
  let (source_width, _) = source.source_size();
  let keyframe_end = leg.from.ms.saturating_add(SLACK_MS + 1);
  let mut samples = Vec::new();
  if leg.forward {
    let mut follower: Option<Follower> = None;
    let mut pending: Option<LumaFrame> = None;
    let mut stopped = false;
    let mut failed = None;
    source.read(
      leg.from.ms.saturating_sub(LEAD_MS),
      leg.until_ms.max(keyframe_end),
      &mut |frame| {
        if stop.load(Ordering::Relaxed) {
          stopped = true;
          return false;
        }
        if frame.ms < keyframe_end {
          pending = Some(frame);
          return true;
        }
        if follower.is_none() {
          let Some(pinned) = pending.take() else {
            failed = Some("The recording has no frame at the keyframe".to_owned());
            return false;
          };
          let started = Follower::new(&pinned, leg, source_width);
          samples.push(started.sample(started.tracker.pinned()));
          follower = Some(started);
        }
        let Some(follower) = follower.as_mut() else {
          return false;
        };
        if frame.ms < leg.until_ms {
          let observation = follower.tracker.step(Pyramid::new(&frame));
          samples.push(follower.sample(observation));
          progress();
        }
        frame.ms < leg.until_ms
      },
    )?;
    if stopped {
      return Ok(None);
    }
    if let Some(error) = failed {
      return Err(error);
    }
    // A keyframe on the last frame has nothing after it.
    if let (None, Some(pinned)) = (follower, pending) {
      let started = Follower::new(&pinned, leg, source_width);
      samples.push(started.sample(started.tracker.pinned()));
    }
    return Ok(Some(samples));
  }

  // Backward, a chunk at a time, each chunk last frame first.
  let lower = leg.until_ms;
  let mut chunk_start = leg.from.ms.saturating_sub(CHUNK_MS).max(lower);
  let mut chunk = Vec::new();
  source.read(chunk_start, keyframe_end, &mut |frame| {
    chunk.push(frame);
    true
  })?;
  chunk.retain(|frame| frame.ms < keyframe_end);
  let Some(pinned) = chunk.pop() else {
    return Err("The recording has no frame at the keyframe".to_owned());
  };
  let mut follower = Follower::new(&pinned, leg, source_width);
  samples.push(follower.sample(follower.tracker.pinned()));
  drop(pinned);
  loop {
    for frame in chunk.drain(..).rev() {
      if stop.load(Ordering::Relaxed) {
        return Ok(None);
      }
      if frame.ms < lower {
        continue;
      }
      let observation = follower.tracker.step(Pyramid::new(&frame));
      samples.push(follower.sample(observation));
      progress();
    }
    if chunk_start <= lower {
      break;
    }
    let chunk_end = chunk_start;
    chunk_start = chunk_end.saturating_sub(CHUNK_MS).max(lower);
    source.read(chunk_start, chunk_end, &mut |frame| {
      chunk.push(frame);
      true
    })?;
    chunk.retain(|frame| frame.ms >= chunk_start && frame.ms < chunk_end);
  }
  samples.reverse();
  Ok(Some(samples))
}

/// A tracker started on a keyframe's frame, and how its answers turn back
/// into source pixels.
struct Follower {
  tracker: Tracker,
  /// The anchor where the keyframe puts it, in tracking pixels.
  anchor: [f32; 2],
  /// The anchor where the annotation was drawn, in source pixels.
  drawn: [f64; 2],
  factor: f64,
}

impl Follower {
  fn new(pinned: &LumaFrame, leg: &Leg, source_width: u32) -> Self {
    let factor = f64::from(pinned.width) / f64::from(source_width.max(1));
    let placed = leg.target.shifted([leg.from.dx, leg.from.dy]);
    let scaled = |value: f64| (value * factor) as f32;
    let target = Target {
      region: Rect {
        x0: scaled(placed.region[0]),
        y0: scaled(placed.region[1]),
        x1: scaled(placed.region[2]),
        y1: scaled(placed.region[3]),
      },
      anchor: [scaled(placed.anchor[0]), scaled(placed.anchor[1])],
      centre_weighted: leg.target.redaction,
    };
    Self {
      tracker: Tracker::new(Pyramid::new(pinned), target),
      anchor: target.anchor,
      drawn: leg.target.anchor,
      factor,
    }
  }

  fn sample(&self, observation: Observation) -> LegSample {
    let [x, y] = observation.transform.apply(self.anchor);
    LegSample {
      ms: observation.ms,
      dx: (f64::from(x) / self.factor - self.drawn[0]) as f32,
      dy: (f64::from(y) / self.factor - self.drawn[1]) as f32,
      scale: observation.transform.scale(),
      confidence: observation.confidence,
      status: observation.status,
    }
  }
}
