// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A panel window larger than what it shows, as the editor's island is: its
//! page reports where its parts are, and clicks pass through the window
//! everywhere else. The editor's island window covers the editor's content
//! area and never resizes as the island changes shape inside it, so clicks,
//! hovers and scrolls on the picture, strip and columns around the island
//! must reach the editor underneath.
//!
//! The pointer's position is read on a timer while any such panel is up, as
//! the island's window does: a window that lets clicks through hears nothing
//! of the pointer, so it cannot tell for itself when the pointer arrives.

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::super::island::{is_on_island, IslandRect};

/// How often the pointer is looked for while it is on a panel's parts,
/// quickly enough to let clicks through as soon as it leaves them.
const POLL_ON_PARTS: Duration = Duration::from_millis(16);
/// How often it is looked for elsewhere: well inside the time between
/// reaching a part and clicking it.
const POLL_ELSEWHERE: Duration = Duration::from_millis(50);

/// A panel's parts, and whether its window was last left taking clicks:
/// `None` until the first look, so the first look always applies.
struct Area {
  rects: Vec<IslandRect>,
  takes_clicks: Option<bool>,
}

/// The panels that let clicks through, by window label.
static AREAS: Mutex<BTreeMap<String, Area>> = Mutex::new(BTreeMap::new());
/// A thread is looking at the pointer.
static WATCHING: AtomicBool = AtomicBool::new(false);
/// The pointer was on a panel's parts at the last look.
static OVER: AtomicBool = AtomicBool::new(false);

fn areas() -> std::sync::MutexGuard<'static, BTreeMap<String, Area>> {
  AREAS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Where a panel's parts are, so clicks on them reach it and clicks anywhere
/// else in its window pass through; `None` makes it an ordinary panel again,
/// taking every click.
#[tauri::command]
pub fn set_standalone_listbox_hit_area(
  app: AppHandle,
  panel: String,
  rects: Option<Vec<IslandRect>>,
) {
  let Some(rects) = rects else {
    if areas().remove(&panel).is_some() {
      if let Some(window) = app.get_webview_window(&panel) {
        let _ = set_takes_clicks(&window, true);
      }
    }
    return;
  };
  match areas().entry(panel) {
    Entry::Occupied(mut area) => area.get_mut().rects = rects,
    Entry::Vacant(slot) => {
      slot.insert(Area {
        rects,
        takes_clicks: None,
      });
    }
  }
  // The parts changing shape can bring an edge to the pointer, or take it
  // away, without the pointer moving.
  let _ = app.run_on_main_thread({
    let app = app.clone();
    move || update(&app)
  });
  watch(app);
}

/// Forgets a panel put away, so its window takes clicks again when it next
/// shows something else.
pub(super) fn forget(panel: &str) {
  areas().remove(panel);
}

#[cfg(target_os = "windows")]
fn set_takes_clicks(window: &tauri::WebviewWindow, takes: bool) -> tauri::Result<()> {
  super::platform::set_pointer_passthrough(window, !takes)
}

#[cfg(not(target_os = "windows"))]
fn set_takes_clicks(window: &tauri::WebviewWindow, takes: bool) -> tauri::Result<()> {
  window.set_ignore_cursor_events(!takes)
}

/// Whether the pointer is on one of `rects`, measured from the window's
/// content in CSS pixels.
fn is_over(app: &AppHandle, window: &tauri::WebviewWindow, rects: &[IslandRect]) -> bool {
  let (Ok(pointer), Ok(origin), Ok(scale)) = (
    app.cursor_position(),
    window.inner_position(),
    window.scale_factor(),
  ) else {
    return false;
  };
  let x = (pointer.x - f64::from(origin.x)) / scale;
  let y = (pointer.y - f64::from(origin.y)) / scale;
  is_on_island(rects, x, y)
}

/// Lets each panel take clicks while the pointer is on its parts, and pass
/// them through while it is not. Only a change is sent to a window.
fn update(app: &AppHandle) {
  let mut over_any = false;
  for (label, area) in areas().iter_mut() {
    let Some(window) = app.get_webview_window(label) else {
      continue;
    };
    if !window.is_visible().unwrap_or(false) {
      continue;
    }
    let over = is_over(app, &window, &area.rects);
    over_any |= over;
    if area.takes_clicks != Some(over) && set_takes_clicks(&window, over).is_ok() {
      area.takes_clicks = Some(over);
    }
  }
  OVER.store(over_any, Ordering::Relaxed);
}

/// Looks at the pointer for as long as any panel lets clicks through.
fn watch(app: AppHandle) {
  if WATCHING.swap(true, Ordering::AcqRel) {
    return;
  }
  std::thread::spawn(move || loop {
    std::thread::sleep(if OVER.load(Ordering::Relaxed) {
      POLL_ON_PARTS
    } else {
      POLL_ELSEWHERE
    });
    if areas().is_empty() {
      WATCHING.store(false, Ordering::Release);
      // A panel registered as this one gave up is watched by a thread of
      // its own.
      if areas().is_empty() || WATCHING.swap(true, Ordering::AcqRel) {
        break;
      }
    }
    let target = app.clone();
    if app.run_on_main_thread(move || update(&target)).is_err() {
      WATCHING.store(false, Ordering::Release);
      break;
    }
  });
}
