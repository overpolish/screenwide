// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "dock/first_reveal.rs"]
pub mod first_reveal;
#[path = "dock/positioning.rs"]
mod positioning;
use positioning::load_recording_dock_offset;
#[cfg(test)]
use positioning::recording_dock_local_position;
use positioning::recording_dock_monitor;
use positioning::recording_dock_offset;
use positioning::recording_dock_position;
use positioning::store_recording_dock_offset;

use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
#[cfg(target_os = "windows")]
use std::time::Duration;

use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use tauri::WindowEvent;
use tauri::{
  AppHandle, LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewWindow,
};

use super::{geometry::monitor_with_most_overlap, platform, WindowLabel};

const RECORDING_DOCK_POSITION_FILE: &str = "recording-dock-position.json";
const RECORDING_DOCK_MIN_WIDTH: f64 = 100.0;
const RECORDING_DOCK_MAX_WIDTH: f64 = 320.0;
const RECORDING_DOCK_HEIGHT: f64 = 40.0;
const RECORDING_DOCK_TOP_GAP: f64 = 8.0;

#[cfg(target_os = "windows")]
static DOCK_DRAG_ACTIVE: AtomicBool = AtomicBool::new(false);

/// The pill's position, held against the work area of the monitor it was
/// dropped on rather than as absolute desktop coordinates, so it lands in the
/// same visual spot whichever monitor the recording bar is on.
///
/// The pill is anchored at its horizontal centre, because its width follows
/// its state: the countdown, starting and recording layouts differ by up to a
/// hundred pixels, and a centre anchor grows and shrinks it in place instead
/// of sliding it. `y` is the top edge; the height never changes.
///
/// The offset is in *logical* pixels. Physical pixels would move the pill twice
/// as far from the corner when a 2x display's offset is applied to a 1x one,
/// and a proportional fraction would distort placement for a fixed-size window
/// that is meant to sit a fixed distance from an edge.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
struct RecordingDockOffset {
  centre_x: f64,
  y: f64,
}

/// Where the user dropped the pill, relative to the work area of the monitor
/// they dropped it on. `None` means it has never been dragged.
static RECORDING_DOCK_OFFSET: Mutex<Option<RecordingDockOffset>> = Mutex::new(None);
/// The position the pill was last placed at programmatically, so that a plain
/// click on its buttons is never mistaken for a drag.
static RECORDING_DOCK_PLACED: Mutex<Option<PhysicalPosition<i32>>> = Mutex::new(None);

#[cfg(not(target_os = "macos"))]
pub fn initialize_recording_dock(app: &AppHandle) -> tauri::Result<()> {
  load_recording_dock_offset(app);
  if let Some(window) = app.get_webview_window(WindowLabel::RecordingDock.as_str()) {
    platform::initialize_recording_dock(&window)?;
  }

  Ok(())
}

pub fn show_recording_dock(app: &AppHandle) -> tauri::Result<()> {
  load_recording_dock_offset(app);
  let dock = app
    .get_webview_window(WindowLabel::RecordingDock.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  if let Some(monitor) = recording_dock_monitor(app)? {
    let offset = *RECORDING_DOCK_OFFSET
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let scale = monitor.scale_factor();
    let dock_scale = dock.scale_factor()?;
    let logical_size = LogicalSize::new(
      dock.outer_size()?.to_logical::<f64>(dock_scale).width,
      RECORDING_DOCK_HEIGHT,
    );
    let dock_size = PhysicalSize::new(
      (logical_size.width * scale).round() as u32,
      (RECORDING_DOCK_HEIGHT * scale).round() as u32,
    );
    let position = recording_dock_position(&monitor, dock_size, offset);
    place_recording_dock(&dock, position, logical_size)?;
  }
  #[cfg(target_os = "macos")]
  let opacity = first_reveal::show_opacity(app);
  #[cfg(not(target_os = "macos"))]
  let opacity = 1.0;
  platform::show(&dock, opacity)?;
  platform::restore_recording_level(&dock)
}

#[tauri::command]
pub fn resize_recording_dock(app: AppHandle, width: f64) -> Result<(), String> {
  if !width.is_finite() {
    return Err("The recording pill width must be finite".to_owned());
  }

  let to_message = |error: tauri::Error| error.to_string();
  let dock = app
    .get_webview_window(WindowLabel::RecordingDock.as_str())
    .ok_or_else(|| "The recording pill is unavailable".to_owned())?;
  let old_position = dock.outer_position().map_err(to_message)?;
  let old_size = dock.outer_size().map_err(to_message)?;
  let logical_size = LogicalSize::new(
    width
      .ceil()
      .clamp(RECORDING_DOCK_MIN_WIDTH, RECORDING_DOCK_MAX_WIDTH),
    RECORDING_DOCK_HEIGHT,
  );
  // This runs whether or not the pill is visible: a resize racing the show can
  // still see it hidden, and a hidden pill is placed again on show anyway.
  let new_size: PhysicalSize<u32> =
    logical_size.to_physical(dock.scale_factor().map_err(to_message)?);
  let Some(monitor) = monitor_with_most_overlap(&app, &dock).map_err(to_message)? else {
    return dock.set_size(logical_size).map_err(to_message);
  };
  let placed = *RECORDING_DOCK_PLACED
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  // A pill still where it was last placed is placed afresh from the saved
  // anchor, exactly as showing it does, so the two can never disagree about
  // where it belongs. Only a pill mid-drag keeps its own centre.
  let position = if placed == Some(old_position) {
    let offset = *RECORDING_DOCK_OFFSET
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    recording_dock_position(&monitor, new_size, offset)
  } else {
    contained_position(
      &monitor,
      PhysicalPosition::new(
        old_position.x + (old_size.width as i32 - new_size.width as i32) / 2,
        old_position.y,
      ),
      new_size,
    )
  };
  place_recording_dock(&dock, position, logical_size).map_err(to_message)
}

/// Applies the pill's position and size as one frame and records it as placed.
///
/// Not `set_position`/`set_size`: on macOS tao queues both on the GCD main
/// queue, which has no ordering with `platform::show` (posted through the event
/// loop), so a fresh launch put the pill on screen at its default centred frame
/// before the move landed; and the two steps could present a stretched frame
/// before the shifted one. `platform::set_frame` is one AppKit call on the same
/// channel as the show. The frame is recorded as requested because reading it
/// back can precede a frame change that is still queued.
fn place_recording_dock(
  dock: &WebviewWindow,
  position: PhysicalPosition<i32>,
  size: LogicalSize<f64>,
) -> tauri::Result<()> {
  platform::set_frame(dock, position.to_logical(dock.scale_factor()?), size)?;
  *RECORDING_DOCK_PLACED
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(position);
  Ok(())
}

fn contained_position(
  monitor: &Monitor,
  position: PhysicalPosition<i32>,
  size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
  let work_area = monitor.work_area();
  let max_x = work_area.position.x + work_area.size.width.saturating_sub(size.width) as i32;
  let max_y = work_area.position.y + work_area.size.height.saturating_sub(size.height) as i32;
  PhysicalPosition::new(
    position
      .x
      .clamp(work_area.position.x, max_x.max(work_area.position.x)),
    position
      .y
      .clamp(work_area.position.y, max_y.max(work_area.position.y)),
  )
}

#[tauri::command]
pub fn finish_recording_dock_drag(app: AppHandle) -> Result<(), String> {
  let to_message = |error: tauri::Error| error.to_string();
  let dock = app
    .get_webview_window(WindowLabel::RecordingDock.as_str())
    .ok_or_else(|| "The recording pill is unavailable".to_owned())?;
  // Clamped so a drag cannot park the pill under the menu bar, the notch or
  // the taskbar. The clamped position is kept as computed: on macOS the move
  // is applied asynchronously, so the frame would read back unclamped.
  let dragged = dock.outer_position().map_err(to_message)?;
  let position = match monitor_with_most_overlap(&app, &dock).map_err(to_message)? {
    Some(monitor) => contained_position(&monitor, dragged, dock.outer_size().map_err(to_message)?),
    None => dragged,
  };
  if position != dragged {
    dock.set_position(position).map_err(to_message)?;
  }
  let placed = *RECORDING_DOCK_PLACED
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  // Pointer-up also fires for a plain click on the pill's buttons. Persisting
  // then would turn "never dragged" into a saved offset, and the pill would
  // stop using the default placement on whichever monitor the bar is on.
  if placed == Some(position) {
    return Ok(());
  }

  *RECORDING_DOCK_PLACED
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(position);
  if let Some(offset) = recording_dock_offset(&app, &dock).map_err(to_message)? {
    store_recording_dock_offset(&app, offset).map_err(to_message)?;
  }

  Ok(())
}

/// Persists a native drag after Windows stops delivering pointer events.
#[cfg(target_os = "windows")]
pub fn manage_recording_dock_movement(app: &AppHandle) {
  let Some(window) = app.get_webview_window(WindowLabel::RecordingDock.as_str()) else {
    return;
  };
  let app = app.clone();

  window.on_window_event(move |event| {
    if !matches!(event, WindowEvent::Moved(_)) {
      return;
    }

    watch_for_recording_dock_mouse_up(app.clone());
  });
}

#[cfg(not(target_os = "windows"))]
pub fn manage_recording_dock_movement(_app: &AppHandle) {}
#[cfg(target_os = "windows")]
fn watch_for_recording_dock_mouse_up(app: AppHandle) {
  use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};

  if unsafe { GetAsyncKeyState(VK_LBUTTON.0.into()) } >= 0
    || DOCK_DRAG_ACTIVE.swap(true, Ordering::Relaxed)
  {
    return;
  }

  tauri::async_runtime::spawn_blocking(move || {
    loop {
      let is_pressed = unsafe { GetAsyncKeyState(VK_LBUTTON.0.into()) } < 0;
      if !is_pressed {
        break;
      }
      std::thread::sleep(Duration::from_millis(8));
    }

    let _ = finish_recording_dock_drag(app);
    DOCK_DRAG_ACTIVE.store(false, Ordering::Relaxed);
  });
}

pub fn hide_recording_dock(app: &AppHandle) -> tauri::Result<()> {
  first_reveal::cancel();
  if let Some(dock) = app.get_webview_window(WindowLabel::RecordingDock.as_str()) {
    platform::hide(&dock)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests;
