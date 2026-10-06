// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::screen_stream;
use crate::recording::SystemAudioSelection;

#[derive(Default)]
pub(super) struct SystemAudioStreams {
  pub all: Option<WatchedStream>,
  pub selected: Option<WatchedStream>,
  pub video_captures_all: bool,
}

impl SystemAudioStreams {
  pub(super) async fn start(&self) -> Result<(), String> {
    if let Some(selected) = &self.selected {
      selected
        .stream
        .start()
        .await
        .map_err(|error| error.to_string())?;
    }
    if let Some(all) = &self.all {
      if let Err(error) = all.stream.start().await {
        if let Some(selected) = &self.selected {
          selected.stream.stop_with_ch(|_| {});
        }
        return Err(error.to_string());
      }
    }
    Ok(())
  }

  pub(super) fn stop(&self) {
    for watched in self.selected.iter().chain(&self.all) {
      watched.stream.stop_with_ch(|_| {});
    }
  }

  pub(super) fn append_to(self, streams: &mut Vec<WatchedStream>) {
    streams.extend(self.all);
    streams.extend(self.selected);
  }
}

pub(super) fn create(
  selection: &SystemAudioSelection,
  content: Option<&sc::ShareableContent>,
  output: Option<&arc::R<ScreenOutput>>,
  queue: &dispatch::Queue,
  watch: &StreamWatch,
  video_can_capture_all: bool,
) -> Result<SystemAudioStreams, String> {
  let captures_selected = selection.enabled && !selection.application_ids.is_empty();
  let captures_all = selection.enabled && !captures_selected;
  let video_captures_all = captures_all && video_can_capture_all;
  if !selection.enabled {
    return Ok(SystemAudioStreams::default());
  }

  let content = content.expect("audio has content");
  let displays = content.displays();
  let display = displays
    .first()
    .ok_or_else(|| "No monitor is available for audio capture".to_owned())?;
  let output = output.expect("content has output");
  let all = if captures_all && !video_captures_all {
    Some(screen_stream::create_all_audio(
      screen_stream::AllAudioStreamRequest {
        content,
        display,
        output,
        queue,
        watch,
      },
    )?)
  } else {
    None
  };
  let selected = if captures_selected {
    let filter = application_audio_filter(content, display, &selection.application_ids)?;
    let mut cfg = sc::StreamCfg::new();
    cfg.set_captures_audio(true);
    configure_system_audio(&mut cfg);
    Some(
      StreamRecipe::new(&filter, cfg, queue)
        .output(RecipeOutput::Screen(output.clone()), sc::OutputType::Audio)
        .build(watch)?,
    )
  } else {
    None
  };

  Ok(SystemAudioStreams {
    all,
    selected,
    video_captures_all,
  })
}
