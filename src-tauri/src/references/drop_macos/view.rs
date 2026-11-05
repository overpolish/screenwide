// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The view filling each catcher panel: a dragging destination for
//! pictures, taking whatever form the drag carries them in and handing them
//! on with where they were let go.

use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
  NSDragOperation, NSDraggingInfo, NSPasteboard, NSPasteboardTypeFileURL, NSPasteboardTypePNG,
  NSPasteboardTypeTIFF, NSPasteboardTypeURL, NSResponder, NSView,
};
use objc2_foundation::{NSArray, NSObject, NSRect, NSURL};

use super::super::drop::Dropped;

define_class!(
  // Safety: NSView has no subclassing requirements this class breaks, and
  // it implements no `Drop`.
  #[unsafe(super(NSView, NSResponder, NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwideReferenceDropView"]
  pub(super) struct DropView;

  impl DropView {
    #[unsafe(method(draggingEntered:))]
    fn dragging_entered(&self, info: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
      operation(info)
    }

    #[unsafe(method(draggingUpdated:))]
    fn dragging_updated(&self, info: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
      operation(info)
    }

    #[unsafe(method(performDragOperation:))]
    fn perform_drag_operation(&self, info: &ProtocolObject<dyn NSDraggingInfo>) -> bool {
      perform(self, info)
    }
  }

  unsafe impl NSObjectProtocol for DropView {}
);

/// Hands the dragged picture on with where it was let go. Answers whether
/// it was taken.
fn perform(view: &DropView, info: &ProtocolObject<dyn NSDraggingInfo>) -> bool {
  let forms = forms(&info.draggingPasteboard());
  let Some(window) = view.window() else {
    return false;
  };
  if forms.is_empty() {
    return false;
  }
  let on_screen = window.convertPointToScreen(info.draggingLocation());
  super::dropped(forms, on_screen, window.backingScaleFactor());
  true
}

impl DropView {
  pub(super) fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
    let this = Self::alloc(mtm).set_ivars(());
    // Safety: NSView's designated initialiser.
    let view: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
    // Safety: the types are AppKit's own constants.
    let types = unsafe {
      NSArray::from_slice(&[
        NSPasteboardTypePNG,
        NSPasteboardTypeTIFF,
        NSPasteboardTypeFileURL,
        NSPasteboardTypeURL,
      ])
    };
    view.registerForDraggedTypes(&types);
    view
  }
}

fn operation(info: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
  if carries_picture(&info.draggingPasteboard()) {
    NSDragOperation::Copy
  } else {
    NSDragOperation::None
  }
}

/// The file extensions a dragged file or address is taken as a picture by.
const PICTURE_EXTENSIONS: [&str; 9] = [
  "png", "jpg", "jpeg", "gif", "webp", "tif", "tiff", "bmp", "heic",
];

fn is_picture_path(path: &str) -> bool {
  let path = path.split(['?', '#']).next().unwrap_or(path);
  path.rsplit_once('.').is_some_and(|(_, extension)| {
    PICTURE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
  })
}

/// A picture file on `pasteboard`.
fn picture_file(pasteboard: &NSPasteboard) -> Option<String> {
  // Safety: the type is AppKit's own constant.
  pasteboard
    .stringForType(unsafe { NSPasteboardTypeFileURL })
    .and_then(|url| NSURL::URLWithString(&url))
    .and_then(|url| url.path())
    .map(|path| path.to_string())
    .filter(|path| is_picture_path(path))
}

/// A picture's address on the web on `pasteboard`.
fn picture_address(pasteboard: &NSPasteboard) -> Option<String> {
  // Safety: the type is AppKit's own constant.
  pasteboard
    .stringForType(unsafe { NSPasteboardTypeURL })
    .map(|address| address.to_string())
    .filter(|address| address.starts_with("http") && is_picture_path(address))
}

/// Whether `pasteboard` holds a picture in a form a reference can open,
/// asked without copying the picture: it is asked as the pointer moves.
pub(super) fn carries_picture(pasteboard: &NSPasteboard) -> bool {
  // Safety: the types are AppKit's own constants.
  let bytes = unsafe { NSArray::from_slice(&[NSPasteboardTypePNG, NSPasteboardTypeTIFF]) };
  pasteboard.availableTypeFromArray(&bytes).is_some()
    || picture_file(pasteboard).is_some()
    || picture_address(pasteboard).is_some()
}

/// The picture on `pasteboard` in every form it offers that a reference can
/// open, the one that keeps the most of it first: its own bytes, a picture
/// file, then its address on the web.
fn forms(pasteboard: &NSPasteboard) -> Vec<Dropped> {
  // Safety: the types are AppKit's own constants.
  let bytes = unsafe { [NSPasteboardTypePNG, NSPasteboardTypeTIFF] };
  let mut forms: Vec<Dropped> = bytes
    .into_iter()
    .filter_map(|kind| pasteboard.dataForType(kind))
    .map(|data| Dropped::Data(data.to_vec()))
    .collect();
  forms.extend(picture_file(pasteboard).map(|path| Dropped::File(path.into())));
  forms.extend(picture_address(pasteboard).map(Dropped::Address));
  forms
}
