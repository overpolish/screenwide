// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pasteboard as a box being typed into copies to and pastes from it.

use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;

/// Puts `text` on the general pasteboard.
pub(super) fn copy(text: &str) {
  let pasteboard = NSPasteboard::generalPasteboard();
  pasteboard.clearContents();
  // Safety: the type is one of AppKit's own constants.
  let kind = unsafe { NSPasteboardTypeString };
  pasteboard.setString_forType(&NSString::from_str(text), kind);
}

/// The text on the general pasteboard, if there is any.
pub(super) fn paste() -> Option<String> {
  let pasteboard = NSPasteboard::generalPasteboard();
  // Safety: as above.
  let kind = unsafe { NSPasteboardTypeString };
  pasteboard.stringForType(kind).map(|text| text.to_string())
}
