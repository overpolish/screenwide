// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The teleprompter's window: one, floating above every app and docked at
//! the top of the main display, just under the menu bar where the camera
//! is, so the eyes reading it are close to the lens. It holds its own
//! controls, is left out of every capture, the app's own recordings and
//! other apps' screen sharing alike, and fills its display when asked, for
//! a teleprompter's own monitor. Opened on a script, it asks which once its
//! page has loaded, as an event sent before then is lost.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WindowEvent};

pub(crate) const LABEL: &str = "teleprompter";
/// Tells the teleprompter to show another script, by its project's
/// manifest.
const SCRIPT_EVENT: &str = "teleprompter://script";
/// Tells the teleprompter the script it shows was renamed, by its new
/// manifest.
const MOVED_EVENT: &str = "teleprompter://moved";
/// The window's size as it first opens, in points: a few lines of large
/// text under its controls, narrow enough to keep the eyes near the camera.
const SIZE: (f64, f64) = (680.0, 280.0);
/// As narrow as its controls fit.
const MIN_SIZE: (f64, f64) = (520.0, 180.0);
/// How far below the top of the work area it opens, in points.
const DOCK_GAP: f64 = 8.0;

/// The script being prompted, by its project's manifest.
static SCRIPT: Mutex<Option<PathBuf>> = Mutex::new(None);

fn shown() -> std::sync::MutexGuard<'static, Option<PathBuf>> {
  SCRIPT.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Opens the teleprompter on the script of `project`, or shows that script
/// in it if it is open, as there is only ever one. A trial or licence that
/// does not allow it opens Settings on Licences instead.
pub(crate) fn open(app: &AppHandle, project: &Path) -> Result<(), String> {
  if !crate::licensing::allow(app, crate::licensing::Tool::Teleprompter) {
    return Ok(());
  }
  *shown() = Some(project.to_owned());
  if let Some(window) = app.get_webview_window(LABEL) {
    window
      .emit(SCRIPT_EVENT, project)
      .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    return window.set_focus().map_err(|error| error.to_string());
  }
  build(app)
}

/// The script the teleprompter shows, which its page asks for as it loads.
#[tauri::command]
pub fn teleprompter_script() -> Option<PathBuf> {
  shown().clone()
}

/// A project renamed: the teleprompter follows the script it shows to its
/// new manifest, so what is typed next is saved there.
pub(crate) fn renamed(app: &AppHandle, from: &Path, to: &Path) {
  let mut script = shown();
  if script.as_deref() != Some(from) {
    return;
  }
  *script = Some(to.to_owned());
  drop(script);
  let _ = app.emit_to(LABEL, MOVED_EVENT, to);
}

/// A project set aside or moved to another location: the teleprompter
/// closes first if it shows its script, as an editor does, so nothing is
/// saved to the folder while it goes.
pub(crate) fn close_on(app: &AppHandle, project: &Path) {
  if shown().as_deref() != Some(project) {
    return;
  }
  if let Some(window) = app.get_webview_window(LABEL) {
    let _ = window.close();
  }
}

/// Fills the display the teleprompter is on, or gives it back its size and
/// place. On macOS it does so without a space of its own, so it stays
/// above other apps and on every desktop as it was.
#[tauri::command]
pub fn set_teleprompter_fullscreen(app: AppHandle, fullscreen: bool) -> Result<(), String> {
  let window = app
    .get_webview_window(LABEL)
    .ok_or_else(|| "The teleprompter is not open".to_owned())?;
  window
    .set_simple_fullscreen(fullscreen)
    .map_err(|error| error.to_string())
}

fn build(app: &AppHandle) -> Result<(), String> {
  let window =
    crate::app_windows::webview_window(app, LABEL, WebviewUrl::App("/teleprompter".into()))
      .title(screenwide_i18n::t!("window-titles-teleprompter"))
      .accept_first_mouse(true)
      .always_on_top(true)
      .decorations(false)
      .inner_size(SIZE.0, SIZE.1)
      .min_inner_size(MIN_SIZE.0, MIN_SIZE.1)
      .resizable(true)
      .shadow(false)
      .skip_taskbar(true)
      .transparent(true)
      .visible(false)
      .visible_on_all_workspaces(true)
      .build()
      .map_err(|error| error.to_string())?;
  if let Some(position) = docked(app) {
    window
      .set_position(position)
      .map_err(|error| error.to_string())?;
  }
  crate::app_windows::set_window_capture_affinity(&window, false)
    .map_err(|error| error.to_string())?;
  let handle = app.clone();
  window.on_window_event(move |event| match event {
    WindowEvent::Moved(_) => super::cue_window::follow(&handle, false),
    WindowEvent::Resized(_) => super::cue_window::follow(&handle, true),
    WindowEvent::Destroyed => {
      *shown() = None;
      super::voice::stop();
      super::cue_window::close(&handle);
    }
    _ => {}
  });
  window.show().map_err(|error| error.to_string())?;
  window.set_focus().map_err(|error| error.to_string())
}

/// Where the window opens: centred at the top of the main display's work
/// area, under the menu bar and the camera above it. None where the display
/// cannot be read, which leaves the system to place it.
fn docked(app: &AppHandle) -> Option<PhysicalPosition<i32>> {
  let monitor = app.primary_monitor().ok().flatten()?;
  let area = monitor.work_area();
  let scale = monitor.scale_factor();
  let width = (SIZE.0 * scale).round() as i32;
  let gap = (DOCK_GAP * scale).round() as i32;
  let area_width = i32::try_from(area.size.width).ok()?;
  Some(PhysicalPosition::new(
    area.position.x + (area_width - width) / 2,
    area.position.y + gap,
  ))
}
