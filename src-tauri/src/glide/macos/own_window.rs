// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Screenwide's own windows, resolved without the Accessibility API. Asking
//! that API about our own process makes the tap thread interrogate our own busy
//! run loop through WebKit's remote proxies, which answers with husks - no
//! role, no subrole - so hit-testing and writing both go native here. Tauri
//! knows exactly where its windows are, how tall their chrome is and how to
//! move them, none of which needs guessing.
//!
//! Which of our windows may be carried is decided by name, not by asking macOS
//! what kind of window it is: every overlay Screenwide puts on screen - the
//! Glide preview above all - is excluded by construction rather than by a
//! classification that could go the wrong way.

use cidre::cg;
use core_graphics::geometry::CGPoint;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewWindow};

use super::titlebar::{ax_titlebar_at, AxTitlebar, CHROME_MARGIN, FALLBACK_TITLEBAR_HEIGHT};
use crate::windows::WindowLabel;

/// Whether a point is on any titlebar a gesture may act on - a foreign one, or
/// one of our own. The Accessibility hit test goes first because it is the one
/// probe already paid for on every sample; the native resolver's main-thread
/// hops are spent only when the hit says the point is over Screenwide itself.
pub(super) fn any_titlebar(app: &AppHandle, point: CGPoint) -> bool {
  match ax_titlebar_at(point) {
    AxTitlebar::Titlebar(..) => true,
    AxTitlebar::OwnProcess => is_own_titlebar(app, point),
    AxTitlebar::Miss => false,
  }
}

/// The windows of ours a gesture is allowed to pick up. Everything absent from
/// this list is an overlay, a panel or a selector - chrome of the application
/// itself, which a window-management gesture must stay clear of. It is a
/// function rather than a bare constant because the list is expected to grow as
/// Screenwide gains ordinary windows.
fn glidable_labels() -> &'static [WindowLabel] {
  &[
    WindowLabel::Settings,
    WindowLabel::RecordingBar,
    WindowLabel::RecordingDock,
    WindowLabel::ExportRecording,
    WindowLabel::ExportScreenshot,
  ]
}

/// The allowlisted window of ours whose titlebar a global point lands on, with
/// its logical outer frame - the same rectangle, in the same space, that the
/// Accessibility path reports for a foreign window.
pub(super) fn own_window_at(app: &AppHandle, point: CGPoint) -> Option<(WebviewWindow, cg::Rect)> {
  glidable_labels().iter().find_map(|label| {
    let window = app.get_webview_window(label.as_str())?;
    titlebar_hit(&window, point).map(|frame| (window.clone(), frame))
  })
}

/// Whether a point is on the titlebar of one of our own windows. The tap-side
/// callers only need the verdict, and phrasing it this way keeps the own path
/// beside the Accessibility `is_titlebar` it stands next to.
pub(super) fn is_own_titlebar(app: &AppHandle, point: CGPoint) -> bool {
  own_window_at(app, point).is_some()
}

/// A window's logical outer frame: where it is and how big it is, decorations
/// included, in the top-left-origin space Accessibility geometry also uses.
pub(super) fn logical_frame(window: &WebviewWindow) -> Option<cg::Rect> {
  let scale = window.scale_factor().ok()?;
  let position: LogicalPosition<f64> = window.outer_position().ok()?.to_logical(scale);
  let size: LogicalSize<f64> = window.outer_size().ok()?.to_logical(scale);
  Some(cg::Rect {
    origin: cg::Point {
      x: position.x,
      y: position.y,
    },
    size: cg::Size {
      width: size.width,
      height: size.height,
    },
  })
}

/// How much larger the window is than the page inside it. Tauri sizes windows
/// by their inner extent, so a caller placing an outer rectangle takes this off
/// first; a window with no native decorations reports zero, which is the truth.
pub(super) fn decoration_size(window: &WebviewWindow) -> Option<LogicalSize<f64>> {
  let scale = window.scale_factor().ok()?;
  let outer: LogicalSize<f64> = window.outer_size().ok()?.to_logical(scale);
  let inner: LogicalSize<f64> = window.inner_size().ok()?.to_logical(scale);
  Some(LogicalSize::new(
    outer.width - inner.width,
    outer.height - inner.height,
  ))
}

/// The frame of a window the point lands on the titlebar of. A hidden or
/// minimized window is nowhere, whatever its last frame said.
fn titlebar_hit(window: &WebviewWindow, point: CGPoint) -> Option<cg::Rect> {
  if !window.is_visible().ok()? || window.is_minimized().ok()? {
    return None;
  }
  let frame = logical_frame(window)?;
  let bottom = chrome_bottom(window, frame)?;
  let hit = point.x >= frame.origin.x
    && point.x <= frame.origin.x + frame.size.width
    && point.y >= frame.origin.y
    && point.y <= bottom;
  hit.then_some(frame)
}

/// The lowest point that still starts a gesture. Tauri measures the real
/// decoration height for us - outer frame against inner frame, no heuristic -
/// but a window that draws its own titlebar in the page has decorations turned
/// off, so that measurement is zero and says nothing about the band the user
/// sees. Those fall back to the same height the Accessibility path uses for the
/// custom titlebars it cannot measure either.
fn chrome_bottom(window: &WebviewWindow, frame: cg::Rect) -> Option<f64> {
  let scale = window.scale_factor().ok()?;
  let inner: LogicalPosition<f64> = window.inner_position().ok()?.to_logical(scale);
  let native = inner.y - frame.origin.y;
  let chrome = if native > 0.0 {
    native
  } else {
    FALLBACK_TITLEBAR_HEIGHT
  };
  Some((frame.origin.y + chrome + CHROME_MARGIN).min(frame.origin.y + frame.size.height))
}
