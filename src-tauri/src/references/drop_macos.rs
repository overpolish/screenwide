// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Option-dragging a picture on macOS: what the drop watch asks of the
//! platform (`references::drop_watch`). A global monitor tells it of each
//! press; the pointer's buttons, the modifiers and the drag pasteboard are
//! read as they stand, rather than relying on another app's drag to pass its
//! events on, and the pasteboard says what a drag carries before any catcher
//! is up. Everything here runs on the main thread.

mod view;

use std::cell::RefCell;
use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSEvent, NSEventMask, NSEventModifierFlags, NSPanel, NSPasteboard,
  NSPasteboardNameDrag, NSPopUpMenuWindowLevel, NSScreen, NSWindowCollectionBehavior,
  NSWindowSharingType, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect};
use tauri::{AppHandle, LogicalPosition, Position};

use super::drop::Dropped;
use super::drop_watch;
use view::{carries_picture, DropView};

/// The drag pasteboard's count as the button went down, which a drag begun
/// from the press moves on.
pub(super) type Mark = isize;

/// The drag pasteboard says what a drag carries as soon as it begins.
pub(super) const SEES_PICTURE_BEFORE_CATCHERS: bool = true;

thread_local! {
  static MONITOR: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
}

fn drag_pasteboard() -> Retained<NSPasteboard> {
  // Safety: the name is AppKit's own constant.
  NSPasteboard::pasteboardWithName(unsafe { NSPasteboardNameDrag })
}

/// Tells the watch of every press on the desktop.
pub(super) fn listen(_app: &AppHandle) {
  let pressed = RcBlock::new(|_: NonNull<NSEvent>| drop_watch::pressed());
  let monitor =
    NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::LeftMouseDown, &pressed);
  MONITOR.with(|kept| *kept.borrow_mut() = monitor);
}

pub(super) fn held() -> bool {
  NSEvent::pressedMouseButtons() & 1 != 0
}

pub(super) fn modifier() -> bool {
  NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Option)
}

pub(super) fn mark() -> Mark {
  drag_pasteboard().changeCount()
}

/// Whether a drag of a picture has begun since the press at `mark`.
pub(super) fn dragging(mark: Mark) -> bool {
  let pasteboard = drag_pasteboard();
  pasteboard.changeCount() != mark && carries_picture(&pasteboard)
}

/// A drop panel over each display, invisible but there to be dropped on.
pub(super) struct Catchers(Vec<Retained<NSPanel>>);

impl Catchers {
  pub(super) fn put_up() -> Self {
    let Some(mtm) = MainThreadMarker::new() else {
      return Self(Vec::new());
    };
    let panels = NSScreen::screens(mtm)
      .iter()
      .map(|screen| {
        let frame = screen.frame();
        let panel = NSPanel::initWithContentRect_styleMask_backing_defer_screen(
          NSPanel::alloc(mtm),
          NSRect::new(NSPoint::new(0.0, 0.0), frame.size),
          NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
          NSBackingStoreType::Buffered,
          false,
          None,
        );
        panel.setFrame_display(frame, false);
        // Not quite clear: a window the pointer sees nothing of is passed
        // over for drops as it is for clicks.
        panel.setBackgroundColor(Some(&NSColor::colorWithWhite_alpha(0.0, 0.005)));
        panel.setOpaque(false);
        panel.setHasShadow(false);
        panel.setLevel(NSPopUpMenuWindowLevel);
        panel.setCollectionBehavior(
          NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
        );
        panel.setHidesOnDeactivate(false);
        panel.setSharingType(NSWindowSharingType::None);
        // Safety: the catchers keep the panel until they are put away.
        unsafe { panel.setReleasedWhenClosed(false) };
        let view = DropView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), frame.size));
        panel.setContentView(Some(&view));
        panel.orderFrontRegardless();
        panel
      })
      .collect();
    Self(panels)
  }

  pub(super) fn put_away(self) {
    for panel in self.0 {
      panel.orderOut(None);
    }
  }
}

/// A picture was dropped on a catcher at `on_screen`, in AppKit's screen
/// points, on a display of `scale` pixels to the point.
fn dropped(forms: Vec<Dropped>, on_screen: NSPoint, scale: f64) {
  let Some(mtm) = MainThreadMarker::new() else {
    return;
  };
  let primary = NSScreen::screens(mtm)
    .firstObject()
    .map_or(0.0, |screen| screen.frame().size.height);
  let at = LogicalPosition::new(on_screen.x, primary - on_screen.y);
  drop_watch::dropped(forms, Position::Logical(at), scale);
}
