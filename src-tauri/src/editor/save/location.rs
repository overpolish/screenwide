// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{path::Path, process::Command};

/// Shows the exported file in the system's file browser, chosen in its
/// folder, so it is the one thing in view to open, drag or send on.
pub(super) fn reveal(path: &Path) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  let opened = Command::new("open").arg("-R").arg(path).spawn();
  #[cfg(target_os = "windows")]
  let opened = {
    let mut select = std::ffi::OsString::from("/select,");
    select.push(path);
    Command::new("explorer").arg(select).spawn()
  };
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let opened = Command::new("xdg-open")
    .arg(
      path
        .parent()
        .ok_or_else(|| "The exported file has no containing folder".to_owned())?,
    )
    .spawn();
  opened.map(|_| ()).map_err(|error| error.to_string())
}
