// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a press outside a panel means, per panel window.

use std::collections::BTreeMap;

use tauri::{AppHandle, Manager};

use super::super::transient_popover::coordinate_is_in_visible_window;
use super::{
  close_standalone_listbox, context_for, standalone_listbox_contexts, StandaloneListboxContext,
};

/// Whether the open panel is one the user works alongside, which an outside
/// press must leave alone.
fn is_sticky(context: Option<&StandaloneListboxContext>) -> bool {
  context.is_some_and(|context| context.sticky)
}

/// The panel windows a press outside them would put away: the ones that are
/// open and are not a panel the user works alongside. A sticky tool panel
/// stays up, and so does every panel belonging to another window.
fn dismissible_panels(contexts: &BTreeMap<String, StandaloneListboxContext>) -> Vec<String> {
  contexts
    .iter()
    .filter(|(_, context)| context.open && !is_sticky(Some(context)))
    .map(|(label, _)| label.clone())
    .collect()
}

/// Hit-tests the stored trigger bounds for a panel whose anchor travels with
/// whichever window opened it.
fn anchor_contains(app: &AppHandle, context: &StandaloneListboxContext, x: f64, y: f64) -> bool {
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

pub(in crate::windows) fn dismiss_standalone_listbox_if_outside(
  app: &AppHandle,
  open_on_press: bool,
  x: f64,
  y: f64,
) {
  if !open_on_press {
    return;
  }
  let dismissible = dismissible_panels(&standalone_listbox_contexts());
  for panel in dismissible {
    let Some(context) = context_for(&panel) else {
      continue;
    };
    if anchor_contains(app, &context, x, y) {
      continue;
    }
    let inside_panel = app
      .get_webview_window(&panel)
      .is_some_and(|window| coordinate_is_in_visible_window(x, y, &window));
    if inside_panel {
      continue;
    }
    let _ = close_standalone_listbox(app.clone(), false, &panel);
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use super::{dismissible_panels, is_sticky, StandaloneListboxContext};

  fn context(parent: &str, open: bool, sticky: bool) -> StandaloneListboxContext {
    StandaloneListboxContext {
      anchor: None,
      attached_to: None,
      focus_contents: false,
      open,
      parent_window_label: parent.to_owned(),
      sticky,
      trigger_id: "tool:cursor".to_owned(),
    }
  }

  fn contexts(
    entries: [(&str, StandaloneListboxContext); 3],
  ) -> BTreeMap<String, StandaloneListboxContext> {
    entries
      .into_iter()
      .map(|(label, context)| (label.to_owned(), context))
      .collect()
  }

  #[test]
  fn only_a_sticky_panel_survives_an_outside_press() {
    assert!(!is_sticky(None));
    assert!(!is_sticky(Some(&context("editor-recording", true, false))));
    assert!(is_sticky(Some(&context("editor-recording", true, true))));
  }

  #[test]
  fn an_outside_press_leaves_each_editors_tool_panel_alone() {
    let contexts = contexts([
      ("standalone-listbox", context("recording-bar", true, false)),
      (
        "tool-panel-recording",
        context("editor-recording", true, true),
      ),
      (
        "tool-panel-screenshot",
        context("editor-screenshot", true, true),
      ),
    ]);

    assert_eq!(dismissible_panels(&contexts), vec!["standalone-listbox"]);
  }

  #[test]
  fn a_panel_already_put_away_is_not_dismissed_again() {
    let contexts = contexts([
      ("standalone-listbox", context("recording-bar", false, false)),
      (
        "tool-panel-recording",
        context("editor-recording", true, false),
      ),
      (
        "tool-panel-screenshot",
        context("editor-screenshot", false, false),
      ),
    ]);

    assert_eq!(dismissible_panels(&contexts), vec!["tool-panel-recording"]);
  }
}
