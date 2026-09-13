// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The envelopes behind the audio-only preview's native bars.
//!
//! A recording with no video still has to show something, and that something
//! is drawn by the preview surface rather than by a canvas in the webview.
//! This module turns the window's enabled tracks into the bounded payload the
//! native side samples each bar from.

use super::*;

#[cfg(any(test, target_os = "windows"))]
mod buckets;
#[cfg(target_os = "windows")]
pub(crate) use buckets::bucket_levels;

/// One row per track, one column per envelope point. Both are clamped here so
/// the native texture is bounded whatever the recording holds.
pub(crate) const MAX_RIBBON_TRACKS: usize = 4;
pub(crate) const MAX_RIBBON_POINTS: usize = 2_048;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioVisualizerTrack {
  #[serde(default)]
  pub gain_decibels: f64,
  #[serde(default)]
  pub stream_index: usize,
  pub waveform: Vec<f32>,
}

/// The sampling payload: `tracks` rows of `points` samples, row-major.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) struct AudioRibbonEnvelopes {
  pub gains: Vec<f32>,
  pub points: u32,
  pub samples: Vec<f32>,
  pub tracks: u32,
}

/// The same curve the lanes and the meter apply, so one slider moves them all.
fn decibel_gain(decibels: f64) -> f32 {
  if decibels <= -60.0 {
    0.0
  } else {
    10.0_f64.powf(decibels / 20.0) as f32
  }
}

/// Resamples every track onto one point count so the shader can read the rows
/// of a single texture with one coordinate.
pub(crate) fn envelopes(tracks: &[AudioVisualizerTrack]) -> AudioRibbonEnvelopes {
  let mut tracks = tracks
    .iter()
    .filter(|track| !track.waveform.is_empty())
    .collect::<Vec<_>>();
  // Row order follows the recording rather than the order the window happened
  // to send, and a track named twice occupies one row.
  tracks.sort_by_key(|track| track.stream_index);
  tracks.dedup_by_key(|track| track.stream_index);
  tracks.truncate(MAX_RIBBON_TRACKS);
  let points = tracks
    .iter()
    .map(|track| track.waveform.len())
    .max()
    .unwrap_or(0)
    .min(MAX_RIBBON_POINTS);
  if points == 0 {
    return AudioRibbonEnvelopes::default();
  }
  let mut samples = vec![0.0_f32; points * tracks.len()];
  for (row, track) in tracks.iter().enumerate() {
    let source = track.waveform.len();
    for point in 0..points {
      // Nearest neighbour: the envelope is already a peak per bucket, so
      // averaging neighbours would only quieten it.
      let index = (point * source) / points;
      let value = track.waveform.get(index).copied().unwrap_or(0.0);
      samples[row * points + point] = if value.is_finite() {
        value.clamp(0.0, 1.0)
      } else {
        0.0
      };
    }
  }
  AudioRibbonEnvelopes {
    gains: tracks
      .iter()
      .map(|track| decibel_gain(track.gain_decibels))
      .collect(),
    points: points as u32,
    samples,
    tracks: tracks.len() as u32,
  }
}

/// Hands the surface the enabled tracks' envelopes and gains. An empty list
/// clears the bars, which is what every layout with a video pane sends.
#[tauri::command]
pub async fn set_recording_audio_visualizer(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  artifact_id: u64,
  tracks: Vec<AudioVisualizerTrack>,
) -> Result<(), String> {
  let manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  let uploaded = envelopes(&tracks);
  if manager.artifact_id != Some(artifact_id) {
    return Ok(());
  }
  let Some(surface) = manager
    .sources
    .as_ref()
    .and_then(|sources| sources.preview_surface.as_ref())
  else {
    return Ok(());
  };
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  surface.set_audio_ribbon(&uploaded);
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let _ = (surface, uploaded);
  Ok(())
}

/// Native playback presents this source position using the same sample clock
/// as video. The webview only uploads envelopes; it never drives this position.
pub(super) fn present_audio_position(sources: &PlayerSources, position_ms: u64) {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let Some(surface) = &sources.preview_surface {
    let ratio = source_ratio(position_ms, sources.duration_ms);
    surface.set_audio_ribbon_playhead(ratio);
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let _ = (sources, position_ms);
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn source_ratio(position_ms: u64, duration_ms: u64) -> f64 {
  if duration_ms == 0 {
    0.0
  } else {
    position_ms.min(duration_ms) as f64 / duration_ms as f64
  }
}

#[cfg(test)]
#[path = "audio_visualizer_tests.rs"]
mod tests;
