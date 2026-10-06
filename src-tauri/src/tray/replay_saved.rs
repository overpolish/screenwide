// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The moment after a replay clip is saved, when the tray confirms it. A save
//! opens nothing, so the tick is the only sign it happened.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use tauri::AppHandle;

const SHOWN_FOR: Duration = Duration::from_millis(1_200);

/// Counts saves, so the tick from one save is not cut short by the timer of
/// the save before it.
static SAVES: AtomicU64 = AtomicU64::new(0);
static SHOWING: AtomicBool = AtomicBool::new(false);

pub(super) fn showing() -> bool {
  SHOWING.load(Ordering::Acquire)
}

/// Shows the tick for a moment, then whatever the tray would show otherwise.
pub fn confirm(app: &AppHandle) {
  let save = SAVES.fetch_add(1, Ordering::AcqRel) + 1;
  SHOWING.store(true, Ordering::Release);
  super::refresh_icon(app);
  let app = app.clone();
  let _ = std::thread::Builder::new()
    .name("replay-saved-tick".to_owned())
    .spawn(move || {
      std::thread::sleep(SHOWN_FOR);
      if SAVES.load(Ordering::Acquire) == save {
        SHOWING.store(false, Ordering::Release);
        super::refresh_icon(&app);
      }
    });
}
