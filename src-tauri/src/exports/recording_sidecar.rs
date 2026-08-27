// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

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

pub(super) fn remove_working_files(
  cursor: Option<&RecordingCursor>,
  keyboard: Option<&RecordingKeyboard>,
) {
  for path in cursor
    .map(|sidecar| &sidecar.path)
    .into_iter()
    .chain(keyboard.map(|sidecar| &sidecar.path))
  {
    let _ = std::fs::remove_file(path);
  }
}

fn for_recording(
  recording: &Path,
  suffix: &str,
  validates: impl FnOnce(&Path) -> bool,
) -> Option<PathBuf> {
  let stem = recording.file_stem()?.to_str()?;
  let sidecar = recording.with_file_name(format!("{stem}.{suffix}.jsonl"));
  (sidecar.is_file() && validates(&sidecar)).then_some(sidecar)
}

pub(super) fn cursor_for_recording(recording: &Path) -> Option<PathBuf> {
  for_recording(recording, "cursor", |path| {
    crate::recording::cursor::read(path).is_ok()
  })
}

pub(super) fn keyboard_for_recording(recording: &Path) -> Option<PathBuf> {
  for_recording(recording, "keyboard", |path| {
    crate::recording::keyboard::read(path).is_ok()
  })
}

fn sweep_unclaimed(directory: &Path, suffix: &str, keep: Option<&Path>) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  let ending = format!(".{suffix}.jsonl");
  for entry in entries.flatten() {
    let path = entry.path();
    let is_sidecar = path
      .file_name()
      .and_then(|name| name.to_str())
      .is_some_and(|name| name.starts_with("recording-") && name.ends_with(&ending));
    if is_sidecar && keep != Some(path.as_path()) {
      let _ = std::fs::remove_file(path);
    }
  }
}

pub(super) fn sweep_unclaimed_cursors(directory: &Path, keep: Option<&Path>) {
  sweep_unclaimed(directory, "cursor", keep);
}

pub(super) fn sweep_unclaimed_keyboards(directory: &Path, keep: Option<&Path>) {
  sweep_unclaimed(directory, "keyboard", keep);
}

#[cfg(test)]
mod tests;
