// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The cue's window going from one picture's shape to the next. Set at once,
//! it would jump to the new shape and only then cross-fade, the old picture
//! cut to the new frame; instead it eases there over the cross-fade's time,
//! so the frame and the pictures change together.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

use super::placement::Placement;

/// As long as the pictures' cross-fade (`motionDurations.rearrange`).
const DURATION: Duration = Duration::from_millis(400);
/// Between steps, about a display's refresh.
const STEP: Duration = Duration::from_millis(16);

/// Counts the changes, so a newer one, or the window placed at once, ends the
/// one before.
static CHANGES: AtomicU64 = AtomicU64::new(0);
/// Set while the window changes shape, so the sizes it passes through are
/// not taken for a resize by hand.
static CHANGING: AtomicBool = AtomicBool::new(false);

pub(super) fn is_changing() -> bool {
  CHANGING.load(Ordering::Relaxed)
}

/// Ends a change under way, for the window to be placed at once.
pub(super) fn stop() {
  CHANGES.fetch_add(1, Ordering::Relaxed);
  CHANGING.store(false, Ordering::Relaxed);
}

/// Takes `window` from where it stands to `to`, easing out, off the calling
/// thread.
pub(super) fn change(window: &WebviewWindow, to: Placement) -> Result<(), String> {
  let fail = |error: tauri::Error| error.to_string();
  let from = (
    window.outer_position().map_err(fail)?,
    window.outer_size().map_err(fail)?,
  );
  let change = CHANGES.fetch_add(1, Ordering::Relaxed) + 1;
  if from == to {
    CHANGING.store(false, Ordering::Relaxed);
    return Ok(());
  }
  CHANGING.store(true, Ordering::Relaxed);
  let window = window.clone();
  std::thread::spawn(move || {
    let start = Instant::now();
    loop {
      if CHANGES.load(Ordering::Relaxed) != change {
        return;
      }
      let done = (start.elapsed().as_secs_f64() / DURATION.as_secs_f64()).min(1.0);
      let eased = 1.0 - (1.0 - done).powi(3);
      if let Err(error) = set_frame(&window, between(from, to, eased)) {
        eprintln!("Could not reshape the cue: {error}");
        break;
      }
      if done >= 1.0 {
        break;
      }
      std::thread::sleep(STEP);
    }
    if CHANGES.load(Ordering::Relaxed) == change {
      CHANGING.store(false, Ordering::Relaxed);
    }
  });
  Ok(())
}

/// The frame `share` of the way from `from` to `to`.
fn between(from: Placement, to: Placement, share: f64) -> Placement {
  let along = |a: f64, b: f64| a + (b - a) * share;
  (
    PhysicalPosition::new(
      along(f64::from(from.0.x), f64::from(to.0.x)).round() as i32,
      along(f64::from(from.0.y), f64::from(to.0.y)).round() as i32,
    ),
    PhysicalSize::new(
      along(f64::from(from.1.width), f64::from(to.1.width)).round() as u32,
      along(f64::from(from.1.height), f64::from(to.1.height)).round() as u32,
    ),
  )
}

/// Moves and sizes the window in one step: two, a size then a place, show a
/// frame with one edge where it was and the other where it will be.
#[cfg(target_os = "macos")]
fn set_frame(window: &WebviewWindow, (position, size): Placement) -> Result<(), String> {
  use core_graphics::display::CGDisplay;
  use objc2_app_kit::NSWindow;
  use objc2_foundation::{NSPoint, NSRect, NSSize};

  let scale = window.scale_factor().map_err(|error| error.to_string())?;
  // AppKit measures the desktop in points, up from the main display's foot.
  let main_height = CGDisplay::main().pixels_high() as f64;
  let (width, height) = (
    f64::from(size.width) / scale,
    f64::from(size.height) / scale,
  );
  let frame = NSRect::new(
    NSPoint::new(
      f64::from(position.x) / scale,
      main_height - f64::from(position.y) / scale - height,
    ),
    NSSize::new(width, height),
  );
  let target = window.clone();
  window
    .run_on_main_thread(move || {
      // Asked for here, on the main thread, where a window closed meanwhile
      // answers with an error rather than a stale pointer.
      if let Ok(raw) = target.ns_window() {
        let native: &NSWindow = unsafe { &*raw.cast() };
        native.setFrame_display(frame, true);
      }
    })
    .map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn set_frame(window: &WebviewWindow, (position, size): Placement) -> Result<(), String> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

  let hwnd = HWND(window.hwnd().map_err(|error| error.to_string())?.0);
  unsafe {
    SetWindowPos(
      hwnd,
      None,
      position.x,
      position.y,
      size.width as i32,
      size.height as i32,
      SWP_NOACTIVATE | SWP_NOZORDER,
    )
  }
  .map_err(|error| error.to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn set_frame(window: &WebviewWindow, (position, size): Placement) -> Result<(), String> {
  window.set_size(size).map_err(|error| error.to_string())?;
  window
    .set_position(position)
    .map_err(|error| error.to_string())
}
