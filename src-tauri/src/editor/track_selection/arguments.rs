// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The FFmpeg arguments that put a [`TrackSelection`] into a file.

use std::path::Path;

use super::{AudioLayout, TrackSelection, MIXDOWN_BITRATE};

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
    if self.stream_indices.is_empty() {
      return vec!["-an".to_owned()];
    }

    let processes = self.processes();
    let opened = self.opened_sources(input);
    match layout {
      // One track needs no summing, so it crosses untouched rather than being
      // decoded and re-encoded for the sake of passing through a filter.
      AudioLayout::Mixdown if self.stream_indices.len() == 1 && !processes => vec![
        "-map".to_owned(),
        format!("{input}:a:{}", self.stream_indices[0]),
        "-c:a".to_owned(),
        "copy".to_owned(),
      ],
      AudioLayout::Mixdown if !processes => {
        let inputs: String = self
          .stream_indices
          .iter()
          .map(|index| self.source(input, *index))
          .collect();
        vec![
          "-filter_complex".to_owned(),
          format!(
            "{inputs}amix=inputs={}:normalize=0[mix]",
            self.stream_indices.len()
          ),
          "-map".to_owned(),
          "[mix]".to_owned(),
          "-c:a".to_owned(),
          "aac".to_owned(),
          "-b:a".to_owned(),
          MIXDOWN_BITRATE.to_owned(),
        ]
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

        vec![
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
          "-c:a".to_owned(),
          "aac".to_owned(),
          "-b:a".to_owned(),
          MIXDOWN_BITRATE.to_owned(),
        ]
      }
      AudioLayout::SeparateTracks if !processes => {
        let mut args = Vec::with_capacity(self.stream_indices.len() * 2 + 2);
        for index in &self.stream_indices {
          args.push("-map".to_owned());
          args.push(format!("{input}:a:{index}"));
        }
        args.push("-c:a".to_owned());
        args.push("copy".to_owned());

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
        args.extend([
          "-c:a".to_owned(),
          "aac".to_owned(),
          "-b:a".to_owned(),
          MIXDOWN_BITRATE.to_owned(),
        ]);
        args
      }
    }
  }
}

/// `path` written as a value inside a filter graph. It is escaped twice, as
/// FFmpeg reads it twice: once as the graph, then as the filter's options.
pub(super) fn filter_path(path: &Path) -> String {
  let escape = |text: &str, special: &[char]| {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
      if special.contains(&character) {
        escaped.push('\\');
      }
      escaped.push(character);
    }
    escaped
  };
  let option = escape(&path.to_string_lossy(), &['\\', '\'', ':']);
  escape(&option, &['\\', '\'', '[', ']', ',', ';'])
}
