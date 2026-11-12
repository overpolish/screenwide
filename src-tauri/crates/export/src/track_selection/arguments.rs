// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The FFmpeg arguments that put a [`TrackSelection`] into a file.

use super::{AudioEncoding, AudioLayout, TrackSelection};

impl TrackSelection {
  /// The FFmpeg arguments that put this selection into the output.
  ///
  /// Video is never among them: nothing here touches the picture. Callers pair
  /// these with either a stream copy or the requested compression encode.
  pub fn audio_args(&self, layout: AudioLayout) -> Vec<String> {
    self.audio_args_from(layout, 0)
  }

  /// The same mapping when video and recorded audio are separate FFmpeg
  /// inputs, such as camera export processing.
  pub fn audio_args_from(&self, layout: AudioLayout, input: usize) -> Vec<String> {
    self.encoded_audio_args(layout, input, AudioEncoding::Aac)
  }

  /// The mapping with the audio encoded as `encoding`.
  pub fn encoded_audio_args(
    &self,
    layout: AudioLayout,
    input: usize,
    encoding: AudioEncoding,
  ) -> Vec<String> {
    if self.stream_indices.is_empty() {
      return vec!["-an".to_owned()];
    }

    let processes = self.processes();
    let opened = self.opened_sources(input);
    match layout {
      // One track needs no summing, so it crosses untouched rather than being
      // decoded and re-encoded for the sake of passing through a filter.
      AudioLayout::Mixdown if self.stream_indices.len() == 1 && !processes => {
        let mut args = vec![
          "-map".to_owned(),
          format!("{input}:a:{}", self.stream_indices[0]),
        ];
        args.extend(encoding.codec_args(true));
        args
      }
      AudioLayout::Mixdown if !processes => {
        let inputs: String = self
          .stream_indices
          .iter()
          .map(|index| self.source(input, *index))
          .collect();
        let mut args = vec![
          "-filter_complex".to_owned(),
          format!(
            "{inputs}amix=inputs={}:normalize=0[mix]",
            self.stream_indices.len()
          ),
          "-map".to_owned(),
          "[mix]".to_owned(),
        ];
        args.extend(encoding.codec_args(false));
        args
      }
      AudioLayout::Mixdown => {
        let mut filters = opened;
        let mut inputs = String::new();
        for (position, index) in self.stream_indices.iter().enumerate() {
          filters.push_str(&format!(
            "{}volume={}dB[track{position}];",
            self.source(input, *index),
            self.volume(*index)
          ));
          inputs.push_str(&format!("[track{position}]"));
        }

        let mut args = vec![
          "-filter_complex".to_owned(),
          // `normalize=0` is the whole point: amix divides by the number of
          // inputs by default, so including a second track would make the
          // first one quieter than it was recorded. A person toggling
          // microphone on must not hear the system audio drop by half.
          format!(
            "{filters}{inputs}amix=inputs={}:normalize=0[mix]",
            self.stream_indices.len()
          ),
          "-map".to_owned(),
          "[mix]".to_owned(),
        ];
        args.extend(encoding.codec_args(false));
        args
      }
      AudioLayout::SeparateTracks if !processes => {
        let mut args = Vec::with_capacity(self.stream_indices.len() * 2 + 2);
        for index in &self.stream_indices {
          args.push("-map".to_owned());
          args.push(format!("{input}:a:{index}"));
        }
        args.extend(encoding.codec_args(true));

        args
      }
      AudioLayout::SeparateTracks => {
        let mut filters = opened;
        let mut args = Vec::new();
        for (position, index) in self.stream_indices.iter().enumerate() {
          filters.push_str(&format!(
            "{}volume={}dB[track{position}];",
            self.source(input, *index),
            self.volume(*index)
          ));
          args.extend(["-map".to_owned(), format!("[track{position}]")]);
        }
        filters.pop();
        args.splice(0..0, ["-filter_complex".to_owned(), filters]);
        args.extend(encoding.codec_args(false));
        args
      }
    }
  }
}
