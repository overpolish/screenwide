// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use std::collections::BTreeSet;

pub(in crate::app_windows) fn standalone_listbox_contexts(
) -> MutexGuard<'static, BTreeMap<String, StandaloneListboxContext>> {
  STANDALONE_LISTBOX_CONTEXTS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The window a command means. The shared listbox is the default, so every
/// pop-up button keeps calling exactly as it did.
pub(in crate::app_windows) fn panel_label(panel: Option<String>) -> String {
  panel.unwrap_or_else(|| WindowLabel::StandaloneListbox.as_str().to_owned())
}

pub(in crate::app_windows) fn context_for(panel: &str) -> Option<StandaloneListboxContext> {
  standalone_listbox_contexts()
    .get(panel)
    .filter(|context| context.open)
    .cloned()
}

/// The panel windows showing something, in a stable order.
pub(in crate::app_windows) fn open_panel_labels() -> Vec<String> {
  standalone_listbox_contexts()
    .iter()
    .filter(|(_, context)| context.open)
    .map(|(label, _)| label.clone())
    .collect()
}

/// Whether any panel window is open. Escape and the outside-press watcher ask
/// this before doing any work at all.
pub(in crate::app_windows) fn is_standalone_listbox_open() -> bool {
  STANDALONE_LISTBOX.is_open()
}

/// Keeps the shared "anything open" flag in step with the per-window entries.
pub(in crate::app_windows) fn synchronize_open_flag() {
  let any_open = standalone_listbox_contexts()
    .values()
    .any(|context| context.open);
  STANDALONE_LISTBOX.set_open(any_open);
}

/// The windows that have opened a menu in the shared listbox. It is kept while
/// any of them is on screen, and let go once none is (`on_demand`).
static SHARED_LISTBOX_OPENERS: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

pub(super) fn note_shared_listbox_opener(parent: &str) {
  SHARED_LISTBOX_OPENERS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .insert(parent.to_owned());
}

/// Whether the shared listbox is wanted though it may be hidden: it is
/// showing something, or a window that opens menus in it is on screen.
pub(crate) fn shared_listbox_needed(app: &AppHandle) -> bool {
  if context_for(WindowLabel::StandaloneListbox.as_str()).is_some() {
    return true;
  }
  // Copied out before asking each window, which waits on the main thread:
  // the main thread may itself be waiting for this lock to note an opener.
  let openers: Vec<String> = {
    let mut openers = SHARED_LISTBOX_OPENERS
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    openers.retain(|label| app.get_webview_window(label).is_some());
    openers.iter().cloned().collect()
  };
  openers.iter().any(|label| {
    app
      .get_webview_window(label)
      .is_some_and(|window| window.is_visible().unwrap_or(false))
  })
}
