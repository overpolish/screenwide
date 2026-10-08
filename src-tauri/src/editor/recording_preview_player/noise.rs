// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone tracks the preview plays cleaned of their noise.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::RecordingPreviewPlayerState;
use crate::editor::speech::noise;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PreviewNoise {
  /// Each track's cleaned file, where one has been made. It is decoded beside
  /// the track, so the switch changes which is heard without a pause.
  pub cleaned: Vec<(usize, PathBuf)>,
  /// The tracks heard cleaned.
  pub enabled: Vec<usize>,
}

impl PreviewNoise {
  /// What the project in `project_folder` says of its `streams`.
  pub(crate) fn for_project(project_folder: &Path, streams: &[usize]) -> Self {
    Self {
      cleaned: noise::cleaned_files(project_folder, streams),
      enabled: noise::cleaned_tracks(project_folder, streams)
        .into_iter()
        .map(|(stream, _)| stream)
        .collect(),
    }
  }
}

/// Tells the preview of the recording open, if one is running, what is
/// cleaned now and heard so.
pub(crate) fn refresh(app: &AppHandle, noise: PreviewNoise) {
  let Some(state) = app.try_state::<RecordingPreviewPlayerState>() else {
    return;
  };
  let Ok(manager) = state.0.lock() else {
    return;
  };
  if let Some(sources) = &manager.sources {
    *sources
      .noise
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = noise;
  }
}
