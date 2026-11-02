// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Why a project would not open: a short sentence for the person who asked,
//! which names what is wrong without paths, and the details, path and all,
//! for the log.

use std::fmt::Display;
use std::path::Path;

use screenwide_project::ReadError;

pub(in crate::editor) struct OpenFailure {
  reason: String,
  detail: String,
}

impl OpenFailure {
  fn new(reason: String, detail: impl Display) -> Self {
    Self {
      reason,
      detail: detail.to_string(),
    }
  }

  pub(in crate::editor) fn discarded() -> Self {
    Self::new(
      screenwide_i18n::t!("project-browser-open-failed-discarded"),
      "The recording was discarded",
    )
  }

  /// A manifest that names what it holds wrongly, or lacks what its kind
  /// needs.
  pub(in crate::editor) fn damaged(detail: impl Display) -> Self {
    Self::new(
      screenwide_i18n::t!("project-browser-open-failed-damaged"),
      detail,
    )
  }

  pub(in crate::editor) fn recording_missing(primary: &Path) -> Self {
    Self::new(
      screenwide_i18n::t!("project-browser-open-failed-recording-missing"),
      format!(
        "The recording is missing, expected at {}",
        primary.display()
      ),
    )
  }

  pub(in crate::editor) fn recording_unplayable() -> Self {
    Self::new(
      screenwide_i18n::t!("project-browser-open-failed-recording-unplayable"),
      "The recording has no readable picture size",
    )
  }

  pub(in crate::editor) fn picture_unreadable(detail: impl Display) -> Self {
    Self::new(
      screenwide_i18n::t!("project-browser-open-failed-picture-unreadable"),
      detail,
    )
  }

  /// Logs what went wrong with `file` and gives the sentence to show.
  pub(in crate::editor) fn into_reason(self, file: &Path) -> String {
    eprintln!("Could not open {}: {}", file.display(), self.detail);
    self.reason
  }
}

impl From<ReadError> for OpenFailure {
  fn from(error: ReadError) -> Self {
    let reason = match &error {
      ReadError::Unreadable(io) if io.kind() == std::io::ErrorKind::NotFound => {
        screenwide_i18n::t!("project-browser-open-failed-gone")
      }
      ReadError::Unreadable(_) => screenwide_i18n::t!("project-browser-open-failed-unreadable"),
      ReadError::NotProject(_) => screenwide_i18n::t!("project-browser-open-failed-not-project"),
      ReadError::Newer => screenwide_i18n::t!("project-browser-open-failed-newer"),
      ReadError::Damaged(_) => screenwide_i18n::t!("project-browser-open-failed-damaged"),
    };
    Self::new(reason, error)
  }
}

// Anything else, such as the editor failing to come up: said plainly, with
// the details in the log.
impl From<String> for OpenFailure {
  fn from(detail: String) -> Self {
    Self::new(
      screenwide_i18n::t!("project-browser-open-failed-other"),
      detail,
    )
  }
}
