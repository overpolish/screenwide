// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The files the preview can play the microphone tracks from, made by
//! Reduce noise and Vocal cleanup, how Auto volume levels each of them, and
//! which is heard.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::RecordingPreviewPlayerState;
use crate::editor::speech::auto_volume::{self, Leveling};
use crate::editor::speech::heard::{self, Processing};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PreviewProcessing {
  /// Each file made of a track. Every one is decoded beside the track, so a
  /// switch changes which is heard without a pause.
  pub files: Vec<(usize, Processing, PathBuf)>,
  /// How each track is heard, leaving out those heard as recorded.
  pub heard: Vec<(usize, Processing)>,
  /// How Auto volume levels each measured source of a track, the recording
  /// among them. Each is decoded leveled too, beside the source itself.
  pub levelings: Vec<(usize, Processing, Leveling)>,
  /// The tracks heard leveled.
  pub leveled: Vec<usize>,
}

impl PreviewProcessing {
  /// What the project in `project_folder` says of its `streams`.
  pub(crate) fn for_project(project_folder: &Path, streams: &[usize]) -> Self {
    Self {
      files: heard::files(project_folder, streams),
      heard: heard::heard(project_folder, streams),
      levelings: streams
        .iter()
        .flat_map(|&stream| {
          auto_volume::levelings(project_folder, stream)
            .into_iter()
            .map(move |(processing, leveling)| (stream, processing, leveling))
        })
        .collect(),
      leveled: auto_volume::heard_filters(project_folder, streams)
        .into_iter()
        .map(|(stream, _)| stream)
        .collect(),
    }
  }
}

/// Tells the preview of the recording open, if one is running, which files
/// are made now and which is heard.
pub(crate) fn refresh(app: &AppHandle, processing: PreviewProcessing) {
  let Some(state) = app.try_state::<RecordingPreviewPlayerState>() else {
    return;
  };
  let Ok(manager) = state.0.lock() else {
    return;
  };
  if let Some(sources) = &manager.sources {
    *sources
      .processing
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = processing;
  }
}
