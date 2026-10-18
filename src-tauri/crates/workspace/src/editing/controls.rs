// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl WorkspaceEditor {
  pub fn set_scale(&self, scale: f64) {
    if let Ok(mut state) = self.inner.state.lock() {
      let scale = scale.max(0.1);
      let changed = state.scale != scale;
      state.scale = scale;
      place_viewport(&self.inner, &state, state.editor_active);
      draw_selection(&self.inner, &state);
      if changed {
        draw_workspace(&self.inner, &mut state);
      }
    }
  }

  pub fn enable_editor(&self, callback: TransformCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.transform = Some(callback);
    }
    if let Ok(mut state) = self.inner.state.lock() {
      state.editor_active = true;
      state.frame_resize = None;
      state.frame_resize_committed = false;
      state.move_auto_fit = None;
      state.panel_fit_width = 0.0;
      state.workspace_transform = WorkspaceTransform::default();
      state.workspace_natural_size = None;
      state.workspace_transforms.clear();
      place_viewport(&self.inner, &state, true);
      // The reset transform is 100%, so every pane returns to composing at
      // its on-screen size.
      draw_selection(&self.inner, &state);
    }
  }

  pub fn set_editor_active(&self, active: bool) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.editor_active = active;
      self.inner.host.set_active(active);
      draw_selection(&self.inner, &state);
    }
  }

  pub fn set_editor_zoom(&self, zoom_percent: f64) {
    let mut changed_zoom = None;
    if let Ok(mut state) = self.inner.state.lock() {
      let zoom = (zoom_percent / 100.0).clamp(0.1, maximum_editor_zoom(&state));
      if (state.workspace_transform.zoom - zoom).abs() > 0.0001 {
        let ratio = zoom / state.workspace_transform.zoom;
        state.workspace_transform.pan_x *= ratio;
        state.workspace_transform.pan_y *= ratio;
        state.workspace_transform.zoom = zoom;
        apply_workspace_transform(&self.inner, &mut state, false);
        changed_zoom = Some(zoom);
      }
    }
    if let Some(zoom) = changed_zoom {
      emit_transform(&self.inner, zoom);
    }
  }

  pub fn set_selection_callback(&self, callback: SelectionCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.selection = Some(callback);
    }
  }

  pub fn set_pointer_down_callback(&self, callback: PointerDownCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.pointer_down = Some(callback);
    }
  }

  pub fn set_context_menu_callback(&self, callback: ContextMenuCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.context_menu = Some(callback);
    }
  }

  pub fn set_selection_gesture_callback(&self, callback: SelectionGestureCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.gesture = Some(callback);
    }
  }

  pub fn set_selection_snapping(&self, enabled: bool) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.selection_snapping_enabled = enabled;
      if !enabled {
        clear_selection_snap_guides(&mut state);
        draw_selection(&self.inner, &state);
      }
    }
  }

  pub fn set_selection(&self, selection: Option<PreviewSelection>) {
    if let Ok(mut state) = self.inner.state.lock() {
      // A drag keeps the selection it moves, resizes or turns, as the macOS
      // surface's `selection_held_by_drag` does: a layout laid out from an
      // earlier sample would pull the chrome back until the next one. A
      // layout that takes the selection away lets a held ring go at the next
      // pointer sample, without keeping the turn.
      let dragging =
        state.turn.drag.is_some() || matches!(state.gesture, Some(ActiveGesture::Selection(_)));
      if dragging && selection.is_some() {
        return;
      }
      state.selection = selection;
      if state.gesture.is_none() {
        clear_selection_snap_guides(&mut state);
      }
      draw_selection(&self.inner, &state);
    }
  }

  pub fn set_selection_visible(&self, visible: bool) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.selection_visible = visible;
      draw_selection(&self.inner, &state);
    }
  }

  /// The layers a press can land on. `hit_testing` lets a press pick and move
  /// them, as the select tool's do; without it they only say which picture a
  /// fresh annotation is drawn on. With no layers there is nothing to pick.
  pub fn set_selection_targets(&self, targets: Option<&[PreviewSelection]>, hit_testing: bool) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.selection_targets.clear();
      state
        .selection_targets
        .extend_from_slice(targets.unwrap_or_default());
      state.selection_hit_testing = hit_testing && targets.is_some();
    }
  }
}

/// Lays the platform's editing surface over the viewport as `state` has it.
fn place_viewport(inner: &SurfaceInner, state: &SurfaceState, active: bool) {
  let (x, right) = pixels::scaled_edges(state.viewport.x, state.viewport.width, state.scale);
  let (y, bottom) = pixels::scaled_edges(state.viewport.y, state.viewport.height, state.scale);
  inner.host.place_viewport(
    (x, y, right - x, bottom - y),
    state.viewport,
    state.scale,
    state.backdrop,
    active,
  );
}
