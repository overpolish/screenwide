// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The bridge from annotations drawn on the live overlay to editable clips.
//!
//! The overlay owns pixels, this owns time. An annotation lives here for as
//! long as it is on screen, in global logical desktop points, and a running
//! recording turns each annotation's visible span into one annotation clip in
//! the recording's own source pixels. With no recording running, nothing is
//! accumulated and nothing is written.

use std::sync::{Arc, LazyLock, Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::annotations::Annotation;
use crate::recording::clock::SidecarClock;
use crate::recording::cursor::CursorSource;

mod clip;
mod replay;
use clip::annotation_clip;
pub(crate) use replay::ReplayAnnotationRecorder;
use replay::ReplayTimed;

/// An annotation on screen. `shown_at_ms` is recording time, and stays absent
/// until a running recording has seen the annotation: one drawn during a pause
/// enters the recording at the resume rather than at the wall-clock moment it
/// was drawn. `replay_shown_at_us` is the same moment on the replay buffer's
/// timeline, which runs independently of any recording.
struct LiveAnnotation {
  annotation: Annotation,
  replay_shown_at_us: Option<u64>,
  shown_at_ms: Option<u64>,
}

/// What a running recording adds: a clock, the space annotations are converted
/// into, and the clips closed out so far.
struct Timed {
  clips: Vec<RecordingAnnotationClip>,
  clock: SidecarClock,
  source: CursorSource,
}

impl Timed {
  /// Closes out one annotation that stopped being visible at `at`.
  /// [`LiveAnnotations::stop`] trims a closing phase the recording had no
  /// room for.
  fn close(&mut self, live: &LiveAnnotation, at: Instant) {
    // No start means the annotation was drawn during a pause the recording
    // never came back from, so it was never part of it.
    let Some(start_ms) = live.shown_at_ms else {
      return;
    };
    let Some(gone_ms) = self.clock.elapsed_us(at).map(millis) else {
      return;
    };
    if let Some(clip) = annotation_clip(&live.annotation, &self.source, start_ms, gone_ms) {
      self.clips.push(clip);
    }
  }
}

/// The annotations on screen, and the recording they are being timed against.
#[derive(Default)]
pub(crate) struct LiveAnnotations {
  annotations: Vec<LiveAnnotation>,
  replay: Option<ReplayTimed>,
  timed: Option<Timed>,
}

impl LiveAnnotations {
  /// Adds a completed stroke, in global logical desktop points. A repeated id
  /// is dropped, because a clip list that reuses one is rejected whole.
  fn add(&mut self, annotation: Annotation, at: Instant) {
    if self
      .annotations
      .iter()
      .any(|live| live.annotation.id == annotation.id)
    {
      return;
    }
    let shown_at_ms = self.timed.as_ref().and_then(|timed| {
      // Until the first frame lands there is no origin to measure from, and an
      // annotation drawn in that window was there from the recording's start.
      timed
        .clock
        .timestamp_us(at)
        .map(millis)
        .or_else(|| timed.clock.is_live().then_some(0))
    });
    let replay_shown_at_us = self
      .replay
      .as_ref()
      .and_then(|replay| replay.shown_at_us(at));
    self.annotations.push(LiveAnnotation {
      annotation,
      replay_shown_at_us,
      shown_at_ms,
    });
  }

  /// Takes the newest annotation off the screen, reporting whether there was
  /// one. The clip it earned is kept: the annotation was visible for exactly
  /// that long.
  fn remove_last(&mut self, at: Instant) -> bool {
    let Some(live) = self.annotations.pop() else {
      return false;
    };
    if let Some(timed) = self.timed.as_mut() {
      timed.close(&live, at);
    }
    if let Some(replay) = self.replay.as_mut() {
      replay.close(&live, at);
    }
    true
  }

  /// Takes every annotation off the screen: a clear, or the overlay being
  /// dismissed.
  fn clear(&mut self, at: Instant) {
    let annotations = std::mem::take(&mut self.annotations);
    if let Some(replay) = self.replay.as_mut() {
      for live in &annotations {
        replay.close(live, at);
      }
    }
    let Some(timed) = self.timed.as_mut() else {
      return;
    };
    for live in &annotations {
      timed.close(live, at);
    }
  }

  /// Starts timing against a recording. Annotations already on screen belong to
  /// it from its first frame.
  fn start(&mut self, origin: Arc<OnceLock<Instant>>, source: CursorSource) {
    for live in &mut self.annotations {
      live.shown_at_ms = Some(0);
    }
    self.timed = Some(Timed {
      clips: Vec::new(),
      clock: SidecarClock::new(origin),
      source,
    });
  }

  fn pause(&mut self, at: Instant) {
    if let Some(timed) = self.timed.as_mut() {
      timed.clock.pause(at);
    }
  }

  /// Annotations drawn during the pause enter the recording here, so their
  /// clips start in recording time rather than at the wall time of the stroke.
  fn resume(&mut self, at: Instant) {
    let Some(timed) = self.timed.as_mut() else {
      return;
    };
    timed.clock.resume(at);
    let Some(now) = timed.clock.timestamp_us(at).map(millis) else {
      return;
    };
    for live in &mut self.annotations {
      live.shown_at_ms.get_or_insert(now);
    }
  }

  /// Ends the recording and hands over its clips. The annotations stay on
  /// screen; the ones still visible are closed out at `stopped_at`.
  ///
  /// A closing phase that ran past the end of the recording is trimmed back to
  /// it: the offset [`Timed::close`] adds is only there when the recording has
  /// the room, and an annotation still on screen when recording stops has none
  /// at all.
  fn stop(&mut self, stopped_at: Instant) -> Vec<RecordingAnnotationClip> {
    let Some(mut timed) = self.timed.take() else {
      return Vec::new();
    };
    for live in &self.annotations {
      timed.close(live, stopped_at);
    }
    for live in &mut self.annotations {
      live.shown_at_ms = None;
    }
    if let Some(stopped_ms) = timed.clock.elapsed_us(stopped_at).map(millis) {
      for clip in &mut timed.clips {
        clip.end_ms = clip.end_ms.min(stopped_ms).max(clip.start_ms + 1);
      }
    }
    timed.clips
  }

  /// Drops the recording's clips. A cancelled recording leaves nothing
  /// behind, here included.
  fn cancel(&mut self) {
    self.timed = None;
    for live in &mut self.annotations {
      live.shown_at_ms = None;
    }
  }
}

const fn millis(micros: u64) -> u64 {
  micros / 1_000
}

static LIVE: LazyLock<Mutex<LiveAnnotations>> =
  LazyLock::new(|| Mutex::new(LiveAnnotations::default()));

/// A poisoned lock still holds an honest annotation list, and one panicking
/// stroke must not silently end annotating.
fn live() -> MutexGuard<'static, LiveAnnotations> {
  LIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Adds a completed stroke, in global logical desktop points. `at` is when the
/// stroke began rather than when it was finished: that is the moment the
/// annotation started appearing on screen, and where the pointer was when it
/// did.
pub(crate) fn add(annotation: Annotation, at: Instant) {
  live().add(annotation, at);
}

/// Undo: takes the newest annotation off the screen, closing out its clip when
/// a recording is running. Reports whether there was an annotation to take.
pub(crate) fn remove_last() -> bool {
  live().remove_last(Instant::now())
}

/// Removes every annotation, closing out their clips when a recording is
/// running.
pub(crate) fn clear() {
  live().clear(Instant::now());
}

/// The annotations on screen, for the overlay to draw.
pub(crate) fn annotations() -> Vec<Annotation> {
  live()
    .annotations
    .iter()
    .map(|live| live.annotation.clone())
    .collect()
}

/// The number the next counter dropped on the overlay takes. Counted rather
/// than remembered: clearing the screen starts the count again, and undo only
/// ever takes the newest annotation, so what is on screen is always 1..n.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(crate) fn next_counter_value() -> u32 {
  crate::editor::annotations::counter::next_counter_value(
    live().annotations.iter().map(|live| &live.annotation),
  )
}

/// Whether anything is on screen, which is what decides if the tray offers to
/// clear it.
pub(crate) fn has_annotations() -> bool {
  !live().annotations.is_empty()
}

/// The live-annotation sidecar. It owns no file and no thread: the annotations
/// are already in memory, so all a recording adds is a clock and a clip list.
pub(crate) struct AnnotationRecorder;

impl AnnotationRecorder {
  pub(crate) fn start(origin: Arc<OnceLock<Instant>>, source: CursorSource) -> Self {
    live().start(origin, source);
    Self
  }

  pub(crate) fn pause(&self, at: Instant) {
    live().pause(at);
  }

  pub(crate) fn resume(&self, at: Instant) {
    live().resume(at);
  }

  pub(crate) fn stop(self, stopped_at: Instant) -> Vec<RecordingAnnotationClip> {
    live().stop(stopped_at)
  }

  pub(crate) fn cancel(self) {
    live().cancel();
  }
}

#[cfg(test)]
mod tests;
