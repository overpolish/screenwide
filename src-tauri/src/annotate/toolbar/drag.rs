// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the toolbar is dropped, settled as the recording bar's is: on the
//! release it is pulled back inside the screen it mostly covers, and that is
//! the place kept.

use tauri::{AppHandle, LogicalPosition, WebviewWindow};

use super::super::settings::{self, ToolbarPosition};
use super::{anchor, toolbar};

/// How far, in logical px, the window may sit from where the toolbar would be
/// placed and still count as not moved: the round trip through physical
/// pixels can land a fraction off.
const UNMOVED: f64 = 1.0;

/// Whether the window is somewhere other than where the toolbar would be
/// placed now, which only a drag puts it. A press that is not a drag - a
/// click on a tool - has to leave the kept place alone: a toolbar slid in
/// from the edge to fit a wider tool is not where it was dropped, and
/// keeping that would stop a narrower tool from putting it back.
fn moved_by_hand(window: &WebviewWindow) -> Result<bool, String> {
  let guard = anchor::current();
  let Some(anchor) = guard.as_ref() else {
    return Ok(false);
  };
  let scale = window.scale_factor().map_err(|error| error.to_string())?;
  let position = window
    .outer_position()
    .map_err(|error| error.to_string())?
    .to_logical::<f64>(scale);
  let size = window
    .outer_size()
    .map_err(|error| error.to_string())?
    .to_logical::<f64>(scale);
  let placed = anchor::placement(anchor, size);
  Ok((position.x - placed.x).abs() > UNMOVED || (position.y - placed.y).abs() > UNMOVED)
}

/// The toolbar released after a press: a drag that ended with any of it off
/// its display is pulled back on, and the place it settled in is kept against
/// that display, so it comes back there on that screen and top-centre on any
/// other.
#[tauri::command]
pub fn finish_annotate_toolbar_drag(app: AppHandle) -> Result<(), String> {
  let Some(window) = toolbar(&app) else {
    return Ok(());
  };
  if !moved_by_hand(&window)? {
    return Ok(());
  }
  crate::app_windows::contain_window_on_its_monitor(&app, &window)
    .map_err(|error| error.to_string())?;
  let scale = window.scale_factor().map_err(|error| error.to_string())?;
  let position: LogicalPosition<f64> = window
    .outer_position()
    .map_err(|error| error.to_string())?
    .to_logical(scale);
  let Some((display_id, origin)) = anchor::display_under(&app, position)? else {
    return Ok(());
  };
  settings::store_toolbar_position(
    &app,
    ToolbarPosition {
      display_id,
      x: position.x - origin.x,
      y: position.y - origin.y,
    },
  )
}

/// Settles every drag of the toolbar's window once the button is released.
#[cfg(target_os = "windows")]
pub(super) fn follow_drags(app: &AppHandle, window: &WebviewWindow) {
  use std::sync::atomic::AtomicBool;

  static DRAG_ACTIVE: AtomicBool = AtomicBool::new(false);
  let app = app.clone();
  window.on_window_event(move |event| {
    if !matches!(event, tauri::WindowEvent::Moved(_)) {
      return;
    }
    let app = app.clone();
    crate::app_windows::drag_release::after_mouse_up(&DRAG_ACTIVE, move || {
      if let Err(error) = finish_annotate_toolbar_drag(app) {
        eprintln!("Could not settle the annotate toolbar: {error}");
      }
    });
  });
}
