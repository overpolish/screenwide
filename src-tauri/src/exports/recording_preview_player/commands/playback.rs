// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tauri::command]
pub async fn play_recording_preview(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  playback_end_ms: Option<u64>,
  playback_ranges: Option<Vec<RecordingPreviewPlaybackRange>>,
  playback_rate: Option<f64>,
  session_id: u64,
  start_position_ms: Option<u64>,
) -> Result<(), String> {
  let worker = {
    let mut manager = state
      .0
      .lock()
      .map_err(|_| "The recording preview player is unavailable".to_owned())?;
    manager.require_session(session_id)?;
    let duration_ms = manager
      .sources
      .as_ref()
      .map_or(0, |sources| sources.duration_ms);
    let playback_ranges = playback_ranges.unwrap_or_default();
    let playback_rate = validate_playback_rate(playback_rate.unwrap_or(1.0))?;
    validate_playback_ranges(&playback_ranges, duration_ms)?;
    manager.is_playing = true;
    manager.playback_end_ms = playback_end_ms;
    manager.playback_rate = playback_rate;
    manager.playback_ranges = playback_ranges;
    manager.take_worker()
  };
  if let Some(worker) = worker {
    cancel_off_thread(worker).await?;
  }
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  if !manager.is_playing {
    return Ok(());
  }
  let duration_ms = manager
    .sources
    .as_ref()
    .map_or(0, |sources| sources.duration_ms);
  manager.position_ms = start_position_ms.map_or(manager.position_ms, |position_ms| {
    decodable_position(position_ms, duration_ms)
  });
  manager.restart(PlaybackMode::Playing)
}

pub(super) fn validate_playback_rate(rate: f64) -> Result<f64, String> {
  (rate.is_finite() && (0.25..=4.0).contains(&rate))
    .then_some(rate)
    .ok_or_else(|| "The preview playback rate must be between 0.25x and 4x".to_owned())
}

fn validate_playback_ranges(
  ranges: &[RecordingPreviewPlaybackRange],
  duration_ms: u64,
) -> Result<(), String> {
  let valid = ranges.iter().enumerate().all(|(index, range)| {
    range.source_start_ms < range.source_end_ms
      && range.source_end_ms <= duration_ms
      && range.playback_rate.is_finite()
      && (0.25..=4.0).contains(&range.playback_rate)
      && index
        .checked_sub(1)
        .is_none_or(|previous| ranges[previous].source_end_ms <= range.source_start_ms)
  });
  valid
    .then_some(())
    .ok_or_else(|| "The recording preview playback ranges are invalid".to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn playback_ranges_must_be_ordered_non_empty_source_intervals() {
    let valid = [
      RecordingPreviewPlaybackRange {
        source_start_ms: 0,
        source_end_ms: 2_000,
        playback_rate: 1.0,
      },
      RecordingPreviewPlaybackRange {
        source_start_ms: 3_000,
        source_end_ms: 4_000,
        playback_rate: 1.0,
      },
    ];
    assert!(validate_playback_ranges(&valid, 4_000).is_ok());
    assert!(validate_playback_ranges(&valid, 3_999).is_err());
    assert!(validate_playback_ranges(
      &[RecordingPreviewPlaybackRange {
        source_start_ms: 2_000,
        source_end_ms: 2_000,
        playback_rate: 1.0,
      }],
      4_000
    )
    .is_err());
  }

  #[test]
  fn playback_rate_accepts_safe_finite_range() {
    assert_eq!(validate_playback_rate(0.25).unwrap(), 0.25);
    assert_eq!(validate_playback_rate(4.0).unwrap(), 4.0);
    assert!(validate_playback_rate(0.2).is_err());
    assert!(validate_playback_rate(f64::NAN).is_err());
  }
}
