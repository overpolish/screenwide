// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Recording source-selector popup layout and visibility lifecycle.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::{
  platform,
  source_selector_layout::{selector_frame, SelectorFrame, SelectorPlacement},
  transient_popover::TransientPopover,
  WindowLabel,
};

static KEYBOARD_FOCUS: AtomicBool = AtomicBool::new(false);
static POPOVER: TransientPopover = TransientPopover::new();
static VISIBLE: AtomicBool = AtomicBool::new(true);
static WINDOW_SELECTOR_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorState {
  expanded: bool,
  focus_contents: bool,
  placement: SelectorPlacement,
  revision: u64,
}

fn frame(app: &AppHandle) -> tauri::Result<(SelectorPlacement, SelectorFrame)> {
  selector_frame(app, WINDOW_SELECTOR_ACTIVE.load(Ordering::Relaxed))
}

fn apply_frame(window: &tauri::WebviewWindow, frame: SelectorFrame) -> tauri::Result<()> {
  platform::set_frame(window, frame.position, frame.size)
}

fn emit_state(app: &AppHandle, placement: SelectorPlacement) -> tauri::Result<()> {
  app.emit_to(
    WindowLabel::RecordingSourceSelector.as_str(),
    "recording-source-selector://state",
    SelectorState {
      expanded: POPOVER.is_open(),
      focus_contents: KEYBOARD_FOCUS.load(Ordering::Relaxed),
      placement,
      revision: POPOVER.revision(),
    },
  )
}

#[tauri::command]
pub fn get_recording_source_selector_state(app: AppHandle) -> tauri::Result<SelectorState> {
  let (placement, _) = frame(&app)?;
  Ok(SelectorState {
    expanded: POPOVER.is_open(),
    focus_contents: KEYBOARD_FOCUS.load(Ordering::Relaxed),
    placement,
    revision: POPOVER.revision(),
  })
}

pub(super) fn reposition(app: &AppHandle) -> tauri::Result<()> {
  let _lifecycle = POPOVER.lock();
  let selector = app
    .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let (placement, target) = frame(app)?;
  POPOVER.touch();
  if POPOVER.is_open() {
    apply_frame(&selector, target)?;
  }
  emit_state(app, placement)
}

pub(super) fn is_expanded() -> bool {
  POPOVER.is_open()
}

pub(super) fn is_visible() -> bool {
  VISIBLE.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn expand_recording_source_selector(
  app: AppHandle,
  focus_contents: bool,
  window_selector: bool,
) -> tauri::Result<()> {
  // A recording hides this chrome deliberately; nothing may bring it back
  // until the recording is over.
  if !crate::recording::is_idle(&app) {
    return Ok(());
  }
  if !is_visible() {
    return Ok(());
  }

  let _lifecycle = POPOVER.lock();
  if is_expanded() {
    return Ok(());
  }

  let window = app
    .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  WINDOW_SELECTOR_ACTIVE.store(window_selector, Ordering::Relaxed);
  let (placement, expanded) = frame(&app)?;
  apply_frame(&window, expanded)?;
  platform::show(&window, 1.0)?;
  if focus_contents {
    if let Err(error) = window.set_focus() {
      let _ = platform::hide(&window);
      return Err(error);
    }
  }
  KEYBOARD_FOCUS.store(focus_contents, Ordering::Relaxed);
  POPOVER.set_open(true);
  emit_state(&app, placement)?;

  Ok(())
}

pub fn collapse(app: AppHandle, return_focus: Option<bool>) -> tauri::Result<()> {
  let _lifecycle = POPOVER.lock();
  if !is_expanded() {
    return Ok(());
  }
  let window = app
    .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let return_focus = return_focus.unwrap_or_else(|| KEYBOARD_FOCUS.load(Ordering::Relaxed));
  KEYBOARD_FOCUS.store(false, Ordering::Relaxed);
  let (placement, _) = frame(&app)?;
  platform::hide(&window)?;
  POPOVER.set_open(false);
  emit_state(&app, placement)?;
  if return_focus {
    if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
      bar.set_focus()?;
    }
  }
  Ok(())
}

#[tauri::command]
pub fn collapse_recording_source_selector(
  app: AppHandle,
  return_focus: Option<bool>,
) -> tauri::Result<()> {
  collapse(app, return_focus)
}

pub(super) fn hide(app: &AppHandle) -> tauri::Result<()> {
  let _lifecycle = POPOVER.lock();
  KEYBOARD_FOCUS.store(false, Ordering::Relaxed);
  if let Some(selector) = app.get_webview_window(WindowLabel::RecordingSourceSelector.as_str()) {
    platform::hide(&selector)?;
  }
  POPOVER.set_open(false);
  if let Ok((placement, _)) = frame(app) {
    emit_state(app, placement)?;
  }
  Ok(())
}

pub(super) fn dismiss_if_outside(app: &AppHandle, open_on_press: bool, x: f64, y: f64) {
  if POPOVER.should_dismiss(
    app,
    open_on_press,
    false,
    x,
    y,
    &[WindowLabel::RecordingSourceSelector],
  ) {
    let _ = collapse(app.clone(), Some(false));
  }
}

#[tauri::command]
pub fn set_recording_source_selector_visible(app: AppHandle, visible: bool) -> tauri::Result<()> {
  VISIBLE.store(visible, Ordering::Relaxed);
  if visible {
    Ok(())
  } else {
    hide(&app)
  }
}
