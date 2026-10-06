// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct RecordingKeyboard {
  pub format_version: u16,
  /// A sidecar can exist yet hold no shortcuts (nothing qualifying was ever
  /// pressed); keyboard UI is only offered when there is something to show.
  pub has_shortcuts: bool,
  pub maximum_width_units: u16,
  pub path: PathBuf,
}

pub(crate) struct RecordingCursor {
  pub format_version: u16,
  pub path: PathBuf,
}

impl RecordingCursor {
  pub(super) fn new(path: PathBuf) -> Self {
    Self {
      format_version: crate::recording::cursor::FORMAT_VERSION,
      path,
    }
  }
}

impl RecordingKeyboard {
  pub(super) fn new(path: PathBuf) -> Self {
    let compositor = super::keyboard_effects::KeyboardCompositor::open(&path).ok();
    Self {
      format_version: crate::recording::keyboard::FORMAT_VERSION,
      has_shortcuts: compositor
        .as_ref()
        .is_some_and(|keyboard| keyboard.shortcut_count() > 0),
      maximum_width_units: compositor
        .map(|keyboard| keyboard.maximum_width_units())
        .unwrap_or(20),
      path,
    }
  }
}

pub(super) fn total_size(
  cursor: Option<&RecordingCursor>,
  keyboard: Option<&RecordingKeyboard>,
) -> u64 {
  cursor
    .map(|sidecar| &sidecar.path)
    .into_iter()
    .chain(keyboard.map(|sidecar| &sidecar.path))
    .filter_map(|path| std::fs::metadata(path).ok())
    .map(|metadata| metadata.len())
    .sum()
}

/// `path`, if it holds cursor data this version can read.
pub(super) fn valid_cursor(path: PathBuf) -> Option<PathBuf> {
  crate::recording::cursor::read(&path)
    .is_ok()
    .then_some(path)
}

/// `path`, if it holds keyboard data this version can read.
pub(super) fn valid_keyboard(path: PathBuf) -> Option<PathBuf> {
  crate::recording::keyboard::read(&path)
    .is_ok()
    .then_some(path)
}

#[cfg(test)]
mod tests;
