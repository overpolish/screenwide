// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tauri::command]
pub async fn play_recording_preview(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  playback_end_ms: Option<u64>,
  playback_ranges: Option<Vec<RecordingPreviewPlaybackRange>>,
  session_id: u64,
  sound_track: Option<sound_track::PreviewSoundTrackRequest>,
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
    validate_playback_ranges(&playback_ranges, duration_ms)?;
    if let Some(sources) = manager.sources.as_ref() {
      let laid_out = sound_track.map_or_else(Default::default, |request| {
        sound_track::PreviewSoundTrack::new(request, &sources.project, &sources.audio_tracks)
      });
      *sources
        .sound_track
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = laid_out;
    }
    manager.is_playing = true;
    manager.playback_end_ms = playback_end_ms;
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

/// Ranges play in the order given, which a reordered timeline leaves out of
/// source order; no two may play the same source.
fn validate_playback_ranges(
  ranges: &[RecordingPreviewPlaybackRange],
  duration_ms: u64,
) -> Result<(), String> {
  let mut spans: Vec<(u64, u64)> = ranges
    .iter()
    .map(|range| (range.source_start_ms, range.source_end_ms))
    .collect();
  spans.sort_unstable();
  let valid = ranges.iter().all(|range| {
    range.source_start_ms < range.source_end_ms
      && range.source_end_ms <= duration_ms
      && range.playback_rate.is_finite()
      && (0.25..=4.0).contains(&range.playback_rate)
  }) && spans.windows(2).all(|pair| pair[0].1 <= pair[1].0);
  valid
    .then_some(())
    .ok_or_else(|| "The recording preview playback ranges are invalid".to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn playback_ranges_must_be_non_empty_non_overlapping_source_intervals() {
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
    let reordered = [valid[1], valid[0]];
    assert!(validate_playback_ranges(&reordered, 4_000).is_ok());
    let overlapping = [
      valid[0],
      RecordingPreviewPlaybackRange {
        source_start_ms: 1_000,
        source_end_ms: 3_000,
        playback_rate: 1.0,
      },
    ];
    assert!(validate_playback_ranges(&overlapping, 4_000).is_err());
  }
}
