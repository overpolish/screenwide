// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system's file browser: Finder or Explorer.

use std::{path::Path, process::Command};

/// Shows `path` in the file browser, chosen in its folder, so it is the one
/// thing in view to open, drag or send on.
pub(crate) fn reveal(path: &Path) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  let shown = Command::new("open").arg("-R").arg(path).spawn();
  // Explorer parses its own command line: Rust would quote the whole
  // "/select,<path>" argument when the path has a space, which Explorer
  // rejects by opening its default folder. Only the path may be quoted.
  #[cfg(target_os = "windows")]
  let shown = {
    use std::os::windows::process::CommandExt;
    Command::new("explorer")
      .raw_arg(format!("/select,\"{}\"", path.display()))
      .spawn()
  };
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let shown = Command::new("xdg-open")
    .arg(path.parent().unwrap_or(path))
    .spawn();
  shown.map(|_| ()).map_err(|error| error.to_string())
}
