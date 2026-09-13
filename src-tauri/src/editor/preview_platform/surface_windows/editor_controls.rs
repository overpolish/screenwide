// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn set_audio_ribbon(
    &self,
    envelopes: &crate::editor::recording_preview_player::audio_visualizer::AudioRibbonEnvelopes,
  ) {
    let Ok(state) = self.inner.state.lock() else {
      return;
    };
    if let Ok(mut ribbon) = self.inner.gpu.audio_ribbon.lock() {
      self
        .inner
        .gpu
        .backdrop
        .set_geometry(state.viewport, state.scale);
      let _ = ribbon.set_viewport(state.viewport, state.scale, state.backdrop);
      if let Err(error) = ribbon.set_envelopes(envelopes) {
        eprintln!("Could not upload the audio ribbon: {error}");
      }
    }
  }

  pub(crate) fn set_audio_ribbon_playhead(&self, ratio: f64) {
    let Ok(state) = self.inner.state.lock() else {
      return;
    };
    if let Ok(mut ribbon) = self.inner.gpu.audio_ribbon.lock() {
      if ribbon.has_envelopes() {
        self
          .inner
          .gpu
          .backdrop
          .set_geometry(state.viewport, state.scale);
        let _ = ribbon.set_playhead(ratio);
      }
    }
  }

  pub(crate) fn set_scale(&self, scale: f64) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.scale = scale.max(0.1);
      if let Ok(mut ribbon) = self.inner.gpu.audio_ribbon.lock() {
        let _ = ribbon.set_viewport(state.viewport, state.scale, state.backdrop);
      }
      let (x, right) = window::scaled_edges(state.viewport.x, state.viewport.width, state.scale);
      let (y, bottom) = window::scaled_edges(state.viewport.y, state.viewport.height, state.scale);
      self
        .inner
        .editor
        .set_frame(x, y, right - x, bottom - y, state.editor_active);
      draw_selection(&self.inner, &state);
    }
  }

  pub(crate) fn enable_editor(&mut self, callback: TransformCallback) {
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
      let (x, right) = window::scaled_edges(state.viewport.x, state.viewport.width, state.scale);
      let (y, bottom) = window::scaled_edges(state.viewport.y, state.viewport.height, state.scale);
      self
        .inner
        .editor
        .set_frame(x, y, right - x, bottom - y, true);
      // The reset transform is 100%, so every pane returns to composing at
      // its on-screen size.
      draw_selection(&self.inner, &state);
    }
  }

  pub(crate) fn set_editor_active(&self, active: bool) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.editor_active = active;
      self.inner.editor.set_active(active);
      draw_selection(&self.inner, &state);
    }
  }

  pub(crate) fn set_editor_zoom(&self, zoom_percent: f64) {
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

  pub(crate) fn set_selection_callback(&mut self, callback: SelectionCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.selection = Some(callback);
    }
  }

  pub(crate) fn set_pointer_down_callback(&mut self, callback: PointerDownCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.pointer_down = Some(callback);
    }
  }

  pub(crate) fn set_context_menu_callback(&mut self, callback: ContextMenuCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.context_menu = Some(callback);
    }
  }

  pub(crate) fn set_selection_gesture_callback(&mut self, callback: SelectionGestureCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.gesture = Some(callback);
    }
  }

  pub(crate) fn set_selection_snapping(&self, enabled: bool) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.selection_snapping_enabled = enabled;
      if !enabled {
        clear_selection_snap_guides(&mut state);
        draw_selection(&self.inner, &state);
      }
    }
  }

  pub(crate) fn set_selection(&self, selection: Option<PreviewSelection>) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.selection = selection;
      if state.gesture.is_none() {
        clear_selection_snap_guides(&mut state);
      }
      draw_selection(&self.inner, &state);
    }
  }

  pub(crate) fn set_selection_visible(&self, visible: bool) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.selection_visible = visible;
      draw_selection(&self.inner, &state);
    }
  }

  pub(crate) fn set_selection_targets(&self, targets: Option<&[PreviewSelection]>) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.selection_targets.clear();
      state
        .selection_targets
        .extend_from_slice(targets.unwrap_or_default());
    }
  }
}
