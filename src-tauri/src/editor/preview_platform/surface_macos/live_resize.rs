// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The picture while its window is live-resized. AppKit resizes the window
//! in passes on the main thread, and the picture is laid out again inside
//! each one and presented with it ([`super::picture_view`]), so it lands in
//! the frame the window does. A draw from another thread meanwhile, such as
//! a layout's or a playing video's, presents on its own schedule, and could
//! land a frame before or after the window, at a place a step out of date:
//! a zoomed picture visibly jumped. While a resize lasts, such a draw is
//! handed to the main thread instead.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use objc2::MainThreadMarker;

use super::host::Views;
use super::main_thread::OnMain;

#[derive(Default)]
pub(super) struct LiveResize {
  resizing: AtomicBool,
  /// A redraw is waiting on the main thread, so held-back draws queue one.
  queued: Arc<AtomicBool>,
}

impl LiveResize {
  pub(super) fn set(&self, resizing: bool) {
    self.resizing.store(resizing, Ordering::Release);
  }

  /// Whether a draw on this thread is to be left to the main thread, which
  /// is then asked to draw the picture as it stands.
  pub(super) fn defers(&self, views: &OnMain<Views>) -> bool {
    if MainThreadMarker::new().is_some() || !self.resizing.load(Ordering::Acquire) {
      return false;
    }
    if !self.queued.swap(true, Ordering::AcqRel) {
      let queued = Arc::clone(&self.queued);
      views.later(move |views, _| {
        queued.store(false, Ordering::Release);
        if let Some(engine) = views.editor.engine() {
          engine.redraw();
        }
      });
    }
    true
  }
}
