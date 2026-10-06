// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Live annotations on the replay buffer's timeline.
//!
//! A recording closes each annotation into a clip once and keeps it. The
//! replay buffer cannot know which stretch will be saved, so it keeps when
//! each annotation was on screen for as long as a clip could still reach it,
//! and cuts the clips only when one is saved.

// Only the macOS replay buffer drives this so far.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::time::Duration;

use super::*;

/// An annotation that has left the screen, on the replay's timeline.
struct Gone {
  annotation: Annotation,
  gone_us: u64,
  shown_us: u64,
}

pub(super) struct ReplayTimed {
  clock: SidecarClock,
  gone: Vec<Gone>,
  horizon_us: u64,
  source: CursorSource,
}

impl ReplayTimed {
  /// Replay time at `at`. Before the first frame there is no origin, and an
  /// annotation drawn in that window was there from the buffer's start.
  pub(super) fn shown_at_us(&self, at: Instant) -> Option<u64> {
    self
      .clock
      .timestamp_us(at)
      .or_else(|| self.clock.is_live().then_some(0))
  }

  pub(super) fn close(&mut self, live: &LiveAnnotation, at: Instant) {
    let (Some(shown_us), Some(gone_us)) = (live.replay_shown_at_us, self.clock.elapsed_us(at))
    else {
      return;
    };
    self.gone.push(Gone {
      annotation: live.annotation.clone(),
      gone_us,
      shown_us,
    });
    let horizon_us = self.horizon_us;
    self
      .gone
      .retain(|gone| gone.gone_us.saturating_add(horizon_us) >= gone_us);
  }
}

impl LiveAnnotations {
  fn start_replay(
    &mut self,
    origin: Arc<OnceLock<Instant>>,
    source: CursorSource,
    horizon: Duration,
  ) {
    for live in &mut self.annotations {
      live.replay_shown_at_us = Some(0);
    }
    self.replay = Some(ReplayTimed {
      clock: SidecarClock::new(origin),
      gone: Vec::new(),
      horizon_us: u64::try_from(horizon.as_micros()).unwrap_or(u64::MAX),
      source,
    });
  }

  fn stop_replay(&mut self) {
    self.replay = None;
    for live in &mut self.annotations {
      live.replay_shown_at_us = None;
    }
  }

  /// The annotation clips of the replay stretch `start_us..=end_us`, in that
  /// stretch's own time. One already on screen when it starts begins at zero;
  /// one still on screen when it ends runs to its end.
  fn replay_clips(&self, start_us: u64, end_us: u64) -> Vec<RecordingAnnotationClip> {
    let Some(replay) = &self.replay else {
      return Vec::new();
    };
    let length_ms = millis(end_us.saturating_sub(start_us));
    let gone = replay
      .gone
      .iter()
      .map(|gone| (&gone.annotation, gone.shown_us, Some(gone.gone_us)));
    let on_screen = self.annotations.iter().filter_map(|live| {
      live
        .replay_shown_at_us
        .map(|shown_us| (&live.annotation, shown_us, None))
    });
    gone
      .chain(on_screen)
      .filter(|(_, shown_us, gone_us)| {
        *shown_us < end_us && gone_us.is_none_or(|gone_us| gone_us > start_us)
      })
      .filter_map(|(annotation, shown_us, gone_us)| {
        let start_ms = millis(shown_us.saturating_sub(start_us));
        let gone_ms = gone_us.map_or(length_ms, |gone_us| {
          millis(gone_us.min(end_us).saturating_sub(start_us))
        });
        let mut clip = annotation_clip(annotation, &replay.source, start_ms, gone_ms)?;
        clip.end_ms = clip.end_ms.min(length_ms).max(clip.start_ms + 1);
        Some(clip)
      })
      .collect()
  }
}

/// Live annotations timed against the replay buffer. Owns no file and no
/// thread; dropping it stops the timing.
pub(crate) struct ReplayAnnotationRecorder;

impl ReplayAnnotationRecorder {
  pub(crate) fn start(
    origin: Arc<OnceLock<Instant>>,
    source: CursorSource,
    horizon: Duration,
  ) -> Self {
    live().start_replay(origin, source, horizon);
    Self
  }

  pub(crate) fn clips(&self, start_us: u64, end_us: u64) -> Vec<RecordingAnnotationClip> {
    live().replay_clips(start_us, end_us)
  }
}

impl Drop for ReplayAnnotationRecorder {
  fn drop(&mut self) {
    live().stop_replay();
  }
}

#[cfg(test)]
mod tests;
