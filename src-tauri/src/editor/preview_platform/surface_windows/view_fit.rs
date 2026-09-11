// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  apply_workspace_transform, emit_transform, union_rect, RecordingPreviewSurface,
  WorkspaceTransform,
};

impl RecordingPreviewSurface {
  /// One-time transform; subsequent resize and double-click use normal base geometry.
  pub(crate) fn reset_editor_view(&self, fit_width: Option<f64>) {
    let zoom = {
      let Ok(mut state) = self.inner.state.lock() else {
        return;
      };
      let transform = fit_width
        .and_then(|width| {
          state
            .panes
            .iter()
            .flatten()
            .filter(|pane| pane.seen)
            .map(|pane| pane.base_rect)
            .reduce(union_rect)
            .map(|base| WorkspaceTransform::for_panel(base, state.viewport, width))
        })
        .unwrap_or_default();
      state.workspace_transform = transform;
      apply_workspace_transform(&self.inner, &mut state, false);
      transform.zoom
    };
    emit_transform(&self.inner, zoom);
  }
}
