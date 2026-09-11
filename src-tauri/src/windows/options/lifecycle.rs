// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Putting a panel window away, one named window at a time.

use tauri::{AppHandle, Emitter, Manager};

use super::{
  context_for, detach_from_parent, open_panel_labels, platform, standalone_listbox_contexts,
  synchronize_open_flag, StandaloneListboxClosed, STANDALONE_LISTBOX,
};

/// Puts one panel window away. Every other panel window is left exactly as it
/// was: each editor's tool panel belongs to its own workspace.
pub(crate) fn close_standalone_listbox(
  app: AppHandle,
  return_focus: bool,
  panel: &str,
) -> tauri::Result<()> {
  let _lifecycle = STANDALONE_LISTBOX.lock();
  close_locked(&app, return_focus, panel)
}

/// Puts every open panel away: a display change, the recording UI going down,
/// and Escape all mean whatever is up, wherever it is.
pub(crate) fn close_all_standalone_listboxes(
  app: AppHandle,
  return_focus: bool,
) -> tauri::Result<()> {
  let _lifecycle = STANDALONE_LISTBOX.lock();
  let mut result = Ok(());
  for panel in open_panel_labels() {
    let outcome = close_locked(&app, return_focus, &panel);
    if result.is_ok() {
      result = outcome;
    }
  }
  result
}

/// Puts away the panels that hang off a window being hidden, minimised or
/// closed. A sticky panel outlives an outside press, so nothing else would
/// take it with its parent.
pub fn close_standalone_listbox_for_parent(app: &AppHandle, parent_window_label: &str) {
  let belonging: Vec<String> = open_panel_labels()
    .into_iter()
    .filter(|panel| {
      context_for(panel).is_some_and(|context| context.parent_window_label == parent_window_label)
    })
    .collect();
  for panel in belonging {
    let _ = close_standalone_listbox(app.clone(), false, &panel);
  }
}

fn close_locked(app: &AppHandle, return_focus: bool, panel: &str) -> tauri::Result<()> {
  let Some(context) = context_for(panel) else {
    return Ok(());
  };
  if let Some(window) = app.get_webview_window(panel) {
    let attached_parent = context
      .attached_to
      .as_ref()
      .and_then(|label| app.get_webview_window(label));
    if let Some(parent) = attached_parent.as_ref() {
      let _ = detach_from_parent(app, parent, &window);
    }
    platform::hide(&window)?;
    if attached_parent.is_some() {
      // The next panel to open in this window may well be a menu, which is
      // expected to float over everything again.
      let _ = platform::restore_recording_level(&window);
    }
  }
  if let Some(entry) = standalone_listbox_contexts().get_mut(panel) {
    entry.open = false;
    // Nothing is attached or anchored once the panel is off screen, and a
    // stale attachment would be detached a second time on the next open.
    entry.anchor = None;
    entry.attached_to = None;
  }
  synchronize_open_flag();
  let should_return_focus = return_focus && context.focus_contents;
  let focus_result = if should_return_focus {
    match app.get_webview_window(&context.parent_window_label) {
      Some(parent) => parent.set_focus(),
      None => Ok(()),
    }
  } else {
    Ok(())
  };
  app.emit(
    "standalone-listbox://closed",
    StandaloneListboxClosed {
      panel: panel.to_owned(),
      return_focus: should_return_focus,
      trigger_id: context.trigger_id,
    },
  )?;
  focus_result
}
