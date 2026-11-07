// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The panel windows opened from another window: the shared listbox every
//! pop-up button borrows, and one tool panel per editor workspace.
//!
//! Every command names the panel window it means, so two editors can each
//! have their own tool panel up at once. A caller that names none gets the
//! shared listbox, which is what a pop-up button has always opened.

mod registry;
pub(super) use registry::context_for;
pub(super) use registry::is_standalone_listbox_open;
use registry::note_shared_listbox_opener;
pub(super) use registry::open_panel_labels;
pub(super) use registry::panel_label;
pub(crate) use registry::shared_listbox_needed;
pub(super) use registry::standalone_listbox_contexts;
pub(super) use registry::synchronize_open_flag;
mod pending;
use pending::{next_panel_request, show_once_built};

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager};

use super::{platform, transient_popover::TransientPopover, WindowLabel};

mod attachment;
pub(crate) mod dismissal;
pub(crate) mod hit_area;
pub(crate) mod lifecycle;
pub(crate) mod placement;
// The tooltip is put on screen concealed and revealed once it is fitted, and
// a sticky panel hangs off its parent as a native child.
pub(crate) mod presentation;
use attachment::{attach_to_parent, detach_from_parent};
#[cfg(any(test, target_os = "windows"))]
mod placement_geometry;
#[cfg(target_os = "windows")]
pub(crate) mod placement_windows;

pub(super) use dismissal::dismiss_standalone_listbox_if_outside;
pub(crate) use lifecycle::{
  close_all_standalone_listboxes, close_standalone_listbox, close_standalone_listbox_for_parent,
  close_standalone_listboxes_of,
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
  #[cfg(target_os = "windows")]
  offset: LogicalPosition<f64>,
  /// The trigger's bounds in logical px, relative to the parent window's
  /// content, the way `offset` is expressed. A press inside it belongs to the
  /// trigger, which toggles the panel on mouse-up, so an outside press must
  /// not dismiss it first.
  anchor: Option<AnchorRect>,
  focus_contents: bool,
  /// The parent page's height, which says where the page begins in its
  /// window, for placing the anchor on screen.
  page_height: Option<f64>,
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
  page_height: Option<f64>,
) -> tauri::Result<()> {
  let panel = panel_label(panel);
  if app.get_webview_window(&panel).is_none() {
    show_once_built(&app, &panel, move |app, panel| {
      show_standalone_listbox(
        app,
        focus_contents,
        parent_window_label,
        trigger_id,
        offset,
        size,
        anchor,
        sticky,
        Some(panel),
        page_height,
      )
    });
    return Ok(());
  }
  if panel == WindowLabel::StandaloneListbox.as_str() {
    note_shared_listbox_opener(&parent_window_label);
  }
  let _lifecycle = STANDALONE_LISTBOX.lock();
  let parent = app
    .get_webview_window(&parent_window_label)
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let window = app
    .get_webview_window(&panel)
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  #[cfg(not(target_os = "windows"))]
  let position = {
    // The offset is relative to where the page begins, below any title bar.
    let parent_position = super::geometry::page_origin(&parent, page_height)?;
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

    position
  };

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

  #[cfg(not(target_os = "windows"))]
  platform::set_frame(&window, position, size)?;
  #[cfg(target_os = "windows")]
  {
    let placement_policy = if sticky {
      placement_windows::PlacementPolicy::PreserveAnchor
    } else {
      placement_windows::PlacementPolicy::WorkArea
    };
    placement_windows::place(&app, &parent, &window, offset, Some(size), placement_policy)?;
  }
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
  standalone_listbox_contexts().insert(
    panel,
    StandaloneListboxContext {
      anchor,
      attached_to,
      #[cfg(target_os = "windows")]
      offset,
      focus_contents,
      page_height,
      open: true,
      parent_window_label,
      sticky,
      trigger_id,
    },
  );
  synchronize_open_flag();
  Ok(())
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
