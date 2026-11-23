// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hands an editor window's new size to its surface as the window is
//! resized, ahead of the page's layout ([`WorkspaceEditor::follow_window`]).
//! The size arrives on the event loop's thread, inside the drag's modal loop,
//! and the surface redraws on its own thread so the window and its webview
//! keep resizing meanwhile. Sizes that arrive while one is drawn replace each
//! other: only the newest is drawn.

use std::sync::Condvar;

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

use super::*;

/// A window's content size in physical pixels, and where its corner is on
/// screen.
type Placed = ((u32, u32), Option<(i32, i32)>);

/// The newest size of each window waiting to be drawn, by its surface key.
static PENDING: Mutex<Vec<(usize, Placed)>> = Mutex::new(Vec::new());
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
  // Read here, on the window's own thread: a left or top edge dragged moves
  // the corner, which a zoomed picture is kept in place against.
  let placed = (size, content_origin(key));
  let Ok(mut pending) = PENDING.lock() else {
    return;
  };
  match pending.iter_mut().find(|(waiting, _)| *waiting == key) {
    Some((_, waiting)) => *waiting = placed,
    None => pending.push((key, placed)),
  }
  WAKE.notify_one();
}

/// The editor window `key` names moved. Handed on as a resize to the size it
/// has, so its surface notes where the window now is, on the thread that
/// follows resizes: this thread must not wait on the surface.
pub(crate) fn window_moved(key: usize) {
  let mut client = RECT::default();
  // Safety: the key is the live window's handle, read as its move arrives.
  if unsafe { GetClientRect(HWND(key as *mut core::ffi::c_void), &mut client) }.is_err() {
    return;
  }
  let size = (
    u32::try_from(client.right - client.left).unwrap_or(0),
    u32::try_from(client.bottom - client.top).unwrap_or(0),
  );
  window_resized(key, size);
}

/// Where the content area of the window `key`, its handle, starts on screen.
fn content_origin(key: usize) -> Option<(i32, i32)> {
  let mut corner = POINT { x: 0, y: 0 };
  // Safety: the key is the live window's handle, read as its resize arrives.
  unsafe { ClientToScreen(HWND(key as *mut core::ffi::c_void), &mut corner) }
    .as_bool()
    .then_some((corner.x, corner.y))
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
    for (key, (size, origin)) in sizes {
      if let Some(inner) = registry::surface_for_host(key) {
        WorkspaceEditor { inner }.follow_window(size, origin);
      }
    }
  }
}
