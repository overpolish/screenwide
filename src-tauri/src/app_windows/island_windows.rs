// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island's window on Windows: at the top of the work area of its
//! monitor, with no notch to sit under, and letting clicks through
//! everywhere but the island. Windows tells a window nothing of the pointer
//! moving over other apps without a system-wide hook, so while the island is
//! up the pointer's position is read at a steady rate, a cheap call, and
//! click-through changes only when the pointer crosses the island's edge.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use super::island::{self, WINDOW_SIZE};
use super::{platform, WindowLabel};

/// How often the pointer is looked for while the island is up.
const POLL_INTERVAL: Duration = Duration::from_millis(16);

/// Which showing of the island the pointer is being looked for in; none
/// when 0. A thread looks only while its own showing lasts, so hiding and
/// showing again quickly never leaves two looking.
static WATCHING: AtomicU64 = AtomicU64::new(0);
static NEXT_WATCH: AtomicU64 = AtomicU64::new(1);
/// The island is tucking away, and takes no clicks.
static HIDING: AtomicBool = AtomicBool::new(false);
/// The pointer is on the island, so its window takes clicks.
static OVER: AtomicBool = AtomicBool::new(false);
/// The window's top left on the desktop, in pixels, and the scale of the
/// monitor it was placed on.
static PLACED: Mutex<Option<(PhysicalPosition<i32>, f64)>> = Mutex::new(None);

fn window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  app
    .get_webview_window(WindowLabel::Island.as_str())
    .ok_or(tauri::Error::WindowNotFound)
}

fn pointer() -> Option<(f64, f64)> {
  let mut point = ::windows::Win32::Foundation::POINT::default();
  unsafe { ::windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.ok()?;
  Some((f64::from(point.x), f64::from(point.y)))
}

/// Whether the pointer is on one of the island's parts, which are measured
/// in points from the window's top left.
fn pointer_on_island() -> bool {
  let Some((origin, scale)) = *PLACED.lock().unwrap_or_else(PoisonError::into_inner) else {
    return false;
  };
  let Some((x, y)) = pointer() else {
    return false;
  };
  island::is_on_island(
    &island::rects(),
    (x - f64::from(origin.x)) / scale,
    (y - f64::from(origin.y)) / scale,
  )
}

fn set_passthrough(app: &AppHandle, passthrough: bool) {
  let target = app.clone();
  let _ = app.run_on_main_thread(move || {
    if let Ok(window) = window(&target) {
      let _ = platform::set_pointer_passthrough(&window, passthrough);
    }
  });
}

fn update(app: &AppHandle) {
  let over = !HIDING.load(Ordering::Relaxed) && pointer_on_island();
  if OVER.swap(over, Ordering::Relaxed) != over {
    set_passthrough(app, !over);
  }
}

fn watch(app: AppHandle) {
  let showing = NEXT_WATCH.fetch_add(1, Ordering::Relaxed);
  WATCHING.store(showing, Ordering::Relaxed);
  std::thread::spawn(move || {
    while WATCHING.load(Ordering::Relaxed) == showing {
      std::thread::sleep(POLL_INTERVAL);
      update(&app);
    }
  });
}

/// The monitor the island sits on, with its display id: the display set for
/// it while connected, otherwise the primary monitor.
fn island_monitor(app: &AppHandle) -> tauri::Result<Option<(Option<u32>, tauri::Monitor)>> {
  let displays = crate::monitor_topology::snapshot(app).unwrap_or_default();
  if let Some(preferred) = island::preferred_display(app) {
    if let Some(display) = displays.iter().find(|display| display.id == preferred) {
      return Ok(Some((Some(display.id), display.native.clone())));
    }
  }
  let Some(primary) = app.primary_monitor()? else {
    return Ok(None);
  };
  let id = displays
    .iter()
    .find(|display| display.native.position() == primary.position())
    .map(|display| display.id);
  Ok(Some((id, primary)))
}

/// The display the island belongs on now.
pub(super) fn target_display(app: &AppHandle) -> Option<u32> {
  island_monitor(app).ok().flatten().and_then(|(id, _)| id)
}

pub(super) fn show(app: &AppHandle) -> tauri::Result<()> {
  let window = window(app)?;
  let Some((display, monitor)) = island_monitor(app)? else {
    return Err(tauri::Error::WindowNotFound);
  };
  island::set_placed_display(display);
  let scale = monitor.scale_factor();
  let area = monitor.work_area();
  let size = PhysicalSize::new(
    (WINDOW_SIZE.0 * scale).round() as u32,
    (WINDOW_SIZE.1 * scale).round() as u32,
  );
  let position = PhysicalPosition::new(
    area.position.x + (area.size.width as i32 - size.width as i32) / 2,
    area.position.y,
  );
  // Moved first, so a change of scale on the way does not resize it after.
  window.set_position(position)?;
  window.set_size(size)?;
  *PLACED.lock().unwrap_or_else(PoisonError::into_inner) = Some((position, scale));

  HIDING.store(false, Ordering::Relaxed);
  OVER.store(false, Ordering::Relaxed);
  platform::set_pointer_passthrough(&window, true)?;
  island::announce_shown(app, None);
  platform::show(&window, 1.0)?;
  platform::restore_recording_level(&window)?;
  watch(app.clone());
  Ok(())
}

/// Tucking away, the island takes no clicks, so a drag that crosses it as it
/// goes is not caught by it.
pub(super) fn set_hiding(app: &AppHandle, is_hiding: bool) {
  HIDING.store(is_hiding, Ordering::Relaxed);
  update(app);
}

/// The next look at the pointer finds the island's new shape.
pub(super) fn refresh_hit_area(app: &AppHandle) {
  update(app);
}

pub(super) fn hide(app: &AppHandle) -> tauri::Result<()> {
  WATCHING.store(0, Ordering::Relaxed);
  OVER.store(false, Ordering::Relaxed);
  platform::hide(&window(app)?)
}
