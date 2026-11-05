// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Option-dragging a picture on macOS (`references::drop`). A press
//! anywhere on the desktop starts a short watch for as long as the button is
//! held: once a drag has begun from it, the drag carries a picture and
//! Option is held, a catcher panel is laid over each display and the hint
//! island comes out; letting Option go takes them away again, so the drop
//! lands where it would have. The watch reads the pointer's buttons, the
//! modifiers and the drag pasteboard as they stand, rather than relying on
//! another app's drag to pass its events on. Everything here runs on the
//! main thread.

mod view;

use std::cell::RefCell;
use std::ptr::NonNull;
use std::time::Duration;

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
use tauri::AppHandle;

use super::drop::Dropped;
use view::{carries_picture, DropView};

/// How often the pointer is looked at while a button is held.
const POLL: Duration = Duration::from_millis(33);
/// How long the catchers stay after the button is let go, for a drop on
/// them to arrive.
const LINGER: Duration = Duration::from_millis(400);

struct Watch {
  app: AppHandle,
  _monitor: Option<Retained<AnyObject>>,
  /// The drag pasteboard's count as the button went down, which a drag
  /// begun from the press moves on.
  pressed_at: Option<isize>,
  catchers: Vec<Retained<NSPanel>>,
}

thread_local! {
  static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) };
}

fn drag_pasteboard() -> Retained<NSPasteboard> {
  // Safety: the name is AppKit's own constant.
  NSPasteboard::pasteboardWithName(unsafe { NSPasteboardNameDrag })
}

/// Starts watching for presses. Main thread only.
pub(crate) fn start(app: &AppHandle) {
  let pressed = RcBlock::new(|_: NonNull<NSEvent>| pressed());
  let monitor =
    NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::LeftMouseDown, &pressed);
  WATCH.with(|watch| {
    *watch.borrow_mut() = Some(Watch {
      app: app.clone(),
      _monitor: monitor,
      pressed_at: None,
      catchers: Vec::new(),
    });
  });
}

fn pressed() {
  let app = WATCH.with(|watch| {
    let mut watch = watch.borrow_mut();
    let watch = watch.as_mut()?;
    if watch.pressed_at.is_some() {
      return None;
    }
    watch.pressed_at = Some(drag_pasteboard().changeCount());
    Some(watch.app.clone())
  });
  let Some(app) = app else {
    return;
  };
  std::thread::spawn(move || loop {
    std::thread::sleep(POLL);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let looked = app.run_on_main_thread(move || {
      let _ = sender.send(look());
    });
    if looked.is_err() || !receiver.recv().unwrap_or(false) {
      break;
    }
  });
}

/// Puts the catchers up or away as the press stands. Answers whether the
/// button is still held, so the watch goes on.
fn look() -> bool {
  let Some(mtm) = MainThreadMarker::new() else {
    return false;
  };
  let held = NSEvent::pressedMouseButtons() & 1 != 0;
  let option = NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Option);
  WATCH.with(|watch| {
    let mut watch = watch.borrow_mut();
    let Some(watch) = watch.as_mut() else {
      return false;
    };
    let pasteboard = drag_pasteboard();
    let dragging = watch
      .pressed_at
      .is_some_and(|count| pasteboard.changeCount() != count)
      && carries_picture(&pasteboard);
    if held && option && dragging {
      if watch.catchers.is_empty() {
        watch.catchers = catchers(mtm);
        super::drop_island::show(&watch.app);
      }
    } else if !watch.catchers.is_empty() && held {
      put_away(watch);
    }
    if !held {
      watch.pressed_at = None;
      if !watch.catchers.is_empty() {
        let app = watch.app.clone();
        std::thread::spawn(move || {
          std::thread::sleep(LINGER);
          let _ = app.run_on_main_thread(|| {
            WATCH.with(|watch| {
              if let Some(watch) = watch.borrow_mut().as_mut() {
                put_away(watch);
              }
            });
          });
        });
      }
    }
    held
  })
}

fn put_away(watch: &mut Watch) {
  for catcher in watch.catchers.drain(..) {
    catcher.orderOut(None);
  }
  super::drop_island::withdraw(&watch.app);
}

/// A drop panel over each display, invisible but there to be dropped on.
fn catchers(mtm: MainThreadMarker) -> Vec<Retained<NSPanel>> {
  NSScreen::screens(mtm)
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
      // Safety: the watch keeps the panel until it is put away.
      unsafe { panel.setReleasedWhenClosed(false) };
      let view = DropView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), frame.size));
      panel.setContentView(Some(&view));
      panel.orderFrontRegardless();
      panel
    })
    .collect()
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
  let at = (on_screen.x, primary - on_screen.y);
  WATCH.with(|watch| {
    if let Some(watch) = watch.borrow_mut().as_mut() {
      put_away(watch);
      super::drop::keep(&watch.app, forms, at, scale);
    }
  });
}
