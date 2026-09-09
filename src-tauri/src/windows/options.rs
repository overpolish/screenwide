// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager};

use super::{platform, transient_popover::TransientPopover, WindowLabel};

static STANDALONE_LISTBOX: TransientPopover = TransientPopover::new();
static STANDALONE_LISTBOX_CONTEXT: Mutex<Option<StandaloneListboxContext>> = Mutex::new(None);

#[derive(Clone)]
struct StandaloneListboxContext {
  /// The trigger's bounds in logical px, relative to the parent window's
  /// content, the way `offset` is expressed. A press inside it belongs to the
  /// trigger, which toggles the panel on mouse-up, so an outside press must
  /// not dismiss it first.
  anchor: Option<AnchorRect>,
  focus_contents: bool,
  parent_window_label: String,
  trigger_id: String,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorRect {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StandaloneListboxClosed {
  return_focus: bool,
  trigger_id: String,
}

fn standalone_listbox_context() -> MutexGuard<'static, Option<StandaloneListboxContext>> {
  STANDALONE_LISTBOX_CONTEXT
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
pub fn show_standalone_listbox(
  app: AppHandle,
  focus_contents: bool,
  parent_window_label: String,
  trigger_id: String,
  offset: LogicalPosition<f64>,
  size: LogicalSize<f64>,
  anchor: Option<AnchorRect>,
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
    anchor,
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

pub(super) fn is_standalone_listbox_open() -> bool {
  STANDALONE_LISTBOX.is_open()
}

/// Hit-tests the stored trigger bounds for a listbox whose anchor travels
/// with whichever window opened it.
fn standalone_listbox_anchor_contains(app: &AppHandle, x: f64, y: f64) -> bool {
  let Some(context) = standalone_listbox_context().clone() else {
    return false;
  };
  let Some(anchor) = context.anchor else {
    return false;
  };
  let Some(parent) = app.get_webview_window(&context.parent_window_label) else {
    return false;
  };
  let Ok(position) = parent.outer_position() else {
    return false;
  };
  let Ok(scale) = parent.scale_factor() else {
    return false;
  };
  let position = position.to_logical::<f64>(scale);
  let left = position.x + anchor.x;
  let top = position.y + anchor.y;

  x >= left && x <= left + anchor.width && y >= top && y <= top + anchor.height
}

pub(super) fn dismiss_standalone_listbox_if_outside(
  app: &AppHandle,
  open_on_press: bool,
  x: f64,
  y: f64,
) {
  let inside_anchor = standalone_listbox_anchor_contains(app, x, y);
  if STANDALONE_LISTBOX.should_dismiss(
    app,
    open_on_press,
    inside_anchor,
    x,
    y,
    &[WindowLabel::StandaloneListbox],
  ) {
    let _ = close_standalone_listbox(app.clone(), false);
  }
}
