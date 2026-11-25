// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Why writing an export failed, said for the person who chose where it
//! goes: a folder that cannot be written to, a file where the folder should
//! be, or a full disk, in their language; anything else in the system's own
//! words.

use std::io::{Error, ErrorKind};

/// The reason the Export island gives for `error`, met making the export's
/// folder or writing a file in it.
pub(crate) fn write_failure(error: &Error) -> String {
  match error.kind() {
    ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem => {
      screenwide_i18n::t!("editor-export-failed-denied")
    }
    ErrorKind::AlreadyExists | ErrorKind::NotADirectory => {
      screenwide_i18n::t!("editor-export-failed-not-folder")
    }
    ErrorKind::StorageFull => screenwide_i18n::t!("editor-export-failed-full"),
    _ => error.to_string(),
  }
}
