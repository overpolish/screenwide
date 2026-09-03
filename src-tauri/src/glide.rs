// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::windows::WindowLabel;
use tauri::AppHandle;
#[cfg(target_os = "windows")]
use tauri::{LogicalSize, Manager, PhysicalPosition, PhysicalSize};

#[cfg(target_os = "macos")]
#[path = "glide/macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "glide/windows.rs"]
mod platform;

mod core;
#[path = "glide/events.rs"]
mod events;
#[path = "glide/settings.rs"]
pub mod settings;

use events::{emit, GlideInputEvent};

#[path = "glide/region_rect.rs"]
mod region_rect;

#[path = "glide/icon.rs"]
mod icon;

#[path = "glide/fit.rs"]
mod fit;

/// How long the hand rests before the detector commits a transition. One
/// timing on every platform: short enough that a multi-fold reads as one
/// gesture, long enough that a flick's overshoot never chains into the next
/// fold.
const REST_MS: f64 = 40.0;

const GLIDABLE_WINDOW_LABELS: &[WindowLabel] = &[
  WindowLabel::Settings,
  WindowLabel::RecordingBar,
  WindowLabel::RecordingDock,
  WindowLabel::ExportRecording,
  WindowLabel::ExportScreenshot,
];

const fn uses_full_surface(label: WindowLabel) -> bool {
  matches!(
    label,
    WindowLabel::RecordingBar | WindowLabel::RecordingDock
  )
}

/// Glide's preview has the same logical dimensions on both platforms. Windows
/// reapplies them when a session begins because moving a hidden WebView between
/// differently-scaled monitors can otherwise retain its previous physical
/// extent.
#[cfg(target_os = "windows")]
const PREVIEW_WIDTH: f64 = 48.0;
#[cfg(target_os = "windows")]
const PREVIEW_HEIGHT: f64 = 32.0;

pub fn initialize(app: &AppHandle) -> Result<(), String> {
  crate::windows::initialize_glide_preview(app).map_err(|error| error.to_string())?;
  platform::start(app.clone())
}

#[cfg(target_os = "macos")]
fn begin_logical(app: &AppHandle, session_id: u64, x: f64, y: f64) -> Result<(), String> {
  let (result_tx, result_rx) = std::sync::mpsc::sync_channel(1);
  let main_app = app.clone();
  app
    .run_on_main_thread(move || {
      let _ = result_tx.send(begin_logical_on_main(&main_app, session_id, x, y));
    })
    .map_err(|error| error.to_string())?;
  result_rx
    .recv()
    .map_err(|_| "The Glide preview main-thread operation was interrupted".to_owned())?
}

#[cfg(target_os = "macos")]
fn begin_logical_on_main(app: &AppHandle, session_id: u64, x: f64, y: f64) -> Result<(), String> {
  crate::windows::position_glide_preview(app, x, y).map_err(|error| error.to_string())?;
  emit(app, GlideInputEvent::Start { session_id })
}

#[cfg(target_os = "windows")]
fn begin_physical(app: &AppHandle, session_id: u64, x: i32, y: i32) -> Result<(), String> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or_else(|| "The Glide preview window is unavailable".to_owned())?;

  // First move the hidden window onto the anchor's monitor so Windows/Tauri
  // resolves the logical size against that monitor's DPI, then centre the
  // resulting physical extent and contain it in the monitor work area.
  window
    .set_position(PhysicalPosition::new(x, y))
    .map_err(|error| error.to_string())?;
  window
    .set_size(LogicalSize::new(PREVIEW_WIDTH, PREVIEW_HEIGHT))
    .map_err(|error| error.to_string())?;
  let size = window.outer_size().map_err(|error| error.to_string())?;
  let origin = preview_origin(app, PhysicalPosition::new(x, y), size)?;
  window
    .set_position(origin)
    .map_err(|error| error.to_string())?;
  emit(app, GlideInputEvent::Start { session_id })
}

#[cfg(target_os = "windows")]
fn preview_origin(
  app: &AppHandle,
  anchor: PhysicalPosition<i32>,
  size: PhysicalSize<u32>,
) -> Result<PhysicalPosition<i32>, String> {
  let monitors = app
    .available_monitors()
    .map_err(|error| error.to_string())?;
  let monitor = monitors
    .into_iter()
    .find(|monitor| {
      let position = monitor.position();
      let extent = monitor.size();
      anchor.x >= position.x
        && anchor.x < position.x + extent.width as i32
        && anchor.y >= position.y
        && anchor.y < position.y + extent.height as i32
    })
    .or_else(|| app.primary_monitor().ok().flatten());
  let centered = PhysicalPosition::new(
    anchor.x - (size.width / 2) as i32,
    anchor.y - (size.height / 2) as i32,
  );
  let Some(monitor) = monitor else {
    return Ok(centered);
  };
  let work = monitor.work_area();
  Ok(contained_origin(centered, size, work.position, work.size))
}

#[cfg(target_os = "windows")]
fn contained_origin(
  origin: PhysicalPosition<i32>,
  size: PhysicalSize<u32>,
  work_origin: PhysicalPosition<i32>,
  work_size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
  let maximum_x = work_origin.x + work_size.width.saturating_sub(size.width) as i32;
  let maximum_y = work_origin.y + work_size.height.saturating_sub(size.height) as i32;
  PhysicalPosition::new(
    origin.x.clamp(work_origin.x, maximum_x.max(work_origin.x)),
    origin.y.clamp(work_origin.y, maximum_y.max(work_origin.y)),
  )
}

fn finish(app: &AppHandle, anchor_x: f64, anchor_y: f64, cancelled: bool) {
  let _ = emit(
    app,
    GlideInputEvent::End {
      anchor_x,
      anchor_y,
      cancelled,
    },
  );
  #[cfg(target_os = "macos")]
  {
    let main_app = app.clone();
    let _ = app.run_on_main_thread(move || {
      let _ = crate::windows::hide_glide_preview(&main_app);
    });
  }
  #[cfg(target_os = "windows")]
  crate::windows::defer_hide_glide_preview(app);
}

/// The cursor returns this far into the fade, so its arrival overlaps the
/// preview's last visible frames instead of trailing an already-empty screen.
#[cfg(target_os = "macos")]
const CURSOR_RESTORE_DELAY: std::time::Duration = std::time::Duration::from_millis(90);

/// Ends a committed session with a fade rather than an instant hide, running
/// `on_restore` a beat into the fade (the cursor's cue) and `on_faded` once the
/// preview is gone. Every failure here still runs both - immediately, and
/// exactly once each - rather than leaving the cursor hidden behind a stuck
/// panel.
#[cfg(target_os = "macos")]
fn finish_with_fade(
  app: &AppHandle,
  anchor_x: f64,
  anchor_y: f64,
  on_restore: Box<dyn FnOnce() + Send>,
  on_faded: Box<dyn FnOnce() + Send>,
) {
  let _ = emit(
    app,
    GlideInputEvent::End {
      anchor_x,
      anchor_y,
      cancelled: false,
    },
  );
  // The scheduled paths and the fallback all hold the completions; whichever
  // gets there first takes one, so each runs exactly once however this ends.
  let on_restore = std::sync::Arc::new(std::sync::Mutex::new(Some(on_restore)));
  let on_faded = std::sync::Arc::new(std::sync::Mutex::new(Some(on_faded)));

  let restore = on_restore.clone();
  let _ = std::thread::Builder::new()
    .name("glide-cursor-restore".to_owned())
    .spawn(move || {
      std::thread::sleep(CURSOR_RESTORE_DELAY);
      // Runs right here: the release is thread-agnostic, and the landing's
      // window read is Accessibility traffic that has no place on the main
      // thread. (The fade completion's ordering fallback still runs it on the
      // main thread in the rare race, bounded by the target's short timeout.)
      run_once(&restore);
    });

  let faded = on_faded.clone();
  let restore_first = on_restore.clone();
  let completion = Box::new(move || {
    // The fade's completion and the restore timer reach the main thread through
    // different mechanisms, so their order is not guaranteed. The restore must
    // land before `on_faded` unblocks new sessions - otherwise a gesture can
    // begin while the cursor is still pinned, reading the old anchor as its
    // location. Running it here first (a no-op if the timer won) pins the
    // order.
    run_once(&restore_first);
    run_once(&faded);
  });
  if let Err(error) = crate::windows::fade_glide_preview(app, completion) {
    eprintln!("Could not fade Glide out: {error}");
    let main_app = app.clone();
    let _ = app.run_on_main_thread(move || {
      let _ = crate::windows::hide_glide_preview(&main_app);
    });
    run_once(&on_restore);
    run_once(&on_faded);
  }
}

/// Runs a shared one-shot completion, if it has not been run already.
#[cfg(target_os = "macos")]
fn run_once(completion: &std::sync::Mutex<Option<Box<dyn FnOnce() + Send>>>) {
  let taken = completion.lock().ok().and_then(|mut slot| slot.take());
  if let Some(completion) = taken {
    completion();
  }
}

#[cfg(test)]
#[path = "glide/input_event_tests.rs"]
mod input_event_tests;

#[cfg(all(test, target_os = "windows"))]
mod windows_preview_tests {
  use super::*;

  #[test]
  fn preview_is_contained_at_each_work_area_edge() {
    let work_origin = PhysicalPosition::new(-1_920, 40);
    let work_size = PhysicalSize::new(1_920, 1_040);
    let preview_size = PhysicalSize::new(72, 48);

    assert_eq!(
      contained_origin(
        PhysicalPosition::new(-1_956, 16),
        preview_size,
        work_origin,
        work_size,
      ),
      PhysicalPosition::new(-1_920, 40),
    );
    assert_eq!(
      contained_origin(
        PhysicalPosition::new(-20, 1_060),
        preview_size,
        work_origin,
        work_size,
      ),
      PhysicalPosition::new(-72, 1_032),
    );
  }

  #[test]
  fn preview_larger_than_work_area_pins_to_its_origin() {
    assert_eq!(
      contained_origin(
        PhysicalPosition::new(40, 50),
        PhysicalSize::new(400, 300),
        PhysicalPosition::new(100, 80),
        PhysicalSize::new(200, 100),
      ),
      PhysicalPosition::new(100, 80),
    );
  }
}
