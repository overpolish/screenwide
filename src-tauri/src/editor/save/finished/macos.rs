// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The exported file handed to AppKit: on the pasteboard and as a drag out
//! of the island's window. Each runs on the main thread, where synchronous
//! commands arrive.

use std::cell::RefCell;
use std::path::Path;

use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, AllocAnyThread, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
  NSApplication, NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession,
  NSDraggingSource, NSEvent, NSEventModifierFlags, NSEventType, NSImage, NSPasteboard,
  NSPasteboardWriting, NSView, NSWindow, NSWorkspace,
};
use objc2_foundation::{
  NSArray, NSData, NSObject, NSPoint, NSProcessInfo, NSRect, NSSize, NSString, NSURL,
};

use super::DragLook;

/// How large the file's icon is drawn under the pointer as it is dragged.
const DRAG_ICON_POINTS: f64 = 48.0;

thread_local! {
  /// The source of the drag under way, kept until the next one, as the
  /// session holds it weakly.
  static SOURCE: RefCell<Option<Retained<ExportDragSource>>> = const { RefCell::new(None) };
  /// The window whose page pressed the drag under way, told of the release
  /// when it ends.
  static PRESSED: RefCell<Option<Retained<NSWindow>>> = const { RefCell::new(None) };
}

define_class!(
  // Safety: NSObject has no subclassing requirements, and this implements no
  // `Drop`.
  #[unsafe(super(NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwideExportDragSource"]
  struct ExportDragSource;

  unsafe impl NSObjectProtocol for ExportDragSource {}

  unsafe impl NSDraggingSource for ExportDragSource {
    /// The file is copied wherever it lands, never moved out of its folder.
    #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
    fn operation_mask(
      &self,
      _session: &NSDraggingSession,
      _context: NSDraggingContext,
    ) -> NSDragOperation {
      NSDragOperation::Copy
    }

    /// The page that started the drag pressed the mouse and never sees it
    /// let go: the drag took the release. Left so, WebKit still holds the
    /// press, so the next click only ends it, and the click after counts as
    /// one with no pointer, which React Aria rings as keyboard focus. It is
    /// handed the release where the drag ended, wherever that was.
    #[unsafe(method(draggingSession:endedAtPoint:operation:))]
    fn ended(&self, _session: &NSDraggingSession, at: NSPoint, _operation: NSDragOperation) {
      let Some(window) = PRESSED.with(|pressed| pressed.borrow_mut().take()) else {
        return;
      };
      if let Some(release) = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
        NSEventType::LeftMouseUp,
        window.convertPointFromScreen(at),
        NSEventModifierFlags::empty(),
        NSProcessInfo::processInfo().systemUptime(),
        window.windowNumber(),
        None,
        0,
        1,
        0.0,
      ) {
        window.sendEvent(&release);
      }
    }
  }
);

impl ExportDragSource {
  fn new(marker: MainThreadMarker) -> Retained<Self> {
    unsafe { msg_send![Self::alloc(marker), init] }
  }
}

fn main_thread() -> Result<MainThreadMarker, String> {
  MainThreadMarker::new().ok_or_else(|| "AppKit was asked off the main thread".to_owned())
}

fn file_url(path: &Path) -> Retained<NSURL> {
  NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()))
}

/// The page's view in `window`, which a drag starts from.
fn page_view(window: &tauri::WebviewWindow) -> Result<&NSView, String> {
  let view = window.ns_view().map_err(|error| error.to_string())?;
  // Safety: the pointer is the window's own content view, alive while the
  // window is, and this runs on the main thread that owns it.
  Ok(unsafe { &*view.cast::<NSView>() })
}

pub(super) fn copy(path: &Path) -> Result<(), String> {
  main_thread()?;
  let url = ProtocolObject::<dyn NSPasteboardWriting>::from_retained(file_url(path));
  let pasteboard = NSPasteboard::generalPasteboard();
  pasteboard.clearContents();
  if pasteboard.writeObjects(&NSArray::from_retained_slice(&[url])) {
    Ok(())
  } else {
    Err("The file could not be put on the clipboard".to_owned())
  }
}

/// The press the drag starts from: the mouse event being handled, or, where
/// AppKit is handling something else by the time the page asks (as when the
/// window was not the key window at the press), one made at the pointer.
fn press_event(view: &NSView, marker: MainThreadMarker) -> Result<Retained<NSEvent>, String> {
  if let Some(event) = NSApplication::sharedApplication(marker)
    .currentEvent()
    .filter(|event| {
      matches!(
        event.r#type(),
        NSEventType::LeftMouseDown | NSEventType::LeftMouseDragged
      )
    })
  {
    return Ok(event);
  }
  let window = view.window().ok_or("The page has no window")?;
  NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
    NSEventType::LeftMouseDragged,
    window.mouseLocationOutsideOfEventStream(),
    NSEventModifierFlags::empty(),
    NSProcessInfo::processInfo().systemUptime(),
    window.windowNumber(),
    None,
    0,
    1,
    1.0,
  )
  .ok_or_else(|| "The drag could not be started".to_owned())
}

/// What is drawn under the pointer, and its size in points.
fn drawn(path: &Path, look: &DragLook) -> (Retained<NSImage>, NSSize) {
  let picture = match look {
    DragLook::Icon => None,
    DragLook::File { width, height } => NSImage::initWithContentsOfFile(
      NSImage::alloc(),
      &NSString::from_str(&path.to_string_lossy()),
    )
    .map(|image| (image, NSSize::new(*width, *height))),
    DragLook::Png { png, width, height } => {
      NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(png))
        .map(|image| (image, NSSize::new(*width, *height)))
    }
  };
  picture.unwrap_or_else(|| {
    (
      NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(&path.to_string_lossy())),
      NSSize::new(DRAG_ICON_POINTS, DRAG_ICON_POINTS),
    )
  })
}

pub(super) fn drag(
  window: &tauri::WebviewWindow,
  path: &Path,
  look: &DragLook,
) -> Result<(), String> {
  let marker = main_thread()?;
  let view = page_view(window)?;
  let event = press_event(view, marker)?;
  let url = file_url(path);
  let item = NSDraggingItem::initWithPasteboardWriter(
    NSDraggingItem::alloc(),
    ProtocolObject::from_ref(&*url),
  );
  let (image, size) = drawn(path, look);
  image.setSize(size);
  let at = view.convertPoint_fromView(event.locationInWindow(), None);
  // Safety: the contents are an NSImage, which a dragging frame draws.
  unsafe {
    item.setDraggingFrame_contents(
      NSRect::new(
        NSPoint::new(at.x - size.width / 2.0, at.y - size.height / 2.0),
        size,
      ),
      Some(&image),
    );
  }
  let source = ExportDragSource::new(marker);
  view.beginDraggingSessionWithItems_event_source(
    &NSArray::from_retained_slice(&[item]),
    &event,
    ProtocolObject::from_ref(&*source),
  );
  SOURCE.with(|kept| kept.replace(Some(source)));
  PRESSED.with(|pressed| pressed.replace(view.window()));
  Ok(())
}
