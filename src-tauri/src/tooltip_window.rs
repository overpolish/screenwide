// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Editor's tooltips, drawn in a window of their own.
//!
//! The Editor's preview is a native surface layered above the webview, so a
//! tooltip drawn in the page is covered by it. This one is a window instead:
//! one window serves every trigger, prebuilt during Windows startup and built
//! on first hover on macOS, then hidden and shown again from then on.
//!
//! It never takes the keyboard. The window is built unfocused and, on macOS,
//! converted into a non-activating panel that refuses key status outright, so
//! a tooltip appearing cannot move focus out of the control it describes.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow};

use crate::editor::export_window::presentation;
use crate::windows::{platform, WindowLabel};

mod creation;

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

/// The tooltip window is prebuilt on Windows so an IPC callback never creates
/// a WebView synchronously. macOS keeps its lazy first-use construction.
fn get_or_create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  let label = WindowLabel::Tooltip.as_str();
  if let Some(window) = app.get_webview_window(label) {
    return Ok(window);
  }
  #[cfg(target_os = "windows")]
  {
    Err(tauri::Error::Anyhow(
      std::io::Error::other("The Windows tooltip window was not initialized").into(),
    ))
  }
  #[cfg(not(target_os = "windows"))]
  creation::build(app)
}

#[cfg(target_os = "windows")]
pub fn initialize(app: &AppHandle) -> tauri::Result<()> {
  creation::initialize(app)
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
  platform::raise_without_activation(window)?;
  #[cfg(not(target_os = "macos"))]
  {
    // Windows conceals by cloaking the window, which stays fully opaque
    // underneath, and is raised without ever being activated.
    platform::show(window, 1.0)?;
    platform::raise_without_activation(window)?;
  }
  // `platform::show` exists for the recording overlays, which take the
  // pointer. A tooltip does not.
  window.set_ignore_cursor_events(true)?;
  Ok(())
}

/// Asks for a tooltip over `anchor`, which is in the asking window's own
/// logical coordinates. A tooltip already on screen is only told the new
/// words: the move comes with the next fit, so crossing from one button to
/// its neighbour does not blink.
#[tauri::command]
pub fn show_tooltip(
  app: AppHandle,
  content: TooltipContent,
  anchor: AnchorRect,
  parent_window_label: String,
) -> Result<(), String> {
  let parent = app
    .get_webview_window(&parent_window_label)
    .ok_or_else(|| "The window asking for a tooltip is gone".to_owned())?;
  let scale = parent.scale_factor().map_err(|error| error.to_string())?;
  let origin = parent
    .outer_position()
    .map_err(|error| error.to_string())?
    .to_logical::<f64>(scale);
  let anchor = AnchorRect {
    x: origin.x + anchor.x,
    y: origin.y + anchor.y,
    ..anchor
  };

  let window = get_or_create(&app).map_err(|error| error.to_string())?;
  // A control on the live annotation overlay's toolbar sits above every
  // ordinary window, so the tooltip describing it has to clear that toolbar.
  // Decided per tooltip rather than kept as state: the overlay comes and goes
  // while this window is reused for the whole session.
  #[cfg(target_os = "macos")]
  platform::set_above_capture_overlays(&window, crate::annotate::is_active(&app))
    .map_err(|error| error.to_string())?;
  let was_visible = window.is_visible().unwrap_or(false);
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

  // A reused window is already loaded and would otherwise still be showing the
  // last tooltip; a fresh one has no listener yet and reads the words on mount.
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

/// Where a tooltip of `size` belongs: centred under its anchor, flipped above
/// it when there is no room below, and never off the edge of the display the
/// anchor is on. Computed from the requested size, because macOS has queued
/// the resize rather than applied it.
fn placement(
  app: &AppHandle,
  parent_label: &str,
  anchor: AnchorRect,
  size: LogicalSize<f64>,
) -> LogicalPosition<f64> {
  let mut position = LogicalPosition::new(
    anchor.x + (anchor.width - size.width) / 2.0,
    anchor.y + anchor.height + GAP,
  );

  let Some(monitor) = anchor_monitor(app, parent_label, anchor) else {
    return position;
  };
  let scale = monitor.scale_factor();
  let work_area = monitor.work_area();
  let origin = work_area.position.to_logical::<f64>(scale);
  let extent = work_area.size.to_logical::<f64>(scale);

  if position.y + size.height > origin.y + extent.height {
    position.y = anchor.y - size.height - GAP;
  }
  let max_x = origin.x + (extent.width - size.width).max(0.0);
  let max_y = origin.y + (extent.height - size.height).max(0.0);
  position.x = position.x.clamp(origin.x, max_x.max(origin.x));
  position.y = position.y.clamp(origin.y, max_y.max(origin.y));
  position
}

/// The display the described control is on, which is the one the tooltip has
/// to stay inside. Asked for by point rather than taken from the tooltip
/// window, which is still wherever the last tooltip left it.
fn anchor_monitor(
  app: &AppHandle,
  parent_label: &str,
  anchor: AnchorRect,
) -> Option<tauri::Monitor> {
  let scale = app
    .get_webview_window(parent_label)
    .and_then(|parent| parent.scale_factor().ok())
    .unwrap_or(1.0);
  let center = (
    (anchor.x + anchor.width / 2.0) * scale,
    (anchor.y + anchor.height / 2.0) * scale,
  );
  match app.monitor_from_point(center.0, center.1) {
    Ok(Some(monitor)) => Some(monitor),
    _ => app.primary_monitor().ok().flatten(),
  }
}

/// Takes the tooltip away. The window is nobody's child, so there is nothing
/// to detach first.
#[tauri::command]
pub fn hide_tooltip(app: AppHandle) -> Result<(), String> {
  let Some(window) = app.get_webview_window(WindowLabel::Tooltip.as_str()) else {
    return Ok(());
  };
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
