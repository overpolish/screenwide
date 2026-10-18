// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view's construction and the plumbing its handlers share: the engine
//! it reports to, point conversion and the tracking area.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{msg_send, AllocAnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
  NSEvent, NSPasteboardTypeFileURL, NSPasteboardTypePNG, NSPasteboardTypeTIFF, NSTrackingArea,
  NSTrackingAreaOptions,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize};

use screenwide_workspace::editing::{PointerInput, SurfaceInner, TypingInput, WorkspaceEditor};

use super::{EditorIvars, EditorView};

impl EditorView {
  pub(crate) fn new(mtm: MainThreadMarker) -> Retained<Self> {
    let this = Self::alloc(mtm).set_ivars(EditorIvars {
      engine: RefCell::new(Weak::new()),
      tracking: RefCell::new(None),
      typing: Cell::new(false),
      caret: Cell::new(NSPoint::new(0.0, 0.0)),
      blink: RefCell::new(None),
      drop_accepts: Cell::new(false),
    });
    let frame = super::super::layers::rect(0.0, 0.0, 1.0, 1.0);
    // Safety: NSView's designated initialiser.
    let view: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
    // Safety: the types are AppKit's own constants.
    let types = unsafe {
      NSArray::from_slice(&[
        NSPasteboardTypeFileURL,
        NSPasteboardTypePNG,
        NSPasteboardTypeTIFF,
      ])
    };
    view.registerForDraggedTypes(&types);
    view.setHidden(true);
    view
  }

  /// Ties the view to the engine it hands its events to.
  pub(crate) fn attach(&self, engine: &Arc<SurfaceInner>) {
    *self.ivars().engine.borrow_mut() = Arc::downgrade(engine);
  }

  pub(super) fn engine(&self) -> Option<WorkspaceEditor> {
    let inner = self.ivars().engine.borrow().upgrade()?;
    Some(WorkspaceEditor { inner })
  }

  /// A point in the view's points as the engine takes it: physical pixels
  /// from the view's top left.
  pub(super) fn pixels(&self, point: NSPoint) -> (f64, f64) {
    let scale = self
      .window()
      .map_or(1.0, |window| window.backingScaleFactor());
    (point.x * scale, point.y * scale)
  }

  /// Where `event` happened, in the engine's pixels.
  pub(super) fn event_point(&self, event: &NSEvent) -> (f64, f64) {
    self.pixels(self.convertPoint_fromView(event.locationInWindow(), None))
  }

  pub(super) fn pointer(&self, input: PointerInput) {
    if let Some(engine) = self.engine() {
      guard(|| engine.handle_pointer(input));
    }
  }

  pub(super) fn typing(&self, input: TypingInput) {
    if let Some(engine) = self.engine() {
      guard(|| engine.handle_typing(input));
    }
  }

  /// The caret as a sliver on screen, where an input method opens.
  pub(super) fn caret_on_screen(&self) -> NSRect {
    let caret = self.ivars().caret.get();
    let local = NSRect::new(caret, NSSize::new(1.0, 16.0));
    let in_window = self.convertRect_toView(local, None);
    self
      .window()
      .map_or(in_window, |window| window.convertRectToScreen(in_window))
  }

  pub(super) fn refresh_tracking(&self) {
    let ivars = self.ivars();
    if let Some(old) = ivars.tracking.borrow_mut().take() {
      self.removeTrackingArea(&old);
    }
    let options = NSTrackingAreaOptions::MouseMoved
      | NSTrackingAreaOptions::CursorUpdate
      | NSTrackingAreaOptions::ActiveAlways
      | NSTrackingAreaOptions::InVisibleRect;
    let bounds: NSRect = self.bounds();
    let owner: &AnyObject = self.as_ref();
    // Safety: the view owns the area and outlives it.
    let area = unsafe {
      NSTrackingArea::initWithRect_options_owner_userInfo(
        NSTrackingArea::alloc(),
        bounds,
        options,
        Some(owner),
        None,
      )
    };
    self.addTrackingArea(&area);
    *ivars.tracking.borrow_mut() = Some(area);
  }
}

/// An event handler runs under AppKit, which a panic must not unwind into:
/// the input is dropped and logged instead, as the Windows window procedure
/// does.
pub(super) fn guard(work: impl FnOnce()) {
  if std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err() {
    eprintln!("The macOS preview editor dropped an input after a panic");
  }
}
