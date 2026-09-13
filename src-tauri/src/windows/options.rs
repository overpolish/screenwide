// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The panel windows opened from another window: the shared listbox every
//! pop-up button borrows, and one tool panel per editor workspace.
//!
//! Every command names the panel window it means, so two editors can each
//! have their own tool panel up at once. A caller that names none gets the
//! shared listbox, which is what a pop-up button has always opened.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager};

use super::{platform, transient_popover::TransientPopover, WindowLabel};

pub(crate) mod dismissal;
pub(crate) mod lifecycle;
pub(crate) mod placement;

pub(super) use dismissal::dismiss_standalone_listbox_if_outside;
pub(crate) use lifecycle::{
  close_all_standalone_listboxes, close_standalone_listbox, close_standalone_listbox_for_parent,
};

/// Serializes every open and close across all the panel windows, so a show
/// racing a hide cannot leave one attached to a window that is going away.
static STANDALONE_LISTBOX: TransientPopover = TransientPopover::new();
static STANDALONE_LISTBOX_CONTEXTS: Mutex<BTreeMap<String, StandaloneListboxContext>> =
  Mutex::new(BTreeMap::new());

#[derive(Clone)]
pub(super) struct StandaloneListboxContext {
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
  /// Whether this window is showing a panel. An entry outlives its panel so
  /// that closing one window's panel says nothing about the others.
  open: bool,
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
  /// Which panel window closed, so a window listening for its own panel can
  /// ignore another's.
  panel: String,
  return_focus: bool,
  trigger_id: String,
}

pub(super) fn standalone_listbox_contexts(
) -> MutexGuard<'static, BTreeMap<String, StandaloneListboxContext>> {
  STANDALONE_LISTBOX_CONTEXTS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The window a command means. The shared listbox is the default, so every
/// pop-up button keeps calling exactly as it did.
pub(super) fn panel_label(panel: Option<String>) -> String {
  panel.unwrap_or_else(|| WindowLabel::StandaloneListbox.as_str().to_owned())
}

pub(super) fn context_for(panel: &str) -> Option<StandaloneListboxContext> {
  standalone_listbox_contexts()
    .get(panel)
    .filter(|context| context.open)
    .cloned()
}

/// The panel windows showing something, in a stable order.
pub(super) fn open_panel_labels() -> Vec<String> {
  standalone_listbox_contexts()
    .iter()
    .filter(|(_, context)| context.open)
    .map(|(label, _)| label.clone())
    .collect()
}

/// Whether any panel window is open. Escape and the outside-press watcher ask
/// this before doing any work at all.
pub(super) fn is_standalone_listbox_open() -> bool {
  STANDALONE_LISTBOX.is_open()
}

/// Keeps the shared "anything open" flag in step with the per-window entries.
pub(super) fn synchronize_open_flag() {
  let any_open = standalone_listbox_contexts()
    .values()
    .any(|context| context.open);
  STANDALONE_LISTBOX.set_open(any_open);
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
pub(super) fn detach_from_parent(
  app: &AppHandle,
  parent: &tauri::WebviewWindow,
  panel: &tauri::WebviewWindow,
) -> tauri::Result<()> {
  crate::editor::export_window::presentation::detach(app, parent, panel)
}

#[cfg(not(target_os = "macos"))]
pub(super) fn detach_from_parent(
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
  panel: Option<String>,
  fitted: Option<bool>,
) -> tauri::Result<()> {
  let panel = panel_label(panel);
  let _lifecycle = STANDALONE_LISTBOX.lock();
  let parent = app
    .get_webview_window(&parent_window_label)
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let window = app
    .get_webview_window(&panel)
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

  // A panel window is reused, so one still attached to another window has to
  // be let go before this one is placed: left attached it would follow that
  // window, and ordering it out would drag it along.
  let sticky = sticky.unwrap_or(false);
  let previously_attached = standalone_listbox_contexts()
    .get(&panel)
    .and_then(|context| context.attached_to.clone())
    .filter(|label| !sticky || label != &parent_window_label);
  if let Some(previous) = previously_attached.and_then(|label| app.get_webview_window(&label)) {
    let _ = detach_from_parent(&app, &previous, &window);
  }

  platform::set_frame(&window, position, size)?;
  // A panel that fits itself to its contents opens unseen at the size it was
  // asked for, lays out, and is revealed by `fit_standalone_listbox` once it
  // is the size of what it holds: showing it at one height and settling at
  // another reads as a flicker. One already on screen keeps its pixels: a
  // swap of contents resizes in place rather than blinking out. Concealed
  // before attachment, because AppKit can order a child on screen as it is
  // attached.
  let conceal = fitted.unwrap_or(false) && !window.is_visible().unwrap_or(false);
  if conceal {
    crate::editor::export_window::presentation::conceal(&app, &window)?;
  }
  let attached_to = if sticky {
    attach_to_parent(&app, &parent, &window)?;
    Some(parent_window_label.clone())
  } else {
    None
  };
  platform::show(&window, if conceal { 0.0 } else { 1.0 })?;
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
  standalone_listbox_contexts().insert(
    panel,
    StandaloneListboxContext {
      anchor,
      attached_to,
      focus_contents,
      open: true,
      parent_window_label,
      sticky,
      trigger_id,
    },
  );
  synchronize_open_flag();
  Ok(())
}

/// A fitted panel's content reporting the height it needs: the window is
/// sized to it and revealed once that resize has landed.
#[tauri::command]
pub fn fit_standalone_listbox(
  app: AppHandle,
  panel: Option<String>,
  height: f64,
) -> tauri::Result<()> {
  if !height.is_finite() || height <= 0.0 {
    return Ok(());
  }
  let window = app
    .get_webview_window(&panel_label(panel))
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let scale = window.scale_factor()?;
  let width = window.inner_size()?.to_logical::<f64>(scale).width;
  window.set_size(LogicalSize::new(width, height.ceil()))?;
  crate::editor::export_window::presentation::reveal_after_resize(&window)
}

/// The panel windows showing something right now, by label. A webview that
/// has just come up asks this before it trusts the open-panel map it inherits
/// from storage: what Rust has on screen is the truth, and anything else in
/// the map is left over from a run that ended.
#[tauri::command]
pub fn open_standalone_listboxes() -> Vec<String> {
  open_panel_labels()
}

#[tauri::command]
pub fn hide_standalone_listbox(
  app: AppHandle,
  return_focus: Option<bool>,
  panel: Option<String>,
) -> tauri::Result<()> {
  close_standalone_listbox(app, return_focus.unwrap_or(false), &panel_label(panel))
}

#[cfg(test)]
mod tests {
  use super::{panel_label, WindowLabel};

  #[test]
  fn a_command_without_a_panel_means_the_shared_listbox() {
    assert_eq!(panel_label(None), WindowLabel::StandaloneListbox.as_str());
    assert_eq!(
      panel_label(Some("tool-panel-screenshot".to_owned())),
      "tool-panel-screenshot"
    );
  }
}
