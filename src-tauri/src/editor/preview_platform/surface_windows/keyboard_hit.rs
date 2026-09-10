// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Current-frame keyboard bounds used for body hit-testing.

use super::{
  display_selection, PreviewSelection, PreviewSurfaceRect, RecordingPreviewSurface, SurfaceInner,
  SurfaceState,
};

pub(super) fn frame(
  inner: &SurfaceInner,
  state: &SurfaceState,
  mut selection: PreviewSelection,
) -> Option<PreviewSurfaceRect> {
  if selection.layer_id != u32::MAX - 1 {
    return display_selection(state, selection);
  }
  let pane = state.panes.get(selection.pane_index as usize)?.as_ref()?;
  let settings = pane.settings.as_ref()?;
  let overlay = pane.last_composition?.keyboard?;
  let bounds = inner
    .gpu
    .compositor
    .keyboard_visible_bounds(
      &inner.gpu.device,
      &overlay,
      (settings.width, settings.height),
    )
    .ok()??;
  selection.x = bounds[0];
  selection.y = bounds[1];
  selection.width = bounds[2];
  selection.height = bounds[3];
  display_selection(state, selection)
}

pub(super) fn clamp_move(
  selection: &mut PreviewSelection,
  gesture: &mut super::EditorGesture,
  state: &mut SurfaceState,
) {
  if selection.layer_id != u32::MAX - 1 {
    return;
  }
  use super::super::workspace_editor::keyboard_bounds::screenwide_keyboard_clamp_origin;
  let x = screenwide_keyboard_clamp_origin(selection.x, selection.width);
  let y = screenwide_keyboard_clamp_origin(selection.y, selection.height);
  if x != selection.x || y != selection.y {
    super::clear_selection_snap_guides(state);
  }
  selection.x = x;
  selection.y = y;
  gesture.last_delta = (
    selection.x - gesture.selection_start.x,
    selection.y - gesture.selection_start.y,
  );
}

pub(super) fn resize_limit(
  start: PreviewSelection,
  resize: &mut super::super::workspace_editor::SelectionResize,
) {
  if start.layer_id != u32::MAX - 1 {
    return;
  }
  use super::super::workspace_editor::keyboard_bounds::screenwide_keyboard_resize_limit;
  resize.maximum_scale = resize.maximum_scale.min(screenwide_keyboard_resize_limit(
    start.x,
    start.y,
    start.width,
    start.height,
    resize.anchor.0,
    resize.anchor.1,
  ));
  resize.minimum_scale = resize.minimum_scale.min(resize.maximum_scale);
  resize.scale = resize
    .scale
    .clamp(resize.minimum_scale, resize.maximum_scale);
}

pub(super) fn redraw_keyboard_transform(
  inner: &std::sync::Arc<SurfaceInner>,
  state: &mut SurfaceState,
  start: Option<crate::editor::keyboard_effects::KeyboardOverlay>,
  selection: PreviewSelection,
  scale: f64,
) {
  let Some(mut keyboard) = start else {
    return;
  };
  let camera_source = state.camera_source.clone();
  let Some(pane) = state
    .panes
    .get_mut(selection.pane_index as usize)
    .and_then(Option::as_mut)
  else {
    return;
  };
  let (Some(settings), Some(mut composition)) = (pane.settings.clone(), pane.last_composition)
  else {
    return;
  };
  keyboard.center_x = (selection.x + selection.width / 2.0) as f32;
  keyboard.center_y = (selection.y + selection.height / 2.0) as f32;
  keyboard.requested_scale *= scale as f32;
  keyboard.scale *= scale as f32;
  let key_count = keyboard.key_count.min(keyboard.keys.len() as u32) as usize;
  for key in &mut keyboard.keys[..key_count] {
    key.scale *= scale as f32;
  }
  composition.keyboard = Some(keyboard);
  let camera = match (pane.last_camera, camera_source.as_ref()) {
    (Some((geometry, drop_shadow, camera_on_top)), Some(source)) => {
      Some((source, geometry, drop_shadow, camera_on_top))
    }
    (Some(_), None) => return,
    (None, _) => None,
  };
  let surface = RecordingPreviewSurface {
    inner: std::sync::Arc::clone(inner),
  };
  let _ = surface.present_cached_source_with_camera(pane, &settings, composition, camera);
}

pub(super) fn keyboard_transform_start(
  state: &SurfaceState,
  selection: PreviewSelection,
) -> Option<crate::editor::keyboard_effects::KeyboardOverlay> {
  (selection.layer_id == u32::MAX - 1)
    .then(|| {
      state
        .panes
        .get(selection.pane_index as usize)
        .and_then(Option::as_ref)
        .and_then(|pane| pane.last_composition)
        .and_then(|composition| composition.keyboard)
    })
    .flatten()
}
