// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which recorded audio tracks a derived file carries, and how they are laid
//! out in it.
//!
//! The export path keeps every included track as its own track
//! ([`AudioLayout::SeparateTracks`]) so system audio and a voice-over can still
//! be balanced, soloed or muted afterwards. Collapsing them into one is an
//! explicit export option.

use std::path::PathBuf;

use super::{AudioTrackVolume, RecordingAudioTrack};
use arguments::filter_path;

/// The bitrate the mixdown is encoded at. Summing tracks means decoding them,
/// so this is the one place in the app that re-encodes audio; generous enough
/// that the mix is not what a person hears a problem in.
const MIXDOWN_BITRATE_BPS: u64 = 192_000;
const MIXDOWN_BITRATE: &str = "192k";

/// How the selected tracks appear in the file being produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioLayout {
  /// Every selected track summed into a single encoded track when the user
  /// enables “Collapse audio tracks”.
  Mixdown,
  /// Every selected track kept as its own stream-copied track. What an export
  /// writes unless the user asks for the tracks to be collapsed.
  ///
  /// Written and tested ahead of the export path that will use it, hence the
  /// allowance: the point of it existing now is that the mixdown below cannot
  /// quietly become the definition of what saving a recording does.
  #[allow(dead_code)]
  SeparateTracks,
}

/// The recorded audio tracks a derived file should carry, in recording order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrackSelection {
  stream_indices: Vec<usize>,
  volumes: Vec<(usize, i16)>,
  /// Tracks played from their cleaned file in place of the recording's.
  cleaned: Vec<(usize, PathBuf)>,
  /// Filters each listed track is played through before its volume, as Auto
  /// volume levels it.
  filters: Vec<(usize, String)>,
}

impl TrackSelection {
  pub(crate) fn stream_indices(&self) -> &[usize] {
    &self.stream_indices
  }

  pub(crate) fn volume_decibels(&self, stream_index: usize) -> i16 {
    self.volume(stream_index)
  }

  /// Reads a selection from what the window's toggle rows are set to.
  ///
  /// Indices that name no track in this recording are dropped, and the rest
  /// are put back into the recording's own order: the window sends the rows it
  /// is showing, and a stale or reordered row must not become a mapping FFmpeg
  /// would refuse or, worse, one that quietly maps the wrong track.
  pub fn new(tracks: &[RecordingAudioTrack], enabled: &[usize]) -> Self {
    Self::with_volumes(tracks, enabled, &[]).expect("zero-decibel track volumes are valid")
  }

  pub fn with_volumes(
    tracks: &[RecordingAudioTrack],
    enabled: &[usize],
    volumes: &[AudioTrackVolume],
  ) -> Result<Self, String> {
    if volumes
      .iter()
      .any(|volume| !(-60..=12).contains(&volume.decibels))
    {
      return Err("Audio volume must be between -60 dB and +12 dB".to_owned());
    }
    let stream_indices = tracks
      .iter()
      .map(|track| track.stream_index)
      .filter(|index| enabled.contains(index))
      .collect::<Vec<_>>();
    Ok(Self {
      volumes: volumes
        .iter()
        .filter(|volume| stream_indices.contains(&volume.stream_index) && volume.decibels != 0)
        .map(|volume| (volume.stream_index, volume.decibels))
        .collect(),
      stream_indices,
      cleaned: Vec::new(),
      filters: Vec::new(),
    })
  }

  /// This selection with each of `cleaned`'s tracks it carries read from its
  /// cleaned file, which is lined up with the recording from its start.
  pub fn with_cleaned(mut self, cleaned: Vec<(usize, PathBuf)>) -> Self {
    self.cleaned = cleaned
      .into_iter()
      .filter(|(stream, _)| self.stream_indices.contains(stream))
      .collect();
    self
  }

  /// This selection with each of `filters`' tracks it carries played through
  /// its filters, after any cleaned file and before its volume.
  pub fn with_filters(mut self, filters: Vec<(usize, String)>) -> Self {
    self.filters = filters
      .into_iter()
      .filter(|(stream, _)| self.stream_indices.contains(stream))
      .collect();
    self
  }

  /// Whether a track has to be decoded and encoded again rather than copied.
  fn processes(&self) -> bool {
    !self.volumes.is_empty() || !self.cleaned.is_empty() || !self.filters.is_empty()
  }

  /// The filter label the `index`th track is read from, out of the `input`th
  /// input.
  pub(crate) fn source(&self, input: usize, index: usize) -> String {
    if self.filters.iter().any(|(stream, _)| *stream == index) {
      format!("[heard{index}]")
    } else {
      self.unfiltered(input, index)
    }
  }

  fn unfiltered(&self, input: usize, index: usize) -> String {
    if self.cleaned.iter().any(|(stream, _)| *stream == index) {
      format!("[clean{index}]")
    } else {
      format!("[{input}:a:{index}]")
    }
  }

  /// The filters that open each cleaned file and run each filtered track,
  /// out of the `input`th input, under the label [`Self::source`] gives it,
  /// each ending in `;`.
  pub(crate) fn opened_sources(&self, input: usize) -> String {
    let opened = self
      .cleaned
      .iter()
      .map(|(stream, path)| format!("amovie={}[clean{stream}];", filter_path(path)));
    let filtered = self.filters.iter().map(|(stream, filters)| {
      format!(
        "{}{filters}[heard{stream}];",
        self.unfiltered(input, *stream)
      )
    });
    opened.chain(filtered).collect()
  }

  fn volume(&self, stream_index: usize) -> i16 {
    self
      .volumes
      .iter()
      .find_map(|(index, decibels)| (*index == stream_index).then_some(*decibels))
      .unwrap_or(0)
  }

  /// Whether this selection leaves nothing out.
  pub fn covers(&self, tracks: &[RecordingAudioTrack]) -> bool {
    self.stream_indices.len() == tracks.len()
  }

  /// Whether this choice needs more than the ordinary all-stream remux.
  ///
  /// Leaving a track out always needs explicit mapping. A mixdown only needs
  /// processing when there is actually more than one input to sum; asking to
  /// collapse a lone track must not re-encode it for no audible difference.
  pub fn needs_processing(&self, tracks: &[RecordingAudioTrack], layout: AudioLayout) -> bool {
    !self.covers(tracks)
      || self.processes()
      || matches!(layout, AudioLayout::Mixdown) && self.stream_indices.len() > 1
  }

  /// The selected audio's expected encoded size.
  ///
  /// Recorded AAC is copied, so its configured bitrate is the useful estimate.
  /// A track found by inspection has no kind metadata; treating it like the larger
  /// system-audio stream is safer than promising a file that is too small.
  pub fn estimated_audio_bytes(
    &self,
    tracks: &[RecordingAudioTrack],
    layout: AudioLayout,
    duration_ms: u64,
  ) -> u64 {
    if self.stream_indices.is_empty() {
      return 0;
    }

    let bitrate = if (matches!(layout, AudioLayout::Mixdown) && self.stream_indices.len() > 1)
      || self.processes()
    {
      if matches!(layout, AudioLayout::SeparateTracks) {
        MIXDOWN_BITRATE_BPS.saturating_mul(self.stream_indices.len() as u64)
      } else {
        MIXDOWN_BITRATE_BPS
      }
    } else {
      tracks
        .iter()
        .filter(|track| self.stream_indices.contains(&track.stream_index))
        .map(|track| match track.kind {
          super::AudioTrackKind::Microphone => 128_000,
          super::AudioTrackKind::SystemAudio | super::AudioTrackKind::Unknown => 192_000,
        })
        .sum()
    };

    bitrate.saturating_mul(duration_ms) / 8_000
  }
}

mod arguments;
#[cfg(test)]
mod tests;
