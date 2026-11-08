// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Editor's tooltips, drawn in a window of their own.
//!
//! The Editor's preview is a native surface layered above the webview, so a
//! tooltip drawn in the page is covered by it. This one is a window instead:
//! one window serves every trigger, built on first hover, then hidden and
//! shown again from then on.
//!
//! It never takes the keyboard. The window is built unfocused and, on macOS,
//! converted into a non-activating panel that refuses key status outright, so
//! a tooltip appearing cannot move focus out of the control it describes.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, WebviewWindow};

use crate::app_windows::options::presentation;
use crate::app_windows::{platform, WindowLabel};

mod creation;
mod placement;
use placement::placement;

/// Sent to an already-loaded tooltip when it is asked to say something else.
/// The first presentation has no one listening yet and reads the words itself.
const SHOWN_EVENT: &str = "tooltip://text";

/// The space between the control and the tooltip that describes it.
const GAP: f64 = 4.0;

/// The widest a tooltip gets. The window starts this wide so the page can lay
/// its words out at their natural width and report it; the fit then takes the
/// window down to what was actually measured.
const MAX_WIDTH: f64 = 360.0;
/// Where the window starts before that fit lands. Tall enough for a wrapped
/// label, and never seen: the window is concealed until it has been fitted.
const INITIAL_HEIGHT: f64 = 96.0;

/// What a tooltip says: a label, and the shortcut that does the same thing.
/// The shortcut is carried apart from the label so the window can draw it as
/// the same keycap the in-page tooltip uses rather than as run-on text.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TooltipContent {
  pub label: String,
  pub shortcut: Option<String>,
}

/// The control the tooltip describes, in logical pixels. Arrives relative to
/// the asking window's content and is stored relative to the screen.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorRect {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

struct PendingTooltip {
  content: TooltipContent,
  /// Screen coordinates, so the placement does not depend on the asking
  /// window staying where it was.
  anchor: AnchorRect,
  /// The window that asked, kept for the scale factor the anchor was
  /// measured at.
  parent_label: String,
}

#[derive(Default)]
pub struct TooltipState(Mutex<Option<PendingTooltip>>);

fn pending_tooltip(app: &AppHandle) -> Option<(AnchorRect, String)> {
  app
    .state::<TooltipState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .map(|pending| (pending.anchor, pending.parent_label.clone()))
}

/// The tooltip window, built on first hover. macOS builds it there and then.
/// WebView2 cannot create a webview while the main thread answers a command,
/// so Windows builds it in the background and has none to give this time; the
/// tooltip asked for is shown once it is built, if it is still wanted.
fn get_or_create(app: &AppHandle) -> tauri::Result<Option<WebviewWindow>> {
  if let Some(window) = app.get_webview_window(WindowLabel::Tooltip.as_str()) {
    return Ok(Some(window));
  }
  #[cfg(target_os = "windows")]
  {
    creation::build_in_background(app);
    Ok(None)
  }
  #[cfg(not(target_os = "windows"))]
  creation::build(app).map(Some)
}

/// Puts the window on screen without its pixels, so the page lays out and can
/// report the size the words need. A hidden window's webview does not lay out,
/// which is why this is a concealment rather than a hide.
fn present_concealed(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
  presentation::conceal(app, window)?;
  // macOS conceals by taking the alpha away, so the panel is only ordered
  // front here: `platform::show` would set an alpha of its own through the
  // event loop, which has no ordering against the dispatch queue the reveal
  // is made on, and could take the tooltip back to nothing after it.
  #[cfg(target_os = "macos")]
  {
    platform::raise_without_activation(window)?;
    window.set_ignore_cursor_events(true)?;
  }
  // Windows conceals by cloaking the window, which stays fully opaque
  // underneath. Shown through tao it would be activated, which blurs the
  // window asking and so closes the tooltip it asked for. It keeps the
  // pointer passthrough it was built with: changing it, a tao flag, while the
  // window is shown would hide it again.
  #[cfg(target_os = "windows")]
  {
    platform::show_without_activation(window)?;
    platform::raise_without_activation(window)?;
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    platform::show(window, 1.0)?;
    window.set_ignore_cursor_events(true)?;
  }
  Ok(())
}

/// Asks for a tooltip over `anchor`, which is in the asking window's own
/// logical coordinates. The window is revealed by the fit that follows, once
/// the words are sized and placed.
#[tauri::command]
pub fn show_tooltip(
  app: AppHandle,
  content: TooltipContent,
  anchor: AnchorRect,
  parent_window_label: String,
  // The asking page's height, which says where it begins in its window.
  page_height: Option<f64>,
) -> Result<(), String> {
  let parent = app
    .get_webview_window(&parent_window_label)
    .ok_or_else(|| "The window asking for a tooltip is gone".to_owned())?;
  // The anchor is measured in the page, so it is placed from where the page
  // begins, below any title bar the window has of its own.
  let origin = crate::app_windows::geometry::page_origin(&parent, page_height)
    .map_err(|error| error.to_string())?;
  let anchor = AnchorRect {
    x: origin.x + anchor.x,
    y: origin.y + anchor.y,
    ..anchor
  };

  // Kept first: a window built for it reads the words on mount.
  {
    let state = app.state::<TooltipState>();
    let mut pending = state
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    *pending = Some(PendingTooltip {
      anchor,
      content: content.clone(),
      parent_label: parent_window_label,
    });
  }
  let Some(window) = get_or_create(&app).map_err(|error| error.to_string())? else {
    return Ok(());
  };
  // A control on the live annotation overlay's toolbar sits above every
  // ordinary window, so the tooltip describing it has to clear that toolbar.
  // Decided per tooltip rather than kept as state: the overlay comes and goes
  // while this window is reused for the whole session.
  #[cfg(target_os = "macos")]
  platform::set_above_capture_overlays(&window, crate::annotate::is_active(&app))
    .map_err(|error| error.to_string())?;
  let was_visible = window.is_visible().unwrap_or(false);

  // A tooltip already on screen is hidden first: shown at once, the new words
  // would sit in the old bubble, at its old size and place, for the round
  // trip until the fit moves it. A reused window is already loaded; a fresh
  // one has no listener yet and reads the words on mount.
  if was_visible {
    presentation::conceal(&app, &window).map_err(|error| error.to_string())?;
  }
  let _ = app.emit_to(WindowLabel::Tooltip.as_str(), SHOWN_EVENT, content);
  if !was_visible {
    present_concealed(&app, &window).map_err(|error| error.to_string())?;
  }
  Ok(())
}

/// The tooltip's content reporting the size it needs, which is when the window
/// is sized, placed under its anchor and revealed.
#[tauri::command]
pub fn fit_tooltip(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
  if !width.is_finite() || width <= 0.0 || !height.is_finite() || height <= 0.0 {
    return Err("The tooltip size must be positive".to_owned());
  }
  let Some((anchor, parent_label)) = pending_tooltip(&app) else {
    return Ok(());
  };
  let Some(window) = app.get_webview_window(WindowLabel::Tooltip.as_str()) else {
    return Ok(());
  };
  let size = LogicalSize::new(width.ceil().min(MAX_WIDTH), height.ceil());
  window.set_size(size).map_err(|error| error.to_string())?;
  window
    .set_position(placement(&app, &parent_label, anchor, size))
    .map_err(|error| error.to_string())?;
  presentation::reveal_after_resize(&window).map_err(|error| error.to_string())
}

/// Takes the tooltip away. The window is nobody's child, so there is nothing
/// to detach first. What it was to say is forgotten too, so a window still
/// being built for it, or a fit still on its way, does not bring it back.
#[tauri::command]
pub fn hide_tooltip(app: AppHandle) -> Result<(), String> {
  *app
    .state::<TooltipState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
  let Some(window) = app.get_webview_window(WindowLabel::Tooltip.as_str()) else {
    return Ok(());
  };
  #[cfg(target_os = "windows")]
  return platform::hide_without_activation(&window).map_err(|error| error.to_string());
  #[cfg(not(target_os = "windows"))]
  platform::hide(&window).map_err(|error| error.to_string())
}

/// What the tooltip should say. Read once on mount; every later tooltip
/// arrives as an event instead.
#[tauri::command]
pub fn get_tooltip(app: AppHandle) -> Option<TooltipContent> {
  app
    .state::<TooltipState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .map(|pending| pending.content.clone())
}
