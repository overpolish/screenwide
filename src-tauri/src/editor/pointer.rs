// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pointer brought back after a key the editor takes.
//!
//! WebKit hides the pointer until it moves whenever a page takes a key
//! (`WebViewImpl::doneWithKeyEvent`), as a text field does while typing. The
//! editor's keys are about what the pointer does next, so the page says when
//! it took one (`keep-pointer-shown.ts`) and the pointer is shown again.
//! WebView2 leaves the pointer alone, so only a Mac listens.

/// What the editor's page emits once a key it took is done.
#[cfg(target_os = "macos")]
const KEY_TAKEN_EVENT: &str = "editor://key-taken";

#[cfg(target_os = "macos")]
pub(super) fn listen(app: &tauri::AppHandle) {
  use tauri::Listener;

  let main = app.clone();
  app.listen_any(KEY_TAKEN_EVENT, move |_| {
    let shown =
      main.run_on_main_thread(|| objc2_app_kit::NSCursor::setHiddenUntilMouseMoves(false));
    if let Err(error) = shown {
      eprintln!("Could not show the pointer again: {error}");
    }
  });
}
