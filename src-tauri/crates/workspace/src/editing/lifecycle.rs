// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl WorkspaceEditor {
  pub fn hide(&self) {
    if let Ok(mut state) = self.inner.state.lock() {
      // Surfaces live for the editor window, not for one preview session, so the scene lets go of every picture the session staged:
      // reusable screenshot item IDs must not find the last session's.
      clear_workspace(&self.inner, &mut state);
      self.inner.host.release();
      for pane in state.panes.iter_mut().flatten() {
        pane.source_token = None;
        pane.settings = None;
        pane.captions = None;
      }
      state.editor_active = false;
      state.selection = None;
      state.selection_targets.clear();
      state.selection_hit_testing = false;
      state.gesture = None;
      // A hidden surface has no drag to finish; never keep the DOM locked out
      // of the pane geometry behind it.
      state.frame_resize = None;
      state.frame_resize_committed = false;
      state.move_auto_fit = None;
      self.inner.host.set_active(false);
      draw_selection(&self.inner, &state);
      self.inner.host.commit(false);
    }
  }
}
