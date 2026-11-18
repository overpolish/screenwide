// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hands an editor window's new size to its surface as the window is
//! resized, ahead of the page's layout ([`WorkspaceEditor::follow_window`]).
//! The size arrives on the event loop's thread, inside the drag's modal loop,
//! and the surface redraws on its own thread so the window and its webview
//! keep resizing meanwhile. Sizes that arrive while one is drawn replace each
//! other: only the newest is drawn.

use std::sync::Condvar;

use super::*;

/// The newest size of each window waiting to be drawn, by its surface key.
static PENDING: Mutex<Vec<(usize, (u32, u32))>> = Mutex::new(Vec::new());
static WAKE: Condvar = Condvar::new();
static THREAD: OnceLock<()> = OnceLock::new();

/// The editor window `key` names now has `size` physical pixels of content.
pub(crate) fn window_resized(key: usize, size: (u32, u32)) {
  THREAD.get_or_init(|| {
    if let Err(error) = std::thread::Builder::new()
      .name("editor-window-follow".to_owned())
      .spawn(run)
    {
      eprintln!("Could not start following editor window resizes: {error}");
    }
  });
  let Ok(mut pending) = PENDING.lock() else {
    return;
  };
  match pending.iter_mut().find(|(waiting, _)| *waiting == key) {
    Some((_, waiting)) => *waiting = size,
    None => pending.push((key, size)),
  }
  WAKE.notify_one();
}

fn run() {
  loop {
    let sizes = {
      let Ok(mut pending) = PENDING.lock() else {
        return;
      };
      while pending.is_empty() {
        pending = match WAKE.wait(pending) {
          Ok(pending) => pending,
          Err(_) => return,
        };
      }
      std::mem::take(&mut *pending)
    };
    for (key, size) in sizes {
      if let Some(inner) = registry::surface_for_host(key) {
        WorkspaceEditor { inner }.follow_window(size);
      }
    }
  }
}
