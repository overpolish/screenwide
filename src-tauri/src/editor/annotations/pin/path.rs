// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::hash::{DefaultHasher, Hash, Hasher};

use super::leg::{Leg, LEAD_MS};
use super::luma::LumaFrame;
use super::model::{AnnotationPin, PinKeyframe};
use super::target::PinTarget;
use crate::editor::annotations::timing::RecordingAnnotationClip;

/// Where a pin's frames come from: a recording decoded at the tracking size.
pub(crate) trait FrameSource {
  /// The recording's size in source pixels.
  fn source_size(&self) -> (u32, u32);
  /// Every frame from `start_ms` up to `end_ms`, in rising order, until
  /// `each` returns false.
  fn read(
    &mut self,
    start_ms: u64,
    end_ms: u64,
    each: &mut dyn FnMut(LumaFrame) -> bool,
  ) -> Result<(), String>;
}

/// Everything a pin's path depends on, in source milliseconds and pixels.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PinRequest {
  pub(crate) start_ms: u64,
  pub(crate) end_ms: u64,
  /// In time order, one to a moment, never empty.
  pub(crate) keyframes: Vec<PinKeyframe>,
  pub(crate) target: PinTarget,
}

impl PinRequest {
  /// What `clip`'s pin asks to be tracked, on a recording `source` pixels in
  /// size.
  pub(crate) fn of(
    clip: &RecordingAnnotationClip,
    pin: &AnnotationPin,
    source: (u32, u32),
  ) -> Self {
    Self {
      start_ms: clip.start_ms,
      end_ms: clip.end_ms,
      keyframes: pin.sorted(),
      target: PinTarget::of(&clip.annotation, source),
    }
  }

  /// A number that changes whenever the path would.
  pub(crate) fn key(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.start_ms.hash(&mut hasher);
    self.end_ms.hash(&mut hasher);
    for keyframe in &self.keyframes {
      keyframe.ms.hash(&mut hasher);
      keyframe.dx.to_bits().hash(&mut hasher);
      keyframe.dy.to_bits().hash(&mut hasher);
    }
    self.target.hash_into(&mut hasher);
    hasher.finish()
  }

  /// The legs the path is joined from: back from the first keyframe to the
  /// clip's start, both ways between each pair of keyframes, and on from the
  /// last to the clip's end. A keyframe left outside the clip by a trim is
  /// still followed to, so the clip keeps its corrections.
  pub(crate) fn legs(&self) -> Vec<Leg> {
    let lower = self.start_ms.saturating_sub(LEAD_MS);
    let leg = |from: PinKeyframe, until_ms, forward| Leg {
      from,
      until_ms,
      forward,
      target: self.target,
    };
    let (Some(first), Some(last)) = (self.keyframes.first(), self.keyframes.last()) else {
      return Vec::new();
    };
    let mut legs = Vec::new();
    if first.ms > lower {
      legs.push(leg(*first, lower, false));
    }
    for pair in self.keyframes.windows(2) {
      legs.push(leg(pair[0], pair[1].ms, true));
      legs.push(leg(pair[1], pair[0].ms, false));
    }
    if last.ms < self.end_ms {
      legs.push(leg(*last, self.end_ms, true));
    }
    legs
  }
}

/// Follows `request` across its clip, one leg after another. `None` once
/// `stop` is set.
#[cfg(test)]
pub(crate) fn track_pin(
  source: &mut dyn FrameSource,
  request: &PinRequest,
  stop: &std::sync::atomic::AtomicBool,
) -> Result<Option<super::resolve::PinnedPath>, String> {
  use super::assemble::assemble;
  use super::leg::track_leg;
  let mut tracked = Vec::new();
  for leg in request.legs() {
    let Some(samples) = track_leg(source, &leg, stop, &mut || {})? else {
      return Ok(None);
    };
    tracked.push((leg, samples));
  }
  let legs: Vec<_> = tracked
    .iter()
    .map(|(leg, samples)| (*leg, samples.as_slice()))
    .collect();
  Ok(Some(assemble(request, &legs, source.source_size())))
}
