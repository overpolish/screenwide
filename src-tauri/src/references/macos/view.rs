// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view laid over a reference's webview, over the picture and the margin
//! its handles reach into. It carries the picture's and the chrome's Metal
//! layers and takes the pointer there, handing every event to the reference
//! it belongs to; the island, under it, stays the page's. It never takes the
//! keyboard, which stays with the page for the island and the arrow keys'
//! nudges.

use std::cell::RefCell;
use std::ptr::NonNull;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{
  define_class, msg_send, AllocAnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, Message,
};
use objc2_app_kit::{NSEvent, NSResponder, NSTrackingArea, NSTrackingAreaOptions, NSView};
use objc2_foundation::{NSObject, NSPoint, NSRect};
use objc2_quartz_core::CAMetalLayer;
use screenwide_gpu::surface::Surface;
use screenwide_osc::geometry::Point;

use super::input;

pub(super) struct ReferenceIvars {
  /// The reference's window label, which names it to its state.
  label: String,
  tracking: RefCell<Option<Retained<NSTrackingArea>>>,
}

define_class!(
  // Safety: NSView has no subclassing requirements this class breaks, and
  // it implements no `Drop`.
  #[unsafe(super(NSView, NSResponder, NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwideReferenceView"]
  #[ivars = ReferenceIvars]
  pub(super) struct ReferenceView;

  impl ReferenceView {
    #[unsafe(method(isFlipped))]
    fn is_flipped(&self) -> bool {
      true
    }

    #[unsafe(method(acceptsFirstMouse:))]
    fn accepts_first_mouse(&self, _event: Option<&NSEvent>) -> bool {
      true
    }

    #[unsafe(method(acceptsFirstResponder))]
    fn accepts_first_responder(&self) -> bool {
      false
    }

    // The view moves the window itself, so the window server's own drag
    // never starts from it.
    #[unsafe(method(mouseDownCanMoveWindow))]
    fn mouse_down_can_move_window(&self) -> bool {
      false
    }

    // The view reaches into the shadow's room and over the island's top;
    // only the picture and its handles' reach are its own, and the rest
    // falls through to the page.
    #[unsafe(method_id(hitTest:))]
    fn hit_test(&self, point: NSPoint) -> Option<Retained<NSView>> {
      // `hitTest:`'s point is in the superview's coordinates. Safety: the
      // superview is the window's content view, alive while this is in it.
      let superview = unsafe { self.superview() };
      let local = self.convertPoint_fromView(point, superview.as_deref());
      input::takes(self.label(), Point { x: local.x, y: local.y })
        .then(|| Retained::into_super(self.retain()))
    }

    #[unsafe(method(updateTrackingAreas))]
    fn update_tracking_areas(&self) {
      self.refresh_tracking();
      // Safety: NSView implements the method.
      let _: () = unsafe { msg_send![super(self), updateTrackingAreas] };
    }

    #[unsafe(method(cursorUpdate:))]
    fn cursor_update(&self, event: &NSEvent) {
      input::hover(self.label(), Some(self.point(event)));
    }

    #[unsafe(method(mouseEntered:))]
    fn mouse_entered(&self, event: &NSEvent) {
      input::hover(self.label(), Some(self.point(event)));
    }

    #[unsafe(method(mouseMoved:))]
    fn mouse_moved(&self, event: &NSEvent) {
      input::hover(self.label(), Some(self.point(event)));
    }

    #[unsafe(method(mouseExited:))]
    fn mouse_exited(&self, _event: &NSEvent) {
      input::hover(self.label(), None);
    }

    #[unsafe(method(mouseDown:))]
    fn mouse_down(&self, event: &NSEvent) {
      input::down(self.label(), self.point(event));
    }

    #[unsafe(method(mouseDragged:))]
    fn mouse_dragged(&self, _event: &NSEvent) {
      input::dragged(self.label());
    }

    #[unsafe(method(mouseUp:))]
    fn mouse_up(&self, event: &NSEvent) {
      input::up(self.label(), self.point(event));
    }
  }
);

impl ReferenceView {
  pub(super) fn new(mtm: MainThreadMarker, label: &str, frame: NSRect) -> Retained<Self> {
    let this = Self::alloc(mtm).set_ivars(ReferenceIvars {
      label: label.to_owned(),
      tracking: RefCell::new(None),
    });
    // Safety: NSView's designated initialiser.
    unsafe { msg_send![super(this), initWithFrame: frame] }
  }

  fn label(&self) -> &str {
    &self.ivars().label
  }

  /// Where `event` happened, in the view's points from its top left.
  fn point(&self, event: &NSEvent) -> Point {
    let point = self.convertPoint_fromView(event.locationInWindow(), None);
    Point {
      x: point.x,
      y: point.y,
    }
  }

  fn refresh_tracking(&self) {
    let ivars = self.ivars();
    if let Some(old) = ivars.tracking.borrow_mut().take() {
      self.removeTrackingArea(&old);
    }
    let options = NSTrackingAreaOptions::MouseMoved
      | NSTrackingAreaOptions::MouseEnteredAndExited
      | NSTrackingAreaOptions::CursorUpdate
      | NSTrackingAreaOptions::ActiveAlways
      | NSTrackingAreaOptions::InVisibleRect;
    let owner: &AnyObject = self.as_ref();
    // Safety: the view owns the area and outlives it.
    let area = unsafe {
      NSTrackingArea::initWithRect_options_owner_userInfo(
        NSTrackingArea::alloc(),
        self.bounds(),
        options,
        Some(owner),
        None,
      )
    };
    self.addTrackingArea(&area);
    *ivars.tracking.borrow_mut() = Some(area);
  }
}

pub(super) fn metal_layer(scale: f64) -> Retained<CAMetalLayer> {
  let layer = CAMetalLayer::new();
  layer.setOpaque(false);
  layer.setContentsScale(scale);
  layer
}

/// A wgpu surface on `layer`.
///
/// # Safety
/// Main thread only, on a live layer that outlives the surface.
pub(super) unsafe fn surface_on(
  gpu: &screenwide_gpu::Gpu,
  layer: &CAMetalLayer,
) -> Result<Surface, String> {
  let pointer = NonNull::from(layer).cast::<std::ffi::c_void>();
  unsafe { Surface::on_metal_layer(gpu, pointer) }
}
