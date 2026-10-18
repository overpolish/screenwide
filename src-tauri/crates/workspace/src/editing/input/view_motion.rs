// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving the view over the workspace: zooming about a point, as a wheel or
//! a pinch does, and panning, as a trackpad's scroll does.

use super::*;

/// Zooms by `factor` about `point`, in logical points from the viewport's
/// corner, keeping what is under it there.
pub(super) fn zoom_at(inner: &Arc<SurfaceInner>, point: (f64, f64), factor: f64) {
  if !factor.is_finite() || factor <= 0.0 {
    return;
  }
  let mut zoom = None;
  if let Ok(mut state) = inner.state.lock() {
    state.last_pointer = point;
    let old = state.workspace_transform.zoom;
    let next = (old * factor).clamp(0.1, maximum_editor_zoom(&state));
    let ratio = next / old;
    let center = (state.viewport.width / 2.0, state.viewport.height / 2.0);
    state.workspace_transform.pan_x =
      point.0 - center.0 - (point.0 - center.0 - state.workspace_transform.pan_x) * ratio;
    state.workspace_transform.pan_y =
      point.1 - center.1 - (point.1 - center.1 - state.workspace_transform.pan_y) * ratio;
    state.workspace_transform.zoom = next;
    apply_workspace_transform(inner, &mut state, false);
    zoom = Some(next);
  }
  if let Some(zoom) = zoom {
    emit_transform(inner, zoom);
  }
}

/// Pans the view by `dx` and `dy` logical points.
pub(super) fn pan_by(inner: &Arc<SurfaceInner>, dx: f64, dy: f64) {
  if let Ok(mut state) = inner.state.lock() {
    state.workspace_transform.pan_x += dx;
    state.workspace_transform.pan_y += dy;
    apply_workspace_transform(inner, &mut state, false);
  }
}
