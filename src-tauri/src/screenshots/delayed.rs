// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Delayed Screenshot: a still taken a few seconds after it is asked for, so
//! the user has time to bring up a tooltip, menu or hover state first.
//!
//! The countdown runs here rather than in the recording bar, which hides as it
//! begins, and the tray icon shows the seconds left.

use std::sync::Mutex;
use std::time::Duration;

use tauri::AppHandle;

use super::still_command::take_still;
use super::{ScreenshotDestination, ScreenshotTarget};
use crate::capture_overlays::CaptureOverlay;

const TICK: Duration = Duration::from_secs(1);
/// Time for the tray to put its ordinary icon back before the shutter, so a
/// shot that takes in the menu bar or taskbar does not show the countdown.
const TRAY_SETTLE: Duration = Duration::from_millis(150);

/// What the shot is of.
#[derive(Clone, Copy, Debug)]
pub(crate) enum DelayedTarget {
  /// The source chosen in the recording bar.
  Source(ScreenshotTarget),
  /// The tray has no source chosen, so it takes the whole display the pointer
  /// is on when the countdown ends.
  DisplayUnderPointer,
}

/// Which countdown is current and how many seconds it has left, zero when none
/// is running. Held together so a superseded countdown can never write over a
/// newer one, or over a cancellation.
struct Countdown {
  generation: u64,
  remaining: u8,
}

static COUNTDOWN: Mutex<Countdown> = Mutex::new(Countdown {
  generation: 0,
  remaining: 0,
});

fn countdown() -> std::sync::MutexGuard<'static, Countdown> {
  COUNTDOWN
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The seconds left before the shutter, while a countdown is running.
pub(crate) fn remaining() -> Option<u8> {
  Some(countdown().remaining).filter(|seconds| *seconds > 0)
}

/// Begins a countdown, replacing any already running.
pub(crate) fn start(app: &AppHandle, target: DelayedTarget) -> Result<(), String> {
  if !crate::recording::is_idle(app) {
    return Err("A screenshot cannot be taken while a recording is active".to_owned());
  }
  // The live annotations stay, to arrive in the editor as a layer of their
  // own. Every other capture tool owns the pointer, which would leave nothing
  // to hover.
  crate::capture_overlays::dismiss_except(app, &[CaptureOverlay::Annotate]);
  let seconds = crate::settings::current(app).screenshot_delay_seconds;
  let generation = {
    let mut state = countdown();
    state.generation += 1;
    state.remaining = seconds;
    state.generation
  };
  crate::tray::refresh(app);
  tauri::async_runtime::spawn(run(app.clone(), generation, seconds, target));
  Ok(())
}

/// Stops the running countdown, if there is one, without taking the shot.
pub(crate) fn cancel(app: &AppHandle) {
  {
    let mut state = countdown();
    if state.remaining == 0 {
      return;
    }
    state.generation += 1;
    state.remaining = 0;
  }
  crate::tray::refresh(app);
}

/// Sets the seconds left for `generation`, reporting whether it is still the
/// current countdown.
fn set_remaining(generation: u64, seconds: u8) -> bool {
  let mut state = countdown();
  if state.generation != generation {
    return false;
  }
  state.remaining = seconds;
  true
}

async fn run(app: AppHandle, generation: u64, seconds: u8, target: DelayedTarget) {
  for left in (1..=seconds).rev() {
    // A recording started meanwhile takes the tray and the screen; the shot
    // it would interrupt is dropped rather than taken afterwards.
    if !crate::recording::is_idle(&app) {
      cancel(&app);
      return;
    }
    if !set_remaining(generation, left) {
      return;
    }
    crate::tray::refresh_icon(&app);
    tokio::time::sleep(TICK).await;
  }
  if !set_remaining(generation, 0) {
    return;
  }
  crate::tray::refresh(&app);
  tokio::time::sleep(TRAY_SETTLE).await;

  let target = match target {
    DelayedTarget::Source(target) => target,
    DelayedTarget::DisplayUnderPointer => match display_under_pointer() {
      Ok(target) => target,
      Err(error) => {
        eprintln!("Could not find the display for the delayed screenshot: {error}");
        return;
      }
    },
  };
  // The pointer is part of what a screenshot is showing, as it is for every
  // other still.
  if let Err(error) = take_still(app, target, true, ScreenshotDestination::Editor).await {
    eprintln!("Could not take the delayed screenshot: {error}");
  }
}

/// The display the pointer is on, or the primary display when that cannot be
/// read.
fn display_under_pointer() -> Result<ScreenshotTarget, String> {
  let monitor = match pointer_position().and_then(|(x, y)| xcap::Monitor::from_point(x, y).ok()) {
    Some(monitor) => monitor,
    None => xcap::Monitor::all()
      .map_err(|error| error.to_string())?
      .into_iter()
      .find(|monitor| monitor.is_primary().unwrap_or(false))
      .ok_or_else(|| "No display is available".to_owned())?,
  };
  let monitor_id = monitor.id().map_err(|error| error.to_string())?;
  Ok(ScreenshotTarget::Screen { monitor_id })
}

/// The pointer in the desktop coordinates xcap's monitors use: global points
/// on macOS, physical pixels on Windows.
#[cfg(target_os = "macos")]
fn pointer_position() -> Option<(i32, i32)> {
  use core_graphics::{
    event::CGEvent,
    event_source::{CGEventSource, CGEventSourceStateID},
  };
  let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
  let point = CGEvent::new(source).ok()?.location();
  Some((point.x.floor() as i32, point.y.floor() as i32))
}

#[cfg(target_os = "windows")]
fn pointer_position() -> Option<(i32, i32)> {
  let mut point = windows::Win32::Foundation::POINT::default();
  unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.ok()?;
  Some((point.x, point.y))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn pointer_position() -> Option<(i32, i32)> {
  None
}

/// The recording bar's way in: the bar steps aside for the countdown so that
/// it neither covers what the user hovers nor appears in the shot.
#[tauri::command]
pub fn start_delayed_screenshot(app: AppHandle, target: ScreenshotTarget) -> Result<(), String> {
  start(&app, DelayedTarget::Source(target))?;
  crate::app_windows::hide_recording_ui(app).map_err(|error| error.to_string())
}
