// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island's window: a transparent panel at the top of its display, the
//! one set for it or else the primary display, in which the island draws
//! itself. The window is larger than
//! the island will ever be, so it never resizes while the island changes
//! shape inside it, and clicks pass through it everywhere but on the island,
//! which reports where it is (`set_island_hit_area`).
//!
//! It is one window whatever the island shows, as there is one notch: the
//! features it shows decide what that is, and this module only brings it out
//! and puts it away. Hiding is the island's to finish: it is asked to tuck
//! itself away (`island://hide`), and the window goes once it has.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use super::WindowLabel;

#[cfg(target_os = "macos")]
use super::island_macos as native;
#[cfg(target_os = "windows")]
use super::island_windows as native;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod native {
  use tauri::AppHandle;

  pub(super) fn show(app: &AppHandle) -> tauri::Result<()> {
    super::announce_shown(app, None);
    Ok(())
  }

  pub(super) fn target_display(_app: &AppHandle) -> Option<u32> {
    None
  }

  pub(super) fn set_hiding(_app: &AppHandle, _is_hiding: bool) {}

  pub(super) fn refresh_hit_area(_app: &AppHandle) {}

  pub(super) fn apply_level(_app: &AppHandle) -> tauri::Result<()> {
    Ok(())
  }

  pub(super) fn hide(_app: &AppHandle) -> tauri::Result<()> {
    Ok(())
  }
}

/// Room for the island's widest part, its sideways shift round a notch and
/// its shadow, in points: the window clips whatever falls outside it.
pub(super) const WINDOW_SIZE: (f64, f64) = (900.0, 640.0);

/// How far round the island the pointer still counts as on it, in points.
/// The pointer leaving the island is then seen by the island itself, inside
/// its window, before clicks start passing through, so what hovering opened
/// closes as it should.
const MARGIN: f64 = 8.0;

/// How long the island is given to tuck away before its window is hidden
/// regardless: its motion takes about half this.
const HIDE_FALLBACK: Duration = Duration::from_secs(1);

/// A part of the island, in its window, in CSS pixels from the top left.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct IslandRect {
  pub height: f64,
  pub width: f64,
  pub x: f64,
  pub y: f64,
}

/// The notch of the screen the island is on, as the screen reports it.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct IslandNotch {
  pub height: f64,
  pub width: f64,
}

/// Whether the island is out, and the notch it is under, for a page that
/// loads after it was shown and so missed `island://shown`.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IslandPresence {
  pub is_shown: bool,
  pub notch: Option<IslandNotch>,
}

/// The window is up, out or tucking away.
static SHOWN: AtomicBool = AtomicBool::new(false);
/// Which hide is waiting for the island to tuck away, by number; none when 0.
/// Showing the island again clears it, so a late finish hides nothing.
static PENDING_HIDE: AtomicU64 = AtomicU64::new(0);
static NEXT_HIDE: AtomicU64 = AtomicU64::new(1);
/// The notch of the screen the island was last shown on.
static NOTCH: Mutex<Option<IslandNotch>> = Mutex::new(None);
/// The display the island was last placed on, by id, so a change of
/// displays knows whether it has to move.
static DISPLAY: Mutex<Option<u32>> = Mutex::new(None);
/// The island's parts as last reported, kept while it is hidden so it comes
/// back knowing where they are.
static RECTS: Mutex<Vec<IslandRect>> = Mutex::new(Vec::new());
/// The island stands above the capture overlays, for as long as Annotate
/// draws: its tools must never be covered by a stroke.
static ABOVE_OVERLAYS: AtomicBool = AtomicBool::new(false);

pub(crate) fn window(app: &AppHandle) -> Option<WebviewWindow> {
  app.get_webview_window(WindowLabel::Island.as_str())
}

/// Raises the island above the capture overlays, or puts it back at its own
/// level. It keeps the choice while hidden and is placed by it when shown.
pub fn set_above_overlays(app: &AppHandle, above: bool) -> tauri::Result<()> {
  ABOVE_OVERLAYS.store(above, Ordering::Relaxed);
  if SHOWN.load(Ordering::Relaxed) {
    native::apply_level(app)?;
  }
  Ok(())
}

#[cfg(target_os = "macos")]
pub(super) fn is_above_overlays() -> bool {
  ABOVE_OVERLAYS.load(Ordering::Relaxed)
}

/// Whether the island is out or coming out: up, and not tucking away.
pub fn is_shown() -> bool {
  SHOWN.load(Ordering::Relaxed) && PENDING_HIDE.load(Ordering::Relaxed) == 0
}

/// Brings the island out on its display. Out already, it stays where it is;
/// tucking away, it comes back out where it was.
pub fn show(app: &AppHandle) -> tauri::Result<()> {
  let was_hiding = PENDING_HIDE.swap(0, Ordering::Relaxed) != 0;
  if SHOWN.load(Ordering::Relaxed) {
    if was_hiding {
      native::set_hiding(app, false);
      announce_shown(app, notch());
    }
    return Ok(());
  }
  SHOWN.store(true, Ordering::Relaxed);
  native::show(app)
}

/// Asks the island to tuck itself away, and hides its window once it says
/// it has (`finish_hiding_island`), or after `HIDE_FALLBACK` if it never
/// does. Tucking away, it lets every click through.
pub fn hide(app: &AppHandle) -> tauri::Result<()> {
  if !is_shown() {
    return Ok(());
  }
  let request = NEXT_HIDE.fetch_add(1, Ordering::Relaxed);
  PENDING_HIDE.store(request, Ordering::Relaxed);
  native::set_hiding(app, true);
  app.emit_to(WindowLabel::Island.as_str(), "island://hide", ())?;
  let app = app.clone();
  std::thread::spawn(move || {
    std::thread::sleep(HIDE_FALLBACK);
    if PENDING_HIDE
      .compare_exchange(request, 0, Ordering::Relaxed, Ordering::Relaxed)
      .is_ok()
    {
      let _ = take_down(&app);
    }
  });
  Ok(())
}

/// Puts the island away at once, for what must not find it on screen: a
/// capture that follows straight after.
pub fn hide_at_once(app: &AppHandle) -> tauri::Result<()> {
  if !SHOWN.load(Ordering::Relaxed) {
    return Ok(());
  }
  PENDING_HIDE.store(0, Ordering::Relaxed);
  // Tucked away unseen, so it comes out of the notch when next shown.
  app.emit_to(WindowLabel::Island.as_str(), "island://hide", ())?;
  take_down(app)
}

fn take_down(app: &AppHandle) -> tauri::Result<()> {
  SHOWN.store(false, Ordering::Relaxed);
  native::hide(app)
}

/// The displays changed. An island that is out and now belongs on another
/// display tucks away and comes out again there, rather than sliding across
/// in view; one that stays is placed again, as its display may have changed
/// size.
pub(super) fn follow_displays(app: &AppHandle) -> tauri::Result<()> {
  if !is_shown() {
    return Ok(());
  }
  let placed = *DISPLAY.lock().unwrap_or_else(PoisonError::into_inner);
  if native::target_display(app) != placed {
    hide_at_once(app)?;
    return show(app);
  }
  native::show(app)
}

/// The display set for the island, if any: it sits on the primary display
/// otherwise, and does not follow the pointer, so drawing a region on
/// another display does not take it away from where the other modes leave
/// it.
pub(super) fn preferred_display(app: &AppHandle) -> Option<u32> {
  crate::settings::current(app).recording_controls_display
}

/// Records the display the island has been placed on.
pub(super) fn set_placed_display(display: Option<u32>) {
  *DISPLAY.lock().unwrap_or_else(PoisonError::into_inner) = display;
}

/// Stores the notch the island is under and tells the island it is out.
pub(super) fn announce_shown(app: &AppHandle, notch: Option<IslandNotch>) {
  *NOTCH.lock().unwrap_or_else(PoisonError::into_inner) = notch;
  let _ = app.emit_to(WindowLabel::Island.as_str(), "island://shown", notch);
}

fn notch() -> Option<IslandNotch> {
  *NOTCH.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The island's parts as last reported.
pub(super) fn rects() -> Vec<IslandRect> {
  RECTS.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// Whether a point in the island's window, in points from its top left, is
/// on one of the island's parts.
pub(super) fn is_on_island(rects: &[IslandRect], x: f64, y: f64) -> bool {
  rects.iter().any(|rect| {
    x >= rect.x - MARGIN
      && x <= rect.x + rect.width + MARGIN
      && y >= rect.y - MARGIN
      && y <= rect.y + rect.height + MARGIN
  })
}

/// The island has tucked itself away after being asked to hide, so its
/// window can go, unless it has been shown again meanwhile.
#[tauri::command]
pub fn finish_hiding_island(app: AppHandle) -> tauri::Result<()> {
  if PENDING_HIDE.swap(0, Ordering::Relaxed) == 0 {
    return Ok(());
  }
  take_down(&app)
}

#[tauri::command]
pub fn island_presence() -> IslandPresence {
  IslandPresence {
    is_shown: is_shown(),
    notch: notch(),
  }
}

/// Where the island's parts are, so clicks on them reach it and clicks
/// anywhere else in its window pass through to what is underneath.
#[tauri::command]
pub fn set_island_hit_area(app: AppHandle, rects: Vec<IslandRect>) {
  rects.clone_into(&mut RECTS.lock().unwrap_or_else(PoisonError::into_inner));
  // The island changing shape can bring its edge to the pointer, or take it
  // away, without the pointer moving.
  native::refresh_hit_area(&app);
}
