// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, NSObjectProtocol};
use objc2_app_kit::NSView;

/// The webview among `host`'s children, which native views are layered
/// against. Wry draws in a `WKWebView` subclass with a name of its own, and
/// key-value observing renames that class again, so it is found by kind.
pub(crate) fn webview_view(host: &NSView) -> Option<Retained<NSView>> {
  let webview = AnyClass::get(c"WKWebView")?;
  host
    .subviews()
    .iter()
    .find(|view| view.isKindOfClass(webview))
}
