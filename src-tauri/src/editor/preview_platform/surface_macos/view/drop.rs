// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures dropped on the workspace: the input view as a dragging
//! destination. The twin of the Windows OLE drop target.

use std::path::PathBuf;

use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
  NSDragOperation, NSDraggingInfo, NSPasteboard, NSPasteboardTypeFileURL, NSPasteboardTypePNG,
  NSPasteboardTypeTIFF,
};
use objc2_foundation::NSURL;

use super::super::mac;
use objc2::DefinedClass;

use super::EditorView;
use crate::editor::images::dropped::{DroppedPicture, ImagePoint};

/// Whether the view takes drops now: an editor showing, not covered by the
/// frontend's own chrome, with somewhere to hand the picture.
fn taking(view: &EditorView) -> bool {
  view.engine().is_some_and(|engine| {
    mac(&engine.inner)
      .image_drop
      .lock()
      .is_ok_and(|callback| callback.is_some())
      && engine.takes_drops()
  })
}

/// The dragged picture, in the form that keeps the most of it.
fn picture(pasteboard: &NSPasteboard) -> Option<DroppedPicture> {
  // Safety: the types are AppKit's own constants.
  let (file, png, tiff) = unsafe {
    (
      NSPasteboardTypeFileURL,
      NSPasteboardTypePNG,
      NSPasteboardTypeTIFF,
    )
  };
  let path = pasteboard
    .stringForType(file)
    .and_then(|url| NSURL::URLWithString(&url))
    .and_then(|url| url.path())
    .map(|path| PathBuf::from(path.to_string()));
  if let Some(path) = path {
    return Some(DroppedPicture::File(path));
  }
  [png, tiff]
    .into_iter()
    .find_map(|kind| pasteboard.dataForType(kind))
    .map(|data| DroppedPicture::Data(data.to_vec()))
}

pub(super) fn operation(view: &EditorView) -> NSDragOperation {
  if view.ivars().drop_accepts.get() {
    NSDragOperation::Copy
  } else {
    NSDragOperation::None
  }
}

pub(super) fn entered(
  view: &EditorView,
  info: &ProtocolObject<dyn NSDraggingInfo>,
) -> NSDragOperation {
  let accepts = taking(view) && picture(&info.draggingPasteboard()).is_some();
  view.ivars().drop_accepts.set(accepts);
  operation(view)
}

pub(super) fn perform(view: &EditorView, info: &ProtocolObject<dyn NSDraggingInfo>) -> bool {
  if !view.ivars().drop_accepts.replace(false) || !taking(view) {
    return false;
  }
  let Some(engine) = view.engine() else {
    return false;
  };
  let Some(picture) = picture(&info.draggingPasteboard()) else {
    return false;
  };
  let point = view.pixels(view.convertPoint_fromView(info.draggingLocation(), None));
  let at = engine
    .land_drop(point)
    .map(|(layer, x, y)| ImagePoint { layer, x, y });
  let Ok(mut callback) = mac(&engine.inner).image_drop.lock() else {
    return false;
  };
  let Some(callback) = callback.as_mut() else {
    return false;
  };
  callback(picture, at);
  true
}
