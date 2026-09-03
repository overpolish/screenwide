// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window an animation is aimed at. Two kinds arrive at the same tween: a
//! foreign window, read and written through the Accessibility API, and one of
//! Screenwide's own, read and written through Tauri. Everything above this file
//! works in logical rectangles and never learns which it is holding.

use cidre::{arc, ax, cf, cg};
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
use tauri::{LogicalPosition, LogicalSize, WebviewWindow};

use super::super::own_window::{decoration_size, logical_frame};

/// The longest a single Accessibility set may block the tween thread. The
/// default runs to seconds, which an unresponsive application would spend the
/// whole animation inside; this bounds the stall while staying far above the
/// step interval, so a merely slow application still animates.
const AX_TIMEOUT_SECS: f32 = 0.25;

/// A window an animation is aimed at, carried across threads.
///
/// SAFETY: the `Ax` variant wraps an `AXUIElementRef`, a Core Foundation
/// object, and Apple's Accessibility API documents its functions as safe to
/// call from any thread - the framework serialises the messaging to the target
/// application itself. `arc::R` owns a retain, so the reference stays valid for
/// as long as the wrapper lives, and nothing here hands out a borrow: every
/// attribute get and set goes through `&self`/`&mut self` on one thread at a
/// time, behind the tween mutex or the session mutex that owns the wrapper. The
/// `Own` variant wraps a `tauri::WebviewWindow`, which is `Send + Clone` in its
/// own right and needs nothing from this impl; the unsafe impl is written by
/// hand only because the `Ax` variant cannot derive it.
pub(in crate::glide) enum WindowTarget {
  Ax(arc::R<ax::UiElement>),
  /// Boxed only for size: a `WebviewWindow` is nearly a kilobyte, and the
  /// Accessibility variant is a single pointer.
  Own(Box<WebviewWindow>),
}

unsafe impl Send for WindowTarget {}

impl WindowTarget {
  /// A foreign window, as the Accessibility hit test resolved it.
  pub(in crate::glide) fn new(element: arc::R<ax::UiElement>) -> Self {
    let _ = element.set_messaging_timeout_secs(AX_TIMEOUT_SECS);
    Self::Ax(element)
  }

  /// One of Screenwide's own windows, as the allowlist resolved it.
  pub(in crate::glide) fn own(window: WebviewWindow) -> Self {
    Self::Own(Box::new(window))
  }

  /// A second handle on the same window, for the caller that needs one outside
  /// the lock the original is stored behind.
  pub(in crate::glide) fn duplicate(&self) -> Self {
    match self {
      Self::Ax(element) => Self::Ax(element.retained()),
      Self::Own(window) => Self::Own(window.clone()),
    }
  }

  /// Brings the window to the front: its application is activated and the
  /// window itself raised above the application's other windows.
  pub(in crate::glide) fn raise(&self) {
    match self {
      Self::Ax(element) => {
        if let Some(app) = element
          .pid()
          .ok()
          .and_then(NSRunningApplication::runningApplicationWithProcessIdentifier)
        {
          #[allow(deprecated)]
          app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
        }
        if let Err(error) = element.perform_action(ax::action::raise()) {
          eprintln!("Could not raise the window: {error}");
        }
      }
      Self::Own(window) => {
        if let Err(error) = window.set_focus() {
          eprintln!("Could not raise the window: {error}");
        }
      }
    }
  }

  pub(in crate::glide) fn frame(&self) -> Option<cg::Rect> {
    match self {
      Self::Ax(element) => element.frame().ok().and_then(|value| value.cg_rect()),
      Self::Own(window) => logical_frame(window),
    }
  }

  /// Minimizes the captured target without hit-testing the session anchor
  /// again after the window has moved away from it.
  pub(in crate::glide) fn minimize(&mut self) {
    match self {
      Self::Ax(element) => {
        if !element.is_settable(ax::attr::minimized()).unwrap_or(false) {
          eprintln!("The application does not allow minimizing this window");
          return;
        }
        if let Err(error) = element.set_attr(ax::attr::minimized(), cf::Boolean::value_true()) {
          eprintln!("Could not minimize the window: {error}");
        }
      }
      Self::Own(window) => {
        if let Err(error) = window.minimize() {
          eprintln!("Could not minimize the window: {error}");
        }
      }
    }
  }

  /// Whether the window can be moved at all. A foreign application may refuse;
  /// one of ours is ours to place.
  pub(in crate::glide) fn is_movable(&self) -> bool {
    match self {
      Self::Ax(element) => element.is_settable(ax::attr::pos()).unwrap_or(false),
      Self::Own(_) => true,
    }
  }

  /// Whether the window can be resized, which decides whether a placement may
  /// change its size on the way.
  pub(in crate::glide) fn is_resizable(&self) -> bool {
    match self {
      Self::Ax(element) => element.is_settable(ax::attr::size()).unwrap_or(false),
      Self::Own(window) => window.is_resizable().unwrap_or(false),
    }
  }

  /// Writes one origin onto the window, reporting whether it took.
  pub(in crate::glide) fn set_pos(&mut self, origin: &cg::Point) -> bool {
    match self {
      Self::Ax(element) => {
        let value = ax::Value::with_cg_point(origin);
        if let Err(error) = element.set_attr(ax::attr::pos(), value.as_ref()) {
          eprintln!("Could not move the window: {error}");
          return false;
        }
        true
      }
      Self::Own(window) => {
        if let Err(error) = window.set_position(LogicalPosition::new(origin.x, origin.y)) {
          eprintln!("Could not move the window: {error}");
          return false;
        }
        true
      }
    }
  }

  /// Writes one size onto the window, reporting whether it took. The size is an
  /// outer one, the same as `frame` reports: Tauri sizes its windows by their
  /// inner extent, so the decorations come off first.
  pub(in crate::glide) fn set_size(&mut self, size: &cg::Size) -> bool {
    match self {
      Self::Ax(element) => {
        let value = ax::Value::with_cg_size(size);
        if let Err(error) = element.set_attr(ax::attr::size(), value.as_ref()) {
          eprintln!("Could not resize the window: {error}");
          return false;
        }
        true
      }
      Self::Own(window) => {
        let Some(decoration) = decoration_size(window) else {
          eprintln!("Could not measure the window to resize");
          return false;
        };
        let inner = LogicalSize::new(
          (size.width - decoration.width).max(0.0),
          (size.height - decoration.height).max(0.0),
        );
        if let Err(error) = window.set_size(inner) {
          eprintln!("Could not resize the window: {error}");
          return false;
        }
        true
      }
    }
  }
}
