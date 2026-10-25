// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Taking the cards away, and holding the next gesture until they are gone.

use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};

use super::{pool, GENERATION, PREFIX};

/// Takes the cards away: at once when `cancelled`, faded out on a commit. A
/// committed carry may finish after its cards have already faded, so a
/// second dismissal neither restarts the fade nor replaces its count.
pub(crate) fn dismiss(app: &AppHandle, cancelled: bool) {
  GENERATION.fetch_add(1, Ordering::AcqRel);
  let windows: Vec<_> = app
    .webview_windows()
    .into_iter()
    .filter(|(label, _)| label.starts_with(PREFIX))
    .map(|(_, window)| window)
    .collect();
  {
    let mut pool = pool();
    if !pool.active {
      return;
    }
    pool.active = false;
    pool.previews.clear();
    pool.fading = if cancelled { 0 } else { windows.len() };
  }
  let _ = app.run_on_main_thread(move || {
    for window in windows {
      if cancelled {
        let _ = crate::app_windows::platform::hide(&window);
      } else if crate::app_windows::platform::fade_glide_window(&window, Box::new(faded)).is_err() {
        let _ = crate::app_windows::platform::hide(&window);
        faded();
      }
    }
  });
}

fn faded() {
  let done = {
    let mut pool = pool();
    pool.fading = pool.fading.saturating_sub(1);
    if pool.fading == 0 {
      pool.after_fade.take()
    } else {
      None
    }
  };
  if let Some(done) = done {
    done();
  }
}

/// Runs `done` once the cards a commit faded out are gone: a fresh gesture
/// cannot reuse a card until then.
pub(crate) fn after_dismissed(done: Box<dyn FnOnce() + Send>) {
  let mut pool = pool();
  if pool.fading == 0 {
    drop(pool);
    done();
  } else {
    pool.after_fade = Some(done);
  }
}

pub(crate) fn is_dismissing() -> bool {
  pool().fading > 0
}
