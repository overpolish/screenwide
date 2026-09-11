// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  apply_workspace_transform, emit_transform, union_rect, RecordingPreviewSurface, SurfaceState,
  WorkspaceTransform,
};

/// The transform the current fit basis calls for: the panel fit while a tool
/// panel holds a width, otherwise 100% centred in the whole viewport, which is
/// what `for_panel` answers for a width of zero.
pub(super) fn fit_basis_transform(state: &SurfaceState) -> WorkspaceTransform {
  state
    .panes
    .iter()
    .flatten()
    .filter(|pane| pane.seen)
    .map(|pane| pane.base_rect)
    .reduce(union_rect)
    .map(|base| WorkspaceTransform::for_panel(base, state.viewport, state.panel_fit_width))
    .unwrap_or_default()
}

impl RecordingPreviewSurface {
  /// One-time transform, and the width also becomes the basis a double-click
  /// resets to; subsequent resizes use normal base geometry.
  pub(crate) fn reset_editor_view(&self, fit_width: Option<f64>) {
    let zoom = {
      let Ok(mut state) = self.inner.state.lock() else {
        return;
      };
      state.panel_fit_width = fit_width.filter(|width| *width > 0.0).unwrap_or(0.0);
      let transform = fit_basis_transform(&state);
      state.workspace_transform = transform;
      apply_workspace_transform(&self.inner, &mut state, false);
      transform.zoom
    };
    emit_transform(&self.inner, zoom);
  }

  /// Moves the double-click reset basis without moving the view.
  pub(crate) fn set_editor_fit_basis(&self, fit_width: Option<f64>) {
    if let Ok(mut state) = self.inner.state.lock() {
      state.panel_fit_width = fit_width.filter(|width| *width > 0.0).unwrap_or(0.0);
    }
  }
}
