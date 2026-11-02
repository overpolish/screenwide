// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Why a manifest could not be read, told apart so the app can say it in
//! words a person reads while the details, path and all, go to the log.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ReadError {
  /// The file could not be read from disk, such as when it is gone.
  Unreadable(std::io::Error),
  /// The file is not a manifest at all.
  NotProject(PathBuf),
  /// A newer version of the app wrote it, in a format this one cannot read.
  Newer,
  /// A manifest whose contents do not make a project.
  Damaged(serde_json::Error),
}

impl fmt::Display for ReadError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Unreadable(error) => write!(formatter, "The project could not be read: {error}"),
      Self::NotProject(file) => write!(formatter, "{} is not a Screenwide project", file.display()),
      Self::Newer => formatter.write_str("This project was made by a newer version of Screenwide"),
      Self::Damaged(error) => write!(formatter, "The project file is damaged: {error}"),
    }
  }
}

impl std::error::Error for ReadError {}

// Most callers only pass the details on, as the app's other errors are.
impl From<ReadError> for String {
  fn from(error: ReadError) -> Self {
    error.to_string()
  }
}
