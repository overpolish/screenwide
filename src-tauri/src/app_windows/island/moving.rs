// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving the island: it tucks away where it is (`island://move`), the page
//! lays it out hidden with nothing showing and says so
//! (`island_ready_to_move`), and only then does the window move and the
//! island come out in its new place, as it would from the notch. No frame
//! shows what it held in the wrong place.

use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Emitter};

use super::{native, put_out, WindowLabel, HIDE_FALLBACK, PENDING_HIDE, SHOWN};

/// Which move is waiting for the page, by number; none when 0.
static MOVING: AtomicU64 = AtomicU64::new(0);
static NEXT_MOVE: AtomicU64 = AtomicU64::new(1);

/// Asks the page to tuck the island away, and moves the window once it has,
/// or after `HIDE_FALLBACK`. Tucking away, it takes no clicks.
pub(super) fn begin(app: &AppHandle) -> tauri::Result<()> {
  let request = NEXT_MOVE.fetch_add(1, Ordering::Relaxed);
  MOVING.store(request, Ordering::Relaxed);
  native::set_hiding(app, true);
  app.emit_to(WindowLabel::Island.as_str(), "island://move", request)?;
  let app = app.clone();
  std::thread::spawn(move || {
    std::thread::sleep(HIDE_FALLBACK);
    let _ = finish(&app, request);
  });
  Ok(())
}

/// The island is going away instead, so no move follows.
pub(super) fn cancel() {
  MOVING.store(0, Ordering::Relaxed);
}

fn finish(app: &AppHandle, request: u64) -> tauri::Result<()> {
  if MOVING
    .compare_exchange(request, 0, Ordering::Relaxed, Ordering::Relaxed)
    .is_err()
  {
    return Ok(());
  }
  if !SHOWN.load(Ordering::Relaxed) || PENDING_HIDE.load(Ordering::Relaxed) != 0 {
    return Ok(());
  }
  put_out(app)
}

/// The page has the island tucked away with nothing showing, so its window
/// can move.
#[tauri::command]
pub fn island_ready_to_move(app: AppHandle, request: u64) -> tauri::Result<()> {
  finish(&app, request)
}
