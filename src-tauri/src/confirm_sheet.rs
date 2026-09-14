// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A confirmation the app draws itself, in its own panel, rather than handing
//! the question to the system dialog.
//!
//! One window serves every caller: it is built the first time something asks
//! and then hidden and shown again, the way the other panels here work. The
//! caller supplies the words and a closure to run with the answer, so the
//! sheet knows nothing about what it is confirming and there is only ever one
//! question outstanding.

use std::sync::Mutex;

use serde::Serialize;
use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{
  AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
  WindowEvent,
};

use crate::editor::export_window::presentation;
use crate::windows::{self, WindowLabel};

/// Sent to an already-loaded sheet when a new question arrives. The first
/// presentation has no one listening yet and reads the words itself instead.
const ASKED_EVENT: &str = "confirm-sheet://asked";

/// The sheet's width. Its height comes from the words: the window fits itself
/// to its content once the page has laid out.
const WIDTH: f64 = 420.0;
/// Where the window starts before that fit lands.
const INITIAL_HEIGHT: f64 = 148.0;

/// Everything the sheet says. Named by role rather than by the action it
/// stands for, because the sheet has no idea what that action is.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmCopy {
  pub cancel_label: String,
  pub confirm_label: String,
  pub message: String,
  pub title: String,
}

/// What the asker wants done with the answer.
type Answer = Box<dyn FnOnce(&AppHandle, bool) + Send>;

struct PendingConfirm {
  copy: ConfirmCopy,
  on_answer: Answer,
  /// The window the sheet hangs off, kept so it can be detached again.
  parent: WebviewWindow,
}

#[derive(Default)]
pub struct ConfirmSheetState(Mutex<Option<PendingConfirm>>);

fn take_pending(app: &AppHandle) -> Option<PendingConfirm> {
  app
    .state::<ConfirmSheetState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take()
}

/// The sheet window, built on first use and reused from then on.
fn get_or_create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
  let label = WindowLabel::ConfirmSheet.as_str();
  if let Some(window) = app.get_webview_window(label) {
    return Ok(window);
  }

  let effect = if cfg!(target_os = "windows") {
    Effect::Mica
  } else {
    Effect::UnderWindowBackground
  };
  let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("/confirm-sheet".into()))
    .title("Screenwide")
    .inner_size(WIDTH, INITIAL_HEIGHT)
    .always_on_top(true)
    // A sheet has no title bar of its own on any platform: the two buttons
    // are the only way out, so there is nothing for a caption to offer.
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
  // The effect's radius is macOS-only; on Windows the frameless sheet asks
  // DWM for the corner it would give a framed window.
  #[cfg(target_os = "windows")]
  crate::windows::round_corners(&window)?;

  let close_app = app.clone();
  window.on_window_event(move |event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
      api.prevent_close();
      // Dismissing the question is not answering it.
      answer(&close_app, false);
    }
  });

  Ok(window)
}

/// Puts the sheet in the middle of the window it belongs to. Physical pixels
/// throughout, so a parent straddling two displays of different scale factors
/// cannot make it drift.
fn center_on(parent: &WebviewWindow, sheet: &WebviewWindow) -> tauri::Result<()> {
  let parent_position = parent.outer_position()?;
  let parent_size = parent.outer_size()?;
  let size = sheet.outer_size()?;
  sheet.set_position(PhysicalPosition::new(
    parent_position.x + (parent_size.width as i32 - size.width as i32) / 2,
    parent_position.y + (parent_size.height as i32 - size.height as i32) / 2,
  ))
}

fn present(app: &AppHandle, parent: &WebviewWindow, copy: &ConfirmCopy) -> tauri::Result<()> {
  let sheet = get_or_create(app)?;
  center_on(parent, &sheet)?;
  // Conceal before attachment: AppKit can order a child onscreen when attached.
  presentation::conceal(app, &sheet)?;
  presentation::attach(app, parent, &sheet)?;
  // Restore the webview and focus on every opening, even if attachment has
  // already made the native window visible. It must lay out to report its size.
  windows::show(&sheet, true)?;
  // A reused sheet is already loaded and would otherwise still be showing the
  // last question; a fresh one has no listener yet and asks for the words on
  // mount instead.
  let _ = app.emit_to(
    WindowLabel::ConfirmSheet.as_str(),
    ASKED_EVENT,
    copy.clone(),
  );
  // The parent is told, so it can go inert the way it does under the export
  // sheet: a child window does not take the parent's input on macOS.
  let _ = app.emit_to(parent.label(), MODAL_EVENT, true);
  Ok(())
}

/// The sheet's content reporting the height it needs, which is when the
/// sheet is sized, centred over its parent and shown.
#[tauri::command]
pub fn fit_confirm_sheet(app: AppHandle, height: f64) -> Result<(), String> {
  if !height.is_finite() || height <= 0.0 {
    return Err("The confirmation sheet height must be positive".to_owned());
  }
  let parent = app
    .state::<ConfirmSheetState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .map(|pending| pending.parent.clone());
  let Some(parent) = parent else {
    return Ok(());
  };
  let Some(sheet) = app.get_webview_window(WindowLabel::ConfirmSheet.as_str()) else {
    return Ok(());
  };
  sheet
    .set_size(tauri::LogicalSize::new(WIDTH, height.ceil()))
    .map_err(|error| error.to_string())?;
  // Centre using the requested size: macOS has queued, not applied, the resize.
  let size = tauri::LogicalSize::new(WIDTH, height.ceil())
    .to_physical::<u32>(sheet.scale_factor().map_err(|error| error.to_string())?);
  let position = parent.outer_position().map_err(|error| error.to_string())?;
  let parent_size = parent.outer_size().map_err(|error| error.to_string())?;
  sheet
    .set_position(PhysicalPosition::new(
      position.x + (parent_size.width as i32 - size.width as i32) / 2,
      position.y + (parent_size.height as i32 - size.height as i32) / 2,
    ))
    .map_err(|error| error.to_string())?;
  presentation::reveal_after_resize(&sheet).map_err(|error| error.to_string())
}

/// Tells the asking window whether the sheet stands over it.
const MODAL_EVENT: &str = "confirm-sheet://modal";

/// Asks the user to confirm something, and runs `on_answer` once they have
/// said. A second ask while one is still on screen is ignored: the sheet holds
/// a single question, and the answer belongs to whoever asked first.
///
/// Nothing here waits. The sheet is a window like any other, and the answer
/// arrives later through `resolve_confirm_sheet`.
pub fn ask<F>(app: &AppHandle, parent: &WebviewWindow, copy: ConfirmCopy, on_answer: F)
where
  F: FnOnce(&AppHandle, bool) + Send + 'static,
{
  {
    let state = app.state::<ConfirmSheetState>();
    let mut pending = state
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if pending.is_some() {
      return;
    }
    // Claimed before the window is touched, so a close request arriving while
    // the sheet is coming up cannot start a second one.
    *pending = Some(PendingConfirm {
      copy: copy.clone(),
      on_answer: Box::new(on_answer),
      parent: parent.clone(),
    });
  }

  if let Err(error) = present(app, parent, &copy) {
    eprintln!("Could not show the confirmation sheet: {error}");
    // A question that cannot be put is not one the user declined to answer,
    // but the caller is owed a call either way, and cancelling is the answer
    // that changes nothing.
    if let Some(pending) = take_pending(app) {
      (pending.on_answer)(app, false);
    }
  }
}

/// Closes the sheet and hands the answer to whoever asked. Answering twice is
/// a no-op: the first answer takes the question with it.
pub fn answer(app: &AppHandle, confirmed: bool) {
  let Some(pending) = take_pending(app) else {
    return;
  };
  if let Some(sheet) = app.get_webview_window(WindowLabel::ConfirmSheet.as_str()) {
    // Detached first: ordering a still-attached child out drags its parent
    // with it.
    let _ = presentation::detach(app, &pending.parent, &sheet);
    let _ = windows::hide_without_focus_transfer(&sheet);
  }
  let _ = app.emit_to(pending.parent.label(), MODAL_EVENT, false);
  (pending.on_answer)(app, confirmed);
}

/// What the sheet should say. Read once on mount; every later question
/// arrives as an event instead.
#[tauri::command]
pub fn get_confirm_sheet(app: AppHandle) -> Option<ConfirmCopy> {
  app
    .state::<ConfirmSheetState>()
    .0
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
    .map(|pending| pending.copy.clone())
}

#[tauri::command]
pub fn resolve_confirm_sheet(app: AppHandle, confirmed: bool) {
  answer(&app, confirmed);
}
