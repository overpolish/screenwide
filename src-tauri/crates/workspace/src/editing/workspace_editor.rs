// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The engine as a platform surface holds it: one per editor window, fed the
//! window's pointer and typing input.

use super::*;

pub struct WorkspaceEditor {
  pub inner: Arc<SurfaceInner>,
}

impl WorkspaceEditor {
  pub fn new(host: Box<dyn Host>) -> Self {
    Self {
      inner: SurfaceInner::new(host, SurfaceState::default()),
    }
  }

  /// One pointer event over the workspace.
  pub fn handle_pointer(&self, input: PointerInput) {
    handle_editor_input(&self.inner, input);
  }

  /// The pointer came onto the picture from the page around it.
  pub fn pointer_arrived(&self) {
    emit_press(&self.inner, PointerPress::Arrived);
  }

  /// One key, typed unit or blink for the box being typed into.
  pub fn handle_typing(&self, input: TypingInput) {
    handle_typing_input(&self.inner, input);
  }

  /// Shows the pointer for wherever it rests now, as a platform asks when
  /// the pointer enters or the window wants its cursor set.
  pub fn refresh_cursor(&self) {
    refresh_cursor_for(&self.inner);
  }

  /// Draws the picture and its chrome again as they stand, on the calling
  /// thread: a platform that held back a draw from another thread, as macOS
  /// does while a window is live-resized, asks for it here instead.
  pub fn redraw(&self) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    draw_selection(&self.inner, &state);
    draw_workspace(&self.inner, &mut state);
    if self.inner.batch_depth.load(Ordering::Acquire) == 0 {
      self.inner.host.commit(false);
    }
  }
}

impl Clone for WorkspaceEditor {
  fn clone(&self) -> Self {
    Self {
      inner: Arc::clone(&self.inner),
    }
  }
}
