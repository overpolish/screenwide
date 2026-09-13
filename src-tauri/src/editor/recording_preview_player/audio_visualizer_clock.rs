// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The display link reads the same timestamped output clock as the player.

use super::{audio::clock::AudioClock, PlayerSources, RecordingPreviewPlaybackRange};
use std::sync::{
  atomic::{AtomicBool, AtomicU64, Ordering},
  Arc,
};
use std::time::Instant;

/// A suspended display link must not leave background playback's resume point stale.
pub(super) struct DisplayClock {
  started: Instant,
  last_refresh_ms: Arc<AtomicU64>,
}

impl DisplayClock {
  pub(super) fn active(&self) -> bool {
    (self.started.elapsed().as_millis() as u64)
      .saturating_sub(self.last_refresh_ms.load(Ordering::Acquire))
      < 100
  }
}

pub(super) fn install(
  sources: &PlayerSources,
  clock: &Arc<AudioClock>,
  ranges: &[RecordingPreviewPlaybackRange],
  playback_rate: f64,
  cancelled: &Arc<AtomicBool>,
  position_ms: &Arc<AtomicU64>,
  start_ms: u64,
) -> Option<DisplayClock> {
  #[cfg(target_os = "macos")]
  if let Some(surface) = &sources.preview_surface {
    let clock = Arc::clone(clock);
    let ranges = ranges.to_vec();
    let cancelled = Arc::clone(cancelled);
    let position_ms = Arc::clone(position_ms);
    let duration = sources.duration_ms as f64;
    let started = Instant::now();
    let last_refresh_ms = Arc::new(AtomicU64::new(0));
    let refreshed = Arc::clone(&last_refresh_ms);
    surface.set_audio_ribbon_clock(move |ahead| {
      refreshed.store(started.elapsed().as_millis() as u64, Ordering::Release);
      let source = if cancelled.load(Ordering::Acquire) {
        position_ms.load(Ordering::Acquire) as f64
      } else {
        let source = source_at(&ranges, clock.seconds_ahead(ahead), playback_rate, start_ms);
        // Resume starts at the frame selected by the display, rather than a
        // separate worker poll that could be a buffer ahead of that frame.
        position_ms.store(source.round() as u64, Ordering::Release);
        source
      };
      if duration > 0.0 {
        (source / duration).clamp(0.0, 1.0)
      } else {
        0.0
      }
    });
    return Some(DisplayClock {
      started,
      last_refresh_ms,
    });
  }
  let _ = (
    sources,
    clock,
    ranges,
    playback_rate,
    cancelled,
    position_ms,
    start_ms,
  );
  None
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn source_at(
  ranges: &[RecordingPreviewPlaybackRange],
  elapsed: f64,
  global_rate: f64,
  start: u64,
) -> f64 {
  let mut remaining_ms = elapsed * 1000.0;
  for range in ranges {
    let rate = range.playback_rate * global_rate;
    // Match the output lengths used to construct the player's audio filters.
    let duration_ms = (range.duration_ms() as f64 / rate).ceil();
    if remaining_ms < duration_ms {
      return (range.source_start_ms as f64 + remaining_ms * rate).min(range.source_end_ms as f64);
    }
    remaining_ms -= duration_ms;
  }
  ranges.last().map_or(start, |range| range.source_end_ms) as f64
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn suspended_display_links_release_position_updates_to_the_worker() {
    let clock = DisplayClock {
      started: Instant::now() - std::time::Duration::from_millis(200),
      last_refresh_ms: Arc::new(AtomicU64::new(0)),
    };
    assert!(!clock.active());
    clock.last_refresh_ms.store(
      clock.started.elapsed().as_millis() as u64,
      Ordering::Release,
    );
    assert!(clock.active());
  }

  #[test]
  fn samples_fractional_positions_and_rate_changes_across_cuts() {
    let ranges = [
      RecordingPreviewPlaybackRange {
        source_start_ms: 1000,
        source_end_ms: 2000,
        playback_rate: 2.0,
      },
      RecordingPreviewPlaybackRange {
        source_start_ms: 4000,
        source_end_ms: 5000,
        playback_rate: 0.5,
      },
    ];
    assert_eq!(source_at(&ranges, 0.0, 1.0, 1000), 1000.0);
    assert!((source_at(&ranges, 1.0 / 60.0, 1.0, 1000) - 1033.333333333).abs() < 1e-6);
    assert_eq!(source_at(&ranges, 0.5, 1.0, 1000), 4000.0);
    assert_eq!(source_at(&ranges, 1.0, 1.0, 1000), 4250.0);
    assert_eq!(source_at(&ranges, 0.25, 2.0, 1000), 4000.0);
    assert_eq!(source_at(&ranges, 5.0, 1.0, 1000), 5000.0);
  }
}
