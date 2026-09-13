// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::windows) fn standalone_listbox_contexts(
) -> MutexGuard<'static, BTreeMap<String, StandaloneListboxContext>> {
  STANDALONE_LISTBOX_CONTEXTS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The window a command means. The shared listbox is the default, so every
/// pop-up button keeps calling exactly as it did.
pub(in crate::windows) fn panel_label(panel: Option<String>) -> String {
  panel.unwrap_or_else(|| WindowLabel::StandaloneListbox.as_str().to_owned())
}

pub(in crate::windows) fn context_for(panel: &str) -> Option<StandaloneListboxContext> {
  standalone_listbox_contexts()
    .get(panel)
    .filter(|context| context.open)
    .cloned()
}

/// The panel windows showing something, in a stable order.
pub(in crate::windows) fn open_panel_labels() -> Vec<String> {
  standalone_listbox_contexts()
    .iter()
    .filter(|(_, context)| context.open)
    .map(|(label, _)| label.clone())
    .collect()
}

/// Whether any panel window is open. Escape and the outside-press watcher ask
/// this before doing any work at all.
pub(in crate::windows) fn is_standalone_listbox_open() -> bool {
  STANDALONE_LISTBOX.is_open()
}

/// Keeps the shared "anything open" flag in step with the per-window entries.
pub(in crate::windows) fn synchronize_open_flag() {
  let any_open = standalone_listbox_contexts()
    .values()
    .any(|context| context.open);
  STANDALONE_LISTBOX.set_open(any_open);
}
