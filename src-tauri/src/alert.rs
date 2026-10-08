// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A failure the user has to hear about, said in a window the app draws itself.
//!
//! It belongs to no other window: a failure can come from the tray, a shortcut
//! or the computer waking, with nothing of Screenwide's on screen. So it is
//! centred on the display under the pointer, floats above everything and
//! stays out of every capture, since a recording is never the place for it.
//!
//! Alerts queue. One that arrives while another is showing waits its turn, so
//! a capture failure followed by a stop failure loses neither.

use std::collections::VecDeque;
use std::sync::Mutex;

use serde::Serialize;
use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{
  AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow, WindowEvent,
};
use ts_rs::TS;

use crate::app_windows::{self, WindowLabel};
use crate::editor::export_window::presentation;

/// Sent to an already-loaded alert when it is to say something else. The
/// first presentation has no one listening yet and reads the words itself.
const SHOWN_EVENT: &str = "alert://shown";

/// The alert's width. Its height comes from the words: the window fits
/// itself to its content once the page has laid out.
const WIDTH: f64 = 420.0;
/// Where the window starts before that fit lands. Never seen: the window is
/// concealed until it has been fitted.
const INITIAL_HEIGHT: f64 = 148.0;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AlertCopy {
  pub message: String,
  pub title: String,
}

/// The display an alert was put on, in logical pixels, kept so the fit that
/// follows centres it on the same one wherever the pointer has gone since.
#[derive(Clone, Copy)]
struct Area {
  origin: LogicalPosition<f64>,
  size: LogicalSize<f64>,
}

#[derive(Default)]
struct Pending {
  area: Option<Area>,
  queue: VecDeque<AlertCopy>,
}

impl Pending {
  /// Queues `copy` unless the same alert is already showing or waiting, and
  /// reports whether it is the one now in front. Retrying a start that keeps
  /// failing for one reason is one problem, and says so once rather than
  /// leaving an alert behind for every attempt.
  fn enqueue(&mut self, copy: AlertCopy) -> bool {
    if self.queue.contains(&copy) {
      return false;
    }
    self.queue.push_back(copy);
    self.queue.len() == 1
  }
}

#[derive(Default)]
pub struct AlertState(Mutex<Pending>);

fn pending(app: &AppHandle) -> std::sync::MutexGuard<'_, Pending> {
  app
    .state::<AlertState>()
    .inner()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Tells the user something failed. Callable from any thread; the window is
/// handled on the main thread.
pub fn show(app: &AppHandle, title: &str, message: &str) {
  let first = pending(app).enqueue(AlertCopy {
    message: message.to_owned(),
    title: title.to_owned(),
  });
  // A later alert is shown when the one in front is dismissed.
  if !first {
    return;
  }
  let task_app = app.clone();
  let _ = app.run_on_main_thread(move || {
    if let Err(error) = present(&task_app) {
      eprintln!("Could not show the alert: {error}");
    }
  });
}

/// The window is prebuilt on Windows so a failure reported from inside an IPC
/// callback never creates a WebView synchronously, as the tooltip's is.
#[cfg(target_os = "windows")]
pub fn initialize(app: &AppHandle) -> tauri::Result<()> {
  get_or_create(app).map(|_| ())
}

fn get_or_create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  let label = WindowLabel::Alert.as_str();
  if let Some(window) = app.get_webview_window(label) {
    return Ok(window);
  }

  let effect = if cfg!(target_os = "windows") {
    Effect::Mica
  } else {
    Effect::UnderWindowBackground
  };
  let window = app_windows::webview_window(app, label, WebviewUrl::App("/alert".into()))
    .inner_size(WIDTH, INITIAL_HEIGHT)
    .always_on_top(true)
    // OK is the only way out, so there is nothing for a title bar to offer.
    .decorations(false)
    .minimizable(false)
    .maximizable(false)
    .resizable(false)
    .shadow(true)
    .skip_taskbar(true)
    .transparent(true)
    .accept_first_mouse(true)
    .visible(false)
    .effects(WindowEffectsConfig {
      color: None,
      effects: vec![effect],
      radius: Some(10.0),
      state: Some(EffectState::Active),
    })
    .build()?;
  // The effect's radius is macOS-only; on Windows the frameless alert asks
  // DWM for the corner it would give a framed window.
  #[cfg(target_os = "windows")]
  app_windows::round_corners(&window)?;

  let close_app = app.clone();
  window.on_window_event(move |event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
      api.prevent_close();
      dismiss(&close_app);
    }
  });
  Ok(window)
}

/// The work area of the display under the pointer, or of the primary display
/// when the pointer cannot be placed.
fn pointer_area(app: &AppHandle) -> Option<Area> {
  let monitor = app
    .cursor_position()
    .ok()
    .and_then(|point| app.monitor_from_point(point.x, point.y).ok().flatten())
    .or_else(|| app.primary_monitor().ok().flatten())?;
  let scale = monitor.scale_factor();
  let work_area = monitor.work_area();
  Some(Area {
    origin: work_area.position.to_logical(scale),
    size: work_area.size.to_logical(scale),
  })
}

fn place(window: &WebviewWindow, area: Option<Area>, height: f64) -> tauri::Result<()> {
  let size = LogicalSize::new(WIDTH, height);
  window.set_size(size)?;
  if let Some(area) = area {
    window.set_position(app_windows::centered_logical_position(
      area.origin,
      area.size,
      size,
    ))?;
  }
  Ok(())
}

fn present(app: &AppHandle) -> tauri::Result<()> {
  let area = pointer_area(app);
  let copy = {
    let mut pending = pending(app);
    pending.area = area;
    pending.queue.front().cloned()
  };
  let Some(copy) = copy else {
    return Ok(());
  };
  let window = get_or_create(app)?;
  place(&window, area, INITIAL_HEIGHT)?;
  #[cfg(target_os = "macos")]
  crate::capture_overlays::set_level(&window, crate::capture_overlays::FOREGROUND_LEVEL + 1)
    .map_err(|error| tauri::Error::Anyhow(std::io::Error::other(error).into()))?;
  app_windows::exclude_from_capture(&window)?;
  // Shown concealed: a hidden webview does not lay out, and the page has to
  // measure the words before the window takes its height.
  presentation::conceal(app, &window)?;
  app_windows::show(&window, true)?;
  // A reused window is already loaded and would otherwise still be showing
  // the last alert; a fresh one has no listener yet and asks on mount.
  let _ = app.emit_to(WindowLabel::Alert.as_str(), SHOWN_EVENT, copy);
  Ok(())
}

/// Takes the alert in front away, and shows the next one if another is
/// waiting.
fn dismiss(app: &AppHandle) {
  let next = {
    let mut pending = pending(app);
    pending.queue.pop_front();
    pending.queue.front().cloned()
  };
  match next {
    // The window stays up; the page measures the new words and refits.
    Some(next) => {
      let _ = app.emit_to(WindowLabel::Alert.as_str(), SHOWN_EVENT, next);
    }
    None => {
      if let Some(window) = app.get_webview_window(WindowLabel::Alert.as_str()) {
        let _ = app_windows::hide_without_focus_transfer(&window);
      }
    }
  }
}

/// What the alert should say. Read once on mount; every later alert arrives
/// as an event instead.
#[tauri::command]
pub fn get_alert(app: AppHandle) -> Option<AlertCopy> {
  pending(&app).queue.front().cloned()
}

/// The alert's content reporting the height it needs, which is when the
/// window is sized, centred and revealed.
#[tauri::command]
pub fn fit_alert(app: AppHandle, height: f64) -> Result<(), String> {
  if !height.is_finite() || height <= 0.0 {
    return Err("The alert height must be positive".to_owned());
  }
  let area = pending(&app).area;
  let Some(window) = app.get_webview_window(WindowLabel::Alert.as_str()) else {
    return Ok(());
  };
  place(&window, area, height.ceil()).map_err(|error| error.to_string())?;
  presentation::reveal_after_resize(&window).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn dismiss_alert(app: AppHandle) {
  dismiss(&app);
}

#[cfg(test)]
mod tests {
  use super::{AlertCopy, Pending};

  fn copy(title: &str) -> AlertCopy {
    AlertCopy {
      message: "The camera no longer offers 1920 × 1080 at 30 fps.".to_owned(),
      title: title.to_owned(),
    }
  }

  #[test]
  fn a_repeated_failure_is_said_once() {
    let mut pending = Pending::default();

    assert!(pending.enqueue(copy("Recording could not start")));
    for _ in 0..4 {
      assert!(!pending.enqueue(copy("Recording could not start")));
    }

    assert_eq!(pending.queue.len(), 1);
  }

  #[test]
  fn different_failures_wait_their_turn() {
    let mut pending = Pending::default();

    assert!(pending.enqueue(copy("Recording ran into a problem")));
    assert!(!pending.enqueue(copy("Recording could not finish")));

    assert_eq!(pending.queue.len(), 2);
  }
}
