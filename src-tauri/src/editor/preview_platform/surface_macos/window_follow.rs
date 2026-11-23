// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where an editor window is, handed to its surface as it changes: its size
//! as AppKit resizes the picture's view in the same pass as the window
//! ([`super::picture_view`]), and its corner as it moves, so a zoomed
//! picture stays where it is on screen. The twin of the Windows follow.

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::NSWindow;

use super::registry::surface_for;
use super::view::guard;
use super::*;

/// `window`'s content size in physical pixels, and where its corner is on
/// screen, in physical pixels down from the screen's top. Both come from
/// here alone, so every corner the engine compares is measured one way.
pub(super) fn placed(window: &NSWindow) -> ((u32, u32), (i32, i32)) {
  let content = window.contentRectForFrameRect(window.frame());
  let scale = window.backingScaleFactor();
  let size = (
    (content.size.width * scale).round().max(0.0) as u32,
    (content.size.height * scale).round().max(0.0) as u32,
  );
  // AppKit measures up from the bottom; the engine measures down.
  let top = content.origin.y + content.size.height;
  let origin = (
    (content.origin.x * scale).round() as i32,
    (-top * scale).round() as i32,
  );
  (size, origin)
}

/// The editor window `key` names moved, which its picture's view does not
/// hear. Called on the main thread, as the window's events arrive.
pub(crate) fn window_moved(key: usize) {
  if MainThreadMarker::new().is_none() {
    return;
  }
  let Some(inner) = surface_for(key) else {
    return;
  };
  // Safety: the key is the live window's pointer, read as its move arrives
  // on the main thread.
  let Some(window) = (unsafe { Retained::retain(key as *mut NSWindow) }) else {
    return;
  };
  let (size, origin) = placed(&window);
  guard(|| WorkspaceEditor { inner }.follow_window(size, Some(origin)));
}
