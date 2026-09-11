// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager};

use super::{platform, transient_popover::TransientPopover, WindowLabel};

pub(crate) mod placement;

static STANDALONE_LISTBOX: TransientPopover = TransientPopover::new();
static STANDALONE_LISTBOX_CONTEXT: Mutex<Option<StandaloneListboxContext>> = Mutex::new(None);

#[derive(Clone)]
struct StandaloneListboxContext {
  /// The window this panel is a native child of, when it is one the user
  /// works alongside. A menu is attached to nothing and floats over
  /// everything; a sticky panel belongs with the window it was opened from,
  /// and has to be detached again before it is ordered out.
  attached_to: Option<String>,
  /// The trigger's bounds in logical px, relative to the parent window's
  /// content, the way `offset` is expressed. A press inside it belongs to the
  /// trigger, which toggles the panel on mouse-up, so an outside press must
  /// not dismiss it first.
  anchor: Option<AnchorRect>,
  focus_contents: bool,
  parent_window_label: String,
  /// A panel the user works alongside rather than a menu they answer: an
  /// outside press leaves it alone. It still closes on Escape, from its own
  /// trigger, and with the window it hangs off.
  sticky: bool,
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

/// A sticky panel stands with the window it belongs to instead of floating
/// over every other application: it drops to the ordinary window level and
/// becomes a native child, the way the confirm sheet does.
#[cfg(target_os = "macos")]
fn attach_to_parent(
  app: &AppHandle,
  parent: &tauri::WebviewWindow,
  panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  platform::set_normal_level(panel)?;
  crate::editor::export_window::presentation::attach(app, parent, panel)
}

/// Windows keeps the floating panel for now: `presentation::attach` there
/// disables the parent window, which is right for a modal sheet and wrong for
/// a panel the user works alongside. Owning it without disabling the editor
/// belongs to the Windows pass.
#[cfg(not(target_os = "macos"))]
fn attach_to_parent(
  _app: &AppHandle,
  _parent: &tauri::WebviewWindow,
  _panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  Ok(())
}

/// Ordering a still-attached child out drags its parent with it, so this runs
/// before the panel is hidden.
#[cfg(target_os = "macos")]
fn detach_from_parent(
  app: &AppHandle,
  parent: &tauri::WebviewWindow,
  panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  crate::editor::export_window::presentation::detach(app, parent, panel)
}

#[cfg(not(target_os = "macos"))]
fn detach_from_parent(
  _app: &AppHandle,
  _parent: &tauri::WebviewWindow,
  _panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  Ok(())
}

#[tauri::command]
#[expect(
  clippy::too_many_arguments,
  reason = "Tauri exposes this function as a flat, named IPC command"
)]
pub fn show_standalone_listbox(
  app: AppHandle,
  focus_contents: bool,
  parent_window_label: String,
  trigger_id: String,
  offset: LogicalPosition<f64>,
  size: LogicalSize<f64>,
  anchor: Option<AnchorRect>,
  sticky: Option<bool>,
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

  // The one panel window is reused, so a panel still attached to another
  // window has to be let go before this one is placed: left attached it would
  // follow that window, and ordering it out would drag it along.
  let sticky = sticky.unwrap_or(false);
  let previously_attached = standalone_listbox_context()
    .as_ref()
    .and_then(|context| context.attached_to.clone())
    .filter(|label| !sticky || label != &parent_window_label);
  if let Some(previous) = previously_attached.and_then(|label| app.get_webview_window(&label)) {
    let _ = detach_from_parent(&app, &previous, &window);
  }

  platform::set_frame(&window, position, size)?;
  let attached_to = if sticky {
    attach_to_parent(&app, &parent, &window)?;
    Some(parent_window_label.clone())
  } else {
    None
  };
  platform::show(&window, 1.0)?;
  // A menu floats over every application; an attached panel keeps the
  // ordinary level it was just given, so it travels with its parent.
  if attached_to.is_none() {
    platform::restore_recording_level(&window)?;
  }
  if focus_contents {
    if let Err(error) = window.set_focus() {
      let _ = platform::hide(&window);
      return Err(error);
    }
  }
  *standalone_listbox_context() = Some(StandaloneListboxContext {
    anchor,
    attached_to,
    focus_contents,
    parent_window_label,
    sticky,
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
    let attached_parent = context
      .as_ref()
      .and_then(|context| context.attached_to.as_ref())
      .and_then(|label| app.get_webview_window(label));
    if let Some(parent) = attached_parent.as_ref() {
      let _ = detach_from_parent(&app, parent, &window);
    }
    platform::hide(&window)?;
    if attached_parent.is_some() {
      // The next panel to open in this window may well be a menu, which is
      // expected to float over everything again.
      let _ = platform::restore_recording_level(&window);
    }
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

/// Puts away a panel that hangs off a window being hidden, minimised or
/// closed. A sticky panel outlives an outside press, so nothing else would
/// take it with its parent.
pub fn close_standalone_listbox_for_parent(app: &AppHandle, parent_window_label: &str) {
  let belongs_to_parent = standalone_listbox_context()
    .as_ref()
    .is_some_and(|context| context.parent_window_label == parent_window_label);
  if belongs_to_parent {
    let _ = close_standalone_listbox(app.clone(), false);
  }
}

#[tauri::command]
pub fn hide_standalone_listbox(app: AppHandle, return_focus: Option<bool>) -> tauri::Result<()> {
  close_standalone_listbox(app, return_focus.unwrap_or(false))
}

pub(super) fn is_standalone_listbox_open() -> bool {
  STANDALONE_LISTBOX.is_open()
}

/// Whether the open panel is one the user works alongside, which an outside
/// press must leave alone.
fn is_sticky(context: Option<&StandaloneListboxContext>) -> bool {
  context.is_some_and(|context| context.sticky)
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
  if is_sticky(standalone_listbox_context().as_ref()) {
    return;
  }
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

#[cfg(test)]
mod tests {
  use super::{is_sticky, StandaloneListboxContext};

  fn context(sticky: bool) -> StandaloneListboxContext {
    StandaloneListboxContext {
      anchor: None,
      attached_to: None,
      focus_contents: false,
      parent_window_label: "editor-recording".to_owned(),
      sticky,
      trigger_id: "tool:cursor".to_owned(),
    }
  }

  #[test]
  fn only_a_sticky_panel_survives_an_outside_press() {
    assert!(!is_sticky(None));
    assert!(!is_sticky(Some(&context(false))));
    assert!(is_sticky(Some(&context(true))));
  }
}
