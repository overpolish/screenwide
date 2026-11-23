// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view below the webview that hosts the picture's layers: the backstop,
//! the workspace and the audio ribbon.

use std::cell::RefCell;
use std::sync::{Arc, Weak};

use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSResponder, NSView};
use objc2_foundation::{NSObject, NSPoint, NSRect, NSSize};
use screenwide_workspace::editing::{SurfaceInner, WorkspaceEditor};

use super::live_resize::LiveResize;
use super::view::guard;
use super::window_follow::placed;

pub(super) struct PictureIvars {
  /// The engine behind the view, once the surface is whole.
  engine: RefCell<Weak<SurfaceInner>>,
  /// Shared with the host, which holds other threads' draws back while it
  /// is set.
  live_resize: Arc<LiveResize>,
}

define_class!(
  // Safety: NSView has no subclassing requirements this class breaks, and
  // it implements no `Drop`.
  #[unsafe(super(NSView, NSResponder, NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwidePictureView"]
  #[ivars = PictureIvars]
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

    /// The view fills the window's content and is resized with it, in the
    /// pass that resizes the window, so the engine hears of the new size
    /// here: drawn now, the picture reaches the screen with the window,
    /// where the page's layout arrives frames later.
    #[unsafe(method(setFrameSize:))]
    fn set_frame_size(&self, size: NSSize) {
      // Safety: NSView implements the method.
      let _: () = unsafe { msg_send![super(self), setFrameSize: size] };
      self.follow_window();
    }

    #[unsafe(method(viewWillStartLiveResize))]
    fn view_will_start_live_resize(&self) {
      // Safety: NSView implements the method.
      let _: () = unsafe { msg_send![super(self), viewWillStartLiveResize] };
      self.ivars().live_resize.set(true);
    }

    /// The draws held back meanwhile are drawn now, as the picture stands.
    #[unsafe(method(viewDidEndLiveResize))]
    fn view_did_end_live_resize(&self) {
      // Safety: NSView implements the method.
      let _: () = unsafe { msg_send![super(self), viewDidEndLiveResize] };
      self.ivars().live_resize.set(false);
      if let Some(inner) = self.ivars().engine.borrow().upgrade() {
        guard(|| WorkspaceEditor { inner }.redraw());
      }
    }
  }

  unsafe impl NSObjectProtocol for PictureView {}
);

impl PictureView {
  pub(super) fn new(
    mtm: MainThreadMarker,
    frame: NSRect,
    live_resize: Arc<LiveResize>,
  ) -> Retained<Self> {
    let this = Self::alloc(mtm).set_ivars(PictureIvars {
      engine: RefCell::new(Weak::new()),
      live_resize,
    });
    // Safety: NSView's designated initialiser.
    unsafe { msg_send![super(this), initWithFrame: frame] }
  }

  pub(super) fn attach(&self, engine: &Arc<SurfaceInner>) {
    *self.ivars().engine.borrow_mut() = Arc::downgrade(engine);
    self.follow_window();
  }

  fn follow_window(&self) {
    let (Some(inner), Some(window)) = (self.ivars().engine.borrow().upgrade(), self.window())
    else {
      return;
    };
    let (size, origin) = placed(&window);
    guard(|| WorkspaceEditor { inner }.follow_window(size, Some(origin)));
  }
}
