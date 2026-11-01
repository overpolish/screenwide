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
  NSDraggingSource, NSEventType, NSPasteboard, NSPasteboardWriting, NSView, NSWorkspace,
};
use objc2_foundation::{NSArray, NSObject, NSPoint, NSRect, NSSize, NSString, NSURL};

/// How large the file's icon is drawn under the pointer as it is dragged.
const DRAG_ICON_POINTS: f64 = 48.0;

thread_local! {
  /// The source of the drag under way, kept until the next one, as the
  /// session holds it weakly.
  static SOURCE: RefCell<Option<Retained<ExportDragSource>>> = const { RefCell::new(None) };
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

pub(super) fn drag(window: &tauri::WebviewWindow, path: &Path) -> Result<(), String> {
  let marker = main_thread()?;
  let view = page_view(window)?;
  let event = NSApplication::sharedApplication(marker)
    .currentEvent()
    .filter(|event| {
      matches!(
        event.r#type(),
        NSEventType::LeftMouseDown | NSEventType::LeftMouseDragged
      )
    })
    .ok_or_else(|| "The drag was let go before it could start".to_owned())?;
  let url = file_url(path);
  let item = NSDraggingItem::initWithPasteboardWriter(
    NSDraggingItem::alloc(),
    ProtocolObject::from_ref(&*url),
  );
  let icon =
    NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(&path.to_string_lossy()));
  let at = view.convertPoint_fromView(event.locationInWindow(), None);
  let half = DRAG_ICON_POINTS / 2.0;
  // Safety: the contents are an NSImage, which a dragging frame draws.
  unsafe {
    item.setDraggingFrame_contents(
      NSRect::new(
        NSPoint::new(at.x - half, at.y - half),
        NSSize::new(DRAG_ICON_POINTS, DRAG_ICON_POINTS),
      ),
      Some(&icon),
    );
  }
  let source = ExportDragSource::new(marker);
  view.beginDraggingSessionWithItems_event_source(
    &NSArray::from_retained_slice(&[item]),
    &event,
    ProtocolObject::from_ref(&*source),
  );
  SOURCE.with(|kept| kept.replace(Some(source)));
  Ok(())
}
