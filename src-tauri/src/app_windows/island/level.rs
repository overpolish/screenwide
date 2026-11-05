// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island's level: its own, among the recording panels, or above the
//! capture overlays while a tool that draws there (Annotate, the Ruler,
//! Text Recognition) shows its tools on it.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::AppHandle;

use super::native;

/// The island stands above the capture overlays, for as long as Annotate
/// draws: its tools must never be covered by a stroke.
static ABOVE_OVERLAYS: AtomicBool = AtomicBool::new(false);

/// Raises the island above the capture overlays, or puts it back at its own
/// level. It keeps the choice while hidden and is placed by it when shown.
pub fn set_above_overlays(app: &AppHandle, above: bool) -> tauri::Result<()> {
  ABOVE_OVERLAYS.store(above, Ordering::Relaxed);
  if super::SHOWN.load(Ordering::Relaxed) {
    native::apply_level(app)?;
  }
  Ok(())
}

/// Raises the island over the other recording panels at its own level. A
/// plain raise puts it at the recording panels' level, under the Ruler or
/// Text Recognition it stands over (`set_above_overlays`).
pub(crate) fn raise(app: &AppHandle) -> tauri::Result<()> {
  // Raised only while out: raising orders a hidden panel on screen.
  if !super::is_shown() {
    return Ok(());
  }
  let Some(island) = super::window(app) else {
    return Ok(());
  };
  super::super::platform::raise_without_activation(&island)?;
  native::apply_level(app)
}

#[cfg(target_os = "macos")]
pub(in crate::app_windows) fn is_above_overlays() -> bool {
  ABOVE_OVERLAYS.load(Ordering::Relaxed)
}
