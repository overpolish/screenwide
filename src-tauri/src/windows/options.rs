// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Mutex, MutexGuard,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager};

use super::{
  platform,
  recording_options_layout::{self, PopoverAnchor},
  transient_popover::{TransientPopover, TransientPopoverState},
  WindowLabel,
};

static OPTIONS_KEYBOARD_FOCUS: AtomicBool = AtomicBool::new(false);
static RECORDING_OPTIONS: TransientPopover = TransientPopover::new();
static STANDALONE_LISTBOX: TransientPopover = TransientPopover::new();
static STANDALONE_LISTBOX_CONTEXT: Mutex<Option<StandaloneListboxContext>> = Mutex::new(None);

#[derive(Clone)]
struct StandaloneListboxContext {
  focus_contents: bool,
  parent_window_label: String,
  trigger_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandaloneListboxClosed {
  return_focus: bool,
  trigger_id: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingOptionsState {
  focus_contents: bool,
  open: bool,
  revision: u64,
}

fn state() -> RecordingOptionsState {
  let TransientPopoverState { open, revision } = RECORDING_OPTIONS.state();
  RecordingOptionsState {
    focus_contents: OPTIONS_KEYBOARD_FOCUS.load(Ordering::Relaxed),
    open,
    revision,
  }
}

fn emit_state(app: &AppHandle) -> tauri::Result<()> {
  app.emit_to(
    WindowLabel::RecordingOptions.as_str(),
    "recording-options://state",
    state(),
  )
}

fn standalone_listbox_context() -> MutexGuard<'static, Option<StandaloneListboxContext>> {
  STANDALONE_LISTBOX_CONTEXT
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
pub fn get_recording_options_state() -> RecordingOptionsState {
  state()
}

#[tauri::command]
pub fn toggle_recording_options(
  app: AppHandle,
  anchor: PopoverAnchor,
  focus_contents: bool,
) -> tauri::Result<()> {
  if !crate::recording::is_idle(&app) {
    return Ok(());
  }

  let _lifecycle = RECORDING_OPTIONS.lock();
  if RECORDING_OPTIONS.is_open() {
    return close_recording_options_locked(&app, focus_contents);
  }

  let layout = recording_options_layout::set_anchor(anchor);
  let window = app
    .get_webview_window(WindowLabel::RecordingOptions.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let position = recording_options_layout::frame(&app, layout)?;
  platform::set_frame(
    &window,
    position,
    LogicalSize::new(recording_options_layout::WIDTH, layout.height),
  )?;
  platform::show(&window, 1.0)?;
  if let Err(error) = platform::restore_recording_level(&window) {
    let _ = platform::hide(&window);
    return Err(error);
  }
  if focus_contents {
    if let Err(error) = window.set_focus() {
      let _ = platform::hide(&window);
      return Err(error);
    }
  }

  OPTIONS_KEYBOARD_FOCUS.store(focus_contents, Ordering::Relaxed);
  RECORDING_OPTIONS.set_open(true);
  emit_state(&app)
}

fn close_recording_options_locked(app: &AppHandle, return_focus: bool) -> tauri::Result<()> {
  if !RECORDING_OPTIONS.is_open() {
    return Ok(());
  }

  close_standalone_listbox(app.clone(), false)?;
  crate::audio_preview::stop_all(app);
  crate::camera_preview::stop_all(app);
  if let Some(window) = app.get_webview_window(WindowLabel::RecordingOptions.as_str()) {
    platform::hide(&window)?;
  }
  OPTIONS_KEYBOARD_FOCUS.store(false, Ordering::Relaxed);
  RECORDING_OPTIONS.set_open(false);
  emit_state(app)?;

  if return_focus {
    if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
      bar.set_focus()?;
    }
  }

  Ok(())
}

pub(super) fn close_recording_options(app: AppHandle, return_focus: bool) -> tauri::Result<()> {
  let _lifecycle = RECORDING_OPTIONS.lock();
  close_recording_options_locked(&app, return_focus)
}

#[tauri::command]
pub fn hide_recording_options(app: AppHandle) -> tauri::Result<()> {
  close_recording_options(app, false)
}

#[tauri::command]
pub fn set_recording_options_content_height(app: AppHandle, height: f64) -> tauri::Result<()> {
  if !height.is_finite() || height <= 0.0 {
    return Ok(());
  }
  let height = height.ceil().clamp(1.0, 10_000.0);
  let _lifecycle = RECORDING_OPTIONS.lock();
  let Some(layout) = recording_options_layout::set_height(height) else {
    return Ok(());
  };

  if !RECORDING_OPTIONS.is_open() {
    return Ok(());
  }
  let window = app
    .get_webview_window(WindowLabel::RecordingOptions.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let position = recording_options_layout::frame(&app, layout)?;
  platform::set_frame(
    &window,
    position,
    LogicalSize::new(recording_options_layout::WIDTH, layout.height),
  )
}

#[tauri::command]
pub fn show_standalone_listbox(
  app: AppHandle,
  focus_contents: bool,
  parent_window_label: String,
  trigger_id: String,
  offset: LogicalPosition<f64>,
  size: LogicalSize<f64>,
) -> tauri::Result<()> {
  let _lifecycle = STANDALONE_LISTBOX.lock();
  let parent = app
    .get_webview_window(&parent_window_label)
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let window = app
    .get_webview_window(WindowLabel::StandaloneListbox.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let scale = parent.scale_factor()?;
  let parent_position = parent.outer_position()?.to_logical::<f64>(scale);
  let mut position =
    LogicalPosition::new(parent_position.x + offset.x, parent_position.y + offset.y);

  if let Some(monitor) = parent.current_monitor()?.or(app.primary_monitor()?) {
    let monitor_scale = monitor.scale_factor();
    let monitor_position = monitor.position().to_logical::<f64>(monitor_scale);
    let monitor_size = monitor.size().to_logical::<f64>(monitor_scale);
    let max_x = monitor_position.x + (monitor_size.width - size.width).max(0.0);
    let max_y = monitor_position.y + (monitor_size.height - size.height).max(0.0);
    position.x = position.x.clamp(monitor_position.x, max_x);
    position.y = position.y.clamp(monitor_position.y, max_y);
  }

  platform::set_frame(&window, position, size)?;
  platform::show(&window, 1.0)?;
  platform::restore_recording_level(&window)?;
  if focus_contents {
    if let Err(error) = window.set_focus() {
      let _ = platform::hide(&window);
      return Err(error);
    }
  }
  *standalone_listbox_context() = Some(StandaloneListboxContext {
    focus_contents,
    parent_window_label,
    trigger_id,
  });
  STANDALONE_LISTBOX.set_open(true);
  Ok(())
}

pub(super) fn close_standalone_listbox(app: AppHandle, return_focus: bool) -> tauri::Result<()> {
  let _lifecycle = STANDALONE_LISTBOX.lock();
  if !STANDALONE_LISTBOX.is_open() {
    return Ok(());
  }
  let context = standalone_listbox_context().clone();
  if let Some(window) = app.get_webview_window(WindowLabel::StandaloneListbox.as_str()) {
    platform::hide(&window)?;
  }
  STANDALONE_LISTBOX.set_open(false);
  *standalone_listbox_context() = None;
  let should_return_focus = return_focus
    && context
      .as_ref()
      .is_some_and(|context| context.focus_contents);
  let focus_result = if should_return_focus {
    if let Some(parent) = context
      .as_ref()
      .and_then(|context| app.get_webview_window(&context.parent_window_label))
    {
      parent.set_focus()
    } else {
      Ok(())
    }
  } else {
    Ok(())
  };
  app.emit(
    "standalone-listbox://closed",
    StandaloneListboxClosed {
      return_focus: should_return_focus,
      trigger_id: context.map_or_else(String::new, |context| context.trigger_id),
    },
  )?;
  focus_result
}

#[tauri::command]
pub fn hide_standalone_listbox(app: AppHandle, return_focus: Option<bool>) -> tauri::Result<()> {
  close_standalone_listbox(app, return_focus.unwrap_or(false))
}

pub(super) fn is_recording_options_open() -> bool {
  RECORDING_OPTIONS.is_open()
}

pub(super) fn is_standalone_listbox_open() -> bool {
  STANDALONE_LISTBOX.is_open()
}

pub(super) fn dismiss_recording_options_if_outside(
  app: &AppHandle,
  open_on_press: bool,
  x: f64,
  y: f64,
) {
  let inside_anchor = recording_options_layout::anchor_contains(app, x, y);
  if RECORDING_OPTIONS.should_dismiss(
    app,
    open_on_press,
    inside_anchor,
    x,
    y,
    &[
      WindowLabel::RecordingOptions,
      WindowLabel::StandaloneListbox,
    ],
  ) {
    let _ = close_recording_options(app.clone(), false);
  }
}
