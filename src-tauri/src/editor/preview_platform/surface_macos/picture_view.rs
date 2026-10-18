// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view below the webview that hosts the picture's layers: the backstop,
//! the workspace and the audio ribbon.

use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSResponder, NSView};
use objc2_foundation::{NSObject, NSPoint, NSRect};

define_class!(
  // Safety: NSView has no subclassing requirements this class breaks, and
  // it implements no `Drop`.
  #[unsafe(super(NSView, NSResponder, NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwidePictureView"]
  pub(super) struct PictureView;

  impl PictureView {
    /// AppKit sets a hosted layer's `geometryFlipped` from its view, so the
    /// view is flipped for the layers to be placed from the top left, as the
    /// engine gives their frames.
    #[unsafe(method(isFlipped))]
    fn is_flipped(&self) -> bool {
      true
    }

    /// The webview over it and the input view over that take every event.
    #[unsafe(method_id(hitTest:))]
    fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> {
      None
    }
  }

  unsafe impl NSObjectProtocol for PictureView {}
);

impl PictureView {
  pub(super) fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
    // Safety: NSView's designated initialiser.
    unsafe { msg_send![Self::alloc(mtm), initWithFrame: frame] }
  }
}
