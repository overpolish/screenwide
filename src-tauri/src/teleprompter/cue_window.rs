// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The cue being read towards, in a window of its own beside the
//! teleprompter, on whichever side of it has more room on its display, as
//! tall as the teleprompter unless resized: a size set by hand is kept as
//! the tallest it grows to. Like the teleprompter it floats above every app
//! and is left out of every capture. Where neither side has room, the
//! teleprompter is told, and shows the cue over its text instead.

mod making;
mod placement;
mod reshape;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalSize};
use ts_rs::TS;

pub(crate) use self::placement::ReadingLine;
use self::placement::{beside, Wanted};
use super::window::LABEL as TELEPROMPTER;

pub(crate) const LABEL: &str = "teleprompter-cue";
/// Tells the cue's window what to show, or that the cue has gone.
const SHOW_EVENT: &str = "teleprompter-cue://show";
/// Tells the teleprompter whether the cue stands beside it.
const BESIDE_EVENT: &str = "teleprompter://cue-beside";

/// A cue's picture as it is shown beside the teleprompter.
#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TeleprompterCuePicture")]
pub struct CuePicture {
  pub path: PathBuf,
  /// Its project's name, for assistive technology.
  pub title: String,
  /// Drawn flipped, as the teleprompter is for a rig's glass.
  pub mirrored: bool,
}

#[derive(Default)]
struct Shown {
  picture: Option<CuePicture>,
  /// The picture's size, in pixels.
  picture_size: (u32, u32),
  /// The height kept from a resize by hand, in points.
  height: Option<f64>,
  line: ReadingLine,
  /// The size last given to the window.
  placed: Option<PhysicalSize<u32>>,
}

impl Shown {
  /// What the window is placed for, while there is a cue.
  fn wanted(&self) -> Option<Wanted> {
    self.picture.as_ref().map(|_| Wanted {
      kept: self.height,
      line: self.line,
      picture: self.picture_size,
    })
  }
}

/// Counts the teleprompter's moves, so only the last places the cue.
static FOLLOWS: AtomicU64 = AtomicU64::new(0);
/// How long the teleprompter rests before the cue is placed afresh.
const SETTLE: Duration = Duration::from_millis(200);

static SHOWN: Mutex<Option<Shown>> = Mutex::new(None);

fn shown() -> MutexGuard<'static, Option<Shown>> {
  SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Shows `picture` beside the teleprompter, level with its reading `line`,
/// or puts the cue away when none, a resize by hand having kept `height`
/// points. False when there is no room beside it, for the teleprompter to
/// show the cue itself.
#[tauri::command]
pub fn show_teleprompter_cue(
  app: AppHandle,
  picture: Option<CuePicture>,
  height: Option<f64>,
  line: Option<ReadingLine>,
) -> Result<bool, String> {
  let Some(picture) = picture else {
    if let Some(state) = shown().as_mut() {
      state.picture = None;
    }
    let _ = app.emit_to(LABEL, SHOW_EVENT, None::<CuePicture>);
    return Ok(true);
  };
  let picture_size = image::image_dimensions(&picture.path).map_err(|error| error.to_string())?;
  {
    let mut state = shown();
    let state = state.get_or_insert_with(Shown::default);
    state.picture_size = picture_size;
    state.height = height;
    state.line = line.unwrap_or_default();
    state.picture = Some(picture);
  }
  place(&app)
}

/// The cue to show, which its window asks for as its page loads.
#[tauri::command]
pub fn teleprompter_cue() -> Option<CuePicture> {
  shown().as_ref().and_then(|state| state.picture.clone())
}

/// Takes the cue's window off screen once it has faded, unless a cue came
/// back meanwhile.
#[tauri::command]
pub fn hide_teleprompter_cue(app: AppHandle) -> Result<(), String> {
  if teleprompter_cue().is_some() {
    return Ok(());
  }
  hide(&app);
  Ok(())
}

/// The teleprompter moved or resized. On macOS the cue, a child of its
/// window, moves with it as one; elsewhere, and as the teleprompter is
/// resized, which a child does not follow, it is moved along at once. Once
/// the teleprompter has come to rest the cue is placed afresh, as the room
/// beside it may have changed sides or gone, and the teleprompter hears
/// whether it still stands beside.
pub(crate) fn follow(app: &AppHandle, resized: bool) {
  if teleprompter_cue().is_none() {
    return;
  }
  let moved = FOLLOWS.fetch_add(1, Ordering::Relaxed) + 1;
  if resized || cfg!(not(target_os = "macos")) {
    move_along(app, resized);
  }
  let app = app.clone();
  std::thread::spawn(move || {
    std::thread::sleep(SETTLE);
    if FOLLOWS.load(Ordering::Relaxed) != moved {
      return;
    }
    match place(&app) {
      Ok(beside) => {
        let _ = app.emit_to(TELEPROMPTER, BESIDE_EVENT, beside);
      }
      Err(error) => eprintln!("Could not move the cue: {error}"),
    }
  });
}

/// Moves the cue as it stands beside the teleprompter, and sizes it again
/// where `resized`, as the teleprompter's own height or room may have
/// changed.
fn move_along(app: &AppHandle, resized: bool) {
  let (Some(teleprompter), Some(window)) = (
    app.get_webview_window(TELEPROMPTER),
    app.get_webview_window(LABEL),
  ) else {
    return;
  };
  let Some(wanted) = shown().as_ref().and_then(Shown::wanted) else {
    return;
  };
  let Ok(Some((position, size))) = beside(&teleprompter, &wanted) else {
    return;
  };
  reshape::stop();
  if resized {
    if let Some(state) = shown().as_mut() {
      state.placed = Some(size);
    }
    let _ = window.set_size(size);
  }
  let _ = window.set_position(position);
}

/// The teleprompter closed: so does its cue.
pub(crate) fn close(app: &AppHandle) {
  *shown() = None;
  if let Some(window) = app.get_webview_window(LABEL) {
    let _ = window.close();
  }
}

/// Places the cue beside the teleprompter and shows it there. False, with
/// the window off screen, when there is no room for it. A window not made
/// yet is made first, and the teleprompter hears where the cue went then.
fn place(app: &AppHandle) -> Result<bool, String> {
  let teleprompter = app
    .get_webview_window(TELEPROMPTER)
    .ok_or_else(|| "The teleprompter is not open".to_owned())?;
  let Some(wanted) = shown().as_ref().and_then(Shown::wanted) else {
    return Ok(true);
  };
  let Some((position, size)) = beside(&teleprompter, &wanted)? else {
    hide(app);
    return Ok(false);
  };
  let Some(window) = app.get_webview_window(LABEL) else {
    making::build_then_place(app);
    return Ok(true);
  };
  if let Some(state) = shown().as_mut() {
    state.placed = Some(size);
  }
  // A cue already showing eases into the next one's shape as the two
  // cross-fade; one coming out of nothing starts where it belongs.
  if window.is_visible().unwrap_or(false) {
    reshape::change(&window, (position, size))?;
  } else {
    reshape::stop();
    window.set_size(size).map_err(|error| error.to_string())?;
    window
      .set_position(position)
      .map_err(|error| error.to_string())?;
  }
  let _ = app.emit_to(LABEL, SHOW_EVENT, teleprompter_cue());
  window.show().map_err(|error| error.to_string())?;
  #[cfg(target_os = "macos")]
  crate::app_windows::options::presentation::attach(app, &teleprompter, &window)
    .map_err(|error| error.to_string())?;
  Ok(true)
}

/// Takes the cue's window off screen. On macOS it is let go of the
/// teleprompter first: ordering out a child still attached would take its
/// parent with it.
fn hide(app: &AppHandle) {
  let Some(window) = app.get_webview_window(LABEL) else {
    return;
  };
  #[cfg(target_os = "macos")]
  if let Some(teleprompter) = app.get_webview_window(TELEPROMPTER) {
    let _ = crate::app_windows::options::presentation::detach(app, &teleprompter, &window);
  }
  let _ = window.hide();
}
