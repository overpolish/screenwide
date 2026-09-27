// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The annotation or layer under a right press, handed to the web layer so it
/// can open the app's own menu there. An annotation comes before the layer it
/// is drawn on, as it does for a left press, and is chosen the same way so the
/// menu acts on what it was opened on.
pub(super) fn open(inner: &std::sync::Arc<SurfaceInner>, point: (f64, f64)) {
  if inner.editor.is_suspended() {
    return;
  }
  {
    let Ok(state) = inner.state.lock() else {
      return;
    };
    if !state.editor_active || state.gesture.is_some() {
      return;
    }
  }
  if let Some(annotation) = annotation::choose_at(inner, point) {
    let Ok(state) = inner.state.lock() else {
      return;
    };
    let layer = state.selection.map_or(0, |selection| selection.layer_id);
    let position = content_point(&state, point);
    drop(state);
    report(inner, layer, Some(annotation as u32), position);
    return;
  }
  let Ok(mut state) = inner.state.lock() else {
    return;
  };
  let Some((target, _)) = shared_selection_hit(inner, &state, point) else {
    return;
  };
  // Frames and keyboard overlays have no layer-order menu.
  if target.layer_id >= u32::MAX - 1 {
    return;
  }
  let changed = state.selection.is_none_or(|current| {
    current.pane_index != target.pane_index || current.layer_id != target.layer_id
  });
  state.last_pointer = point;
  state.selection = Some(target);
  clear_selection_snap_guides(&mut state);
  draw_selection(inner, &state);
  let position = content_point(&state, point);
  drop(state);
  if changed {
    emit_selection(inner, Some(target.layer_id));
  }
  report(inner, target.layer_id, None, position);
}

/// Win32 input is local to the inset native viewport. React's menu is
/// positioned in the webview's logical content coordinates.
fn content_point(state: &SurfaceState, point: (f64, f64)) -> (f64, f64) {
  (state.viewport.x + point.0, state.viewport.y + point.1)
}

/// Reports the press. MUST be called with no surface state held.
fn report(inner: &SurfaceInner, layer: u32, annotation: Option<u32>, position: (f64, f64)) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.context_menu.as_mut() {
      callback(layer, annotation, position.0, position.1);
    }
  }
}
