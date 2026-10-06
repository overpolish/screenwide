// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An audio recording's still: the editor's ribbon over the ten seconds
//! around its middle.

use super::*;
use crate::editor::preview_platform::{ribbon_still, RIBBON_BAR_PITCH_POINTS};
use crate::editor::recording_preview_player::audio_visualizer::{
  AudioRibbonEnvelopes, MAX_RIBBON_TRACKS,
};

const SPAN_MS: u64 = 10_000;
/// The preview's width at 16:9, drawn at the ribbon's Retina scale.
const SIZE: (u32, u32) = (PREVIEW_WIDTH, PREVIEW_WIDTH * 9 / 16);
const SCALE: f32 = 2.0;
/// The envelope points `bucket_levels` averages into one bar.
const POINTS_PER_BAR: u32 = 4;

/// Draws the ribbon in white with coverage as alpha, for the card to tint.
pub(super) fn write(movie: &Path, duration_ms: Option<u64>, target: &Path) -> Result<(), String> {
  let duration_ms = duration_ms.ok_or("The recording's length is unknown")?;
  let span_ms = SPAN_MS.min(duration_ms).max(1);
  let window = ((duration_ms - span_ms) / 2, span_ms);
  let bars = (SIZE.0 as f32 / (RIBBON_BAR_PITCH_POINTS * SCALE)).ceil() as u32;
  let points = (bars * POINTS_PER_BAR) as usize;
  let mut samples = Vec::new();
  let mut tracks = 0;
  for track in media_preview::inspect_audio_tracks(movie)?
    .iter()
    .take(MAX_RIBBON_TRACKS)
  {
    samples.extend(media_preview::peaks(
      movie,
      track.stream_index,
      &track.label,
      window,
      points,
    )?);
    tracks += 1;
  }
  let envelopes = AudioRibbonEnvelopes {
    gains: vec![1.0; tracks],
    points: points as u32,
    samples,
    tracks: tracks as u32,
  };
  let pixels = ribbon_still(&envelopes, SIZE, SCALE, [1.0, 1.0, 1.0, 1.0])?;
  let image = image::RgbaImage::from_raw(SIZE.0, SIZE.1, pixels)
    .ok_or_else(|| "The ribbon is not the size it was drawn at".to_owned())?;
  write_atomically(target, false, |partial| {
    image
      .save_with_format(partial, image::ImageFormat::Png)
      .map_err(|error| error.to_string())
  })
}
