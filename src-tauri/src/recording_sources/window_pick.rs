// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Picking the window to record by clicking it. While picking, the region
//! overlay cuts the window under the pointer out of the dimmed, blurred
//! desktop, and the island names it. The windows are read once as picking starts, in
//! the order they are stacked, so following the pointer is a lookup rather
//! than a question to the system each time it moves.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Emitter, WebviewWindow};

mod backdrop;
#[cfg(target_os = "macos")]
mod clicks_macos;
mod hover;
mod raise;
use backdrop::Backdrops;
use hover::{hover_of, monitor_at, WindowPickHover};
mod targets;
use targets::{pointer, read_targets, Target};

use super::{application_icon_cache_dir, list_monitors, platform, MonitorDetails, WindowDetails};

/// How often the pointer is looked for while picking.
const POLL_INTERVAL: Duration = Duration::from_millis(16);
const HOVERED_EVENT: &str = "window-pick://hovered";
const PICKED_EVENT: &str = "window-pick://picked";
const ENDED_EVENT: &str = "window-pick://ended";
/// How long the overlay takes to fade once picking ends (`LEAVE_MS` in
/// `use-window-pick-overlay.ts`), after which its backdrop goes.
const FADE_OUT: Duration = Duration::from_millis(150);

struct Pick {
  generation: u64,
  /// The display the overlay stays anchored to for the whole pick. Moving its
  /// anchor would move its window and rebuild its surfaces, which shows one
  /// display's picture on another for a frame.
  anchor: u32,
  /// Front to back, so the first that holds the pointer is the one seen.
  targets: Vec<Target>,
  monitors: Vec<MonitorDetails>,
  hovered: Option<usize>,
  /// Where the pointer was last seen. A pick moved by the keyboard holds
  /// until the pointer moves again.
  pointer: Option<(f64, f64)>,
  icon_dir: PathBuf,
  /// Each app's icon, by process, as it is first hovered.
  icons: HashMap<u32, Option<PathBuf>>,
  backdrops: Backdrops,
}

static PICK: Mutex<Option<Pick>> = Mutex::new(None);
static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

fn pick() -> std::sync::MutexGuard<'static, Option<Pick>> {
  PICK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Moves the pick to the window under the pointer when the pointer has
/// moved, and says so when that changes which window it is.
fn follow_pointer(pick: &mut Pick) -> Option<Option<WindowPickHover>> {
  let point = pointer()?;
  if pick.pointer == Some(point) {
    return None;
  }
  pick.pointer = Some(point);
  let hovered = pick
    .targets
    .iter()
    .position(|target| target.contains(point));
  if hovered == pick.hovered {
    return None;
  }
  pick.hovered = hovered;
  Some(hover_of(pick))
}

/// Says which window the pick is on, with its app's icon. An app's icon is
/// looked up the first time one of its windows is hovered, outside the lock,
/// as it may be extracted from the app on disk.
fn announce(app: &AppHandle, generation: u64, hover: Option<WindowPickHover>) {
  let missing = {
    let guard = pick();
    guard
      .as_ref()
      .filter(|pick| pick.generation == generation)
      .and_then(|pick| {
        let target = &pick.targets[pick.hovered?];
        (!pick.icons.contains_key(&target.pid)).then(|| (target.pid, pick.icon_dir.clone()))
      })
  };
  let hover = match missing {
    Some((pid, icon_dir)) => {
      let icon = platform::app_icon(&icon_dir, pid);
      let mut guard = pick();
      let Some(pick) = guard.as_mut().filter(|pick| pick.generation == generation) else {
        return;
      };
      pick.icons.insert(pid, icon);
      hover_of(pick)
    }
    None => hover,
  };
  let _ = app.emit(HOVERED_EVENT, hover);
}

fn follow(app: AppHandle, generation: u64) {
  std::thread::spawn(move || loop {
    std::thread::sleep(POLL_INTERVAL);
    let change = {
      let mut guard = pick();
      let Some(pick) = guard.as_mut().filter(|pick| pick.generation == generation) else {
        break;
      };
      follow_pointer(pick)
    };
    if let Some(hover) = change {
      announce(&app, generation, hover);
    }
  });
}

/// Starts picking, from the window under the pointer, with the overlay
/// anchored to `anchor_monitor_id`: the display it is already on, or else the
/// pointer's.
#[tauri::command]
pub async fn start_window_pick(
  app: AppHandle,
  anchor_monitor_id: Option<u32>,
) -> Result<(), String> {
  let reading = tauri::async_runtime::spawn_blocking(read_targets);
  let monitors = list_monitors(app.clone())?;
  let anchor = anchor_monitor_id
    .filter(|id| monitors.iter().any(|monitor| monitor.id == *id))
    .or_else(|| {
      pointer()
        .and_then(|point| monitor_at(&monitors, point))
        .map(|monitor| monitor.id)
    })
    .or_else(|| monitors.first().map(|monitor| monitor.id))
    .ok_or_else(|| "There is no display to pick a window on".to_owned())?;
  let backdrops = Backdrops::capture(&monitors).await;
  let targets = reading.await.map_err(|error| error.to_string())??;
  let icon_dir = application_icon_cache_dir(&app)?;
  let generation = NEXT_GENERATION.fetch_add(1, Ordering::Relaxed);
  let hover = {
    let mut guard = pick();
    let pick = guard.insert(Pick {
      generation,
      anchor,
      targets,
      monitors,
      hovered: None,
      pointer: None,
      icon_dir,
      icons: HashMap::new(),
      backdrops,
    });
    follow_pointer(pick).unwrap_or(None)
  };
  // Said even when nothing is under the pointer: hearing it is what tells
  // the overlay and the island that picking has begun.
  announce(&app, generation, hover);
  #[cfg(target_os = "macos")]
  clicks_macos::catch(&app);
  follow(app, generation);
  Ok(())
}

/// Moves the pick by `delta` windows through the stack, for the keyboard.
#[tauri::command]
pub fn step_window_pick(app: AppHandle, delta: i32) {
  let stepped = {
    let mut guard = pick();
    let Some(pick) = guard.as_mut() else {
      return;
    };
    let count = pick.targets.len() as i32;
    if count == 0 {
      return;
    }
    let next = match pick.hovered {
      Some(index) => (index as i32 + delta).rem_euclid(count),
      None if delta < 0 => count - 1,
      None => 0,
    };
    pick.hovered = Some(next as usize);
    (pick.generation, hover_of(pick))
  };
  announce(&app, stepped.0, stepped.1);
}

/// Puts every display's frozen desktop behind the overlay, as the overlay asks
/// once it is attached.
#[tauri::command]
pub fn show_window_pick_backdrop(window: WebviewWindow) {
  let Some(images) = pick().as_ref().map(|pick| pick.backdrops.images()) else {
    return;
  };
  if !backdrop::install(&window, &images) {
    eprintln!("Could not put the desktop behind window picking");
  }
}

/// Ends picking: with the window it is on, when `chosen` is set, or with
/// none. A window picked is announced (`window-pick://picked`) and brought to
/// the front.
fn end(app: &AppHandle, chosen: bool) {
  let Some(state) = pick().take() else {
    return;
  };
  #[cfg(target_os = "macos")]
  clicks_macos::release(app);
  let _ = app.emit(ENDED_EVENT, ());
  let chosen = chosen
    .then_some(state.hovered)
    .flatten()
    .map(|index| state.targets[index].clone());
  let faded = app.clone();
  std::thread::spawn(move || {
    std::thread::sleep(FADE_OUT);
    backdrop::release(&faded);
  });
  let Some(target) = chosen else {
    return;
  };
  // Raised at once, under the overlay as it fades. Off this thread, as an
  // app may take a moment to answer Accessibility.
  let raised = target.clone();
  std::thread::spawn(move || raise::raise(&raised));
  let picked = WindowDetails {
    id: target.id,
    pid: target.pid,
    app_icon_path: state
      .icons
      .get(&target.pid)
      .cloned()
      .flatten()
      .or_else(|| platform::app_icon(&state.icon_dir, target.pid)),
    app_name: target.app_name,
    title: target.title,
    position: target.position,
    size: target.size,
  };
  let _ = app.emit(PICKED_EVENT, picked);
}

/// Picks the window the pick is on; over none, picking goes on.
pub(super) fn pick_hovered(app: &AppHandle) {
  let has_window = pick().as_ref().is_some_and(|pick| pick.hovered.is_some());
  if has_window {
    end(app, true);
  }
}

#[tauri::command]
pub fn pick_window(app: AppHandle) {
  pick_hovered(&app);
}

#[tauri::command]
pub fn cancel_window_pick(app: AppHandle) {
  cancel(&app);
}

pub(crate) fn is_active() -> bool {
  pick().is_some()
}

/// Ends picking without a window, for whatever puts the controls away.
pub(crate) fn cancel(app: &AppHandle) {
  end(app, false);
}
