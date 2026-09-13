// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn open(inner: &std::sync::Arc<SurfaceInner>, point: (f64, f64)) {
  if inner.editor.is_suspended() {
    return;
  }
  let Ok(mut state) = inner.state.lock() else {
    return;
  };
  if !state.editor_active || state.gesture.is_some() {
    return;
  }
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
  // Win32 input is local to the inset native viewport. React's menu is
  // positioned in the webview's logical content coordinates.
  let position = (state.viewport.x + point.0, state.viewport.y + point.1);
  drop(state);
  if changed {
    emit_selection(inner, Some(target.layer_id));
  }
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.context_menu.as_mut() {
      callback(target.layer_id, position.0, position.1);
    }
  }
}
