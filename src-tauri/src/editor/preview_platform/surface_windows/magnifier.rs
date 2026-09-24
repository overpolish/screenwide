// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn update_magnifier(state: &mut SurfaceState) {
  let Some(ActiveGesture::Selection(gesture)) = state.gesture else {
    for pane in state.panes.iter_mut().flatten() {
      pane.magnifier = None;
    }
    return;
  };
  let show = gesture.operation == SelectionGestureOperation::CropResize;
  for pane in state.panes.iter_mut().flatten() {
    pane.magnifier = None;
  }
  if !show {
    return;
  }
  let Some(selection) = state.selection else {
    return;
  };
  let owns_geometry = state.frame_resize.is_some();
  let Some(pane) = state
    .panes
    .get_mut(selection.pane_index as usize)
    .and_then(Option::as_mut)
  else {
    return;
  };
  let rect = state
    .workspace_transform
    .apply(state.viewport, pane_canvas_rect(pane, owns_geometry));
  let Some(settings) = pane.settings.as_ref() else {
    return;
  };
  let display_point = if gesture.operation == SelectionGestureOperation::CropResize {
    let frame = PreviewSurfaceRect {
      x: rect.x + selection.x * rect.width,
      y: rect.y + selection.y * rect.height,
      width: selection.width * rect.width,
      height: selection.height * rect.height,
    };
    crop_magnifier_anchor(
      [frame.x, frame.y, frame.width, frame.height],
      state.last_pointer,
      gesture.edges,
    )
  } else {
    state.last_pointer
  };
  let x = ((display_point.0 - rect.x) / rect.width * settings.width as f64) as f32;
  let y = ((display_point.1 - rect.y) / rect.height * settings.height as f64) as f32;
  let diameter = (96.0 * settings.width as f64 / rect.width.max(1.0)) as f32;
  let sample_camera = state.camera_source.is_some() && selection.layer_id != selection.pane_index;
  let luminance =
    state.backdrop[0] * 0.2126 + state.backdrop[1] * 0.7152 + state.backdrop[2] * 0.0722;
  pane.magnifier = Some(recenter::CropMagnifier {
    bounds: recenter::magnifier_bounds(selection),
    display_box: [
      (display_point.0 - 48.0) as f32,
      (display_point.1 - 48.0) as f32,
      96.0,
      96.0,
    ],
    // A 40-source-pixel window fills a 96-DIP native overlay.
    geometry: [x, y, diameter, diameter / 40.0],
    options: [
      if sample_camera { 1.0 } else { 0.0 },
      gesture.edges as f32,
      if luminance > 0.5 { 1.0 } else { 0.0 },
      0.0,
    ],
  });
}

/// Redraws every visible pane from its cached source and composition, without
/// a decode: what a change that lives only in the present path - a halo, a
/// magnifier - needs to reach the screen.
pub(super) fn redraw_composed_panes(
  inner: &std::sync::Arc<SurfaceInner>,
  state: &mut SurfaceState,
) {
  let camera_source = state.camera_source.clone();
  let hover = state.annotation.hover;
  annotation::sync_typing_marks(state);
  let surface = RecordingPreviewSurface {
    inner: std::sync::Arc::clone(inner),
  };
  for (index, pane) in state
    .panes
    .iter_mut()
    .enumerate()
    .filter_map(|(index, pane)| pane.as_mut().map(|pane| (index, pane)))
    .filter(|(_, pane)| pane.seen)
  {
    // The live presents key the halo by the layer they were given; a redraw
    // from cache has to resolve it again, or it draws the halo the pane last
    // saw. A screenshot pane knows its layer by token; a recording pane is
    // its own layer.
    pane.annotation_halo = hover
      .filter(|(layer, _, _)| pane.source_token == Some(*layer) || *layer == index as u64)
      .map(|(_, hovered, width)| (hovered, width));
    let (Some(settings), Some(composition), true) = (
      pane.settings.clone(),
      pane.last_composition,
      pane.source.is_some(),
    ) else {
      continue;
    };
    // As in `redraw_magnifier`: redraw exactly what the last present composed
    // rather than dropping a baked camera for a frame.
    let camera = match (pane.last_camera, camera_source.as_ref()) {
      (Some((geometry, drop_shadow, camera_on_top)), Some(source)) => {
        Some((source, geometry, drop_shadow, camera_on_top))
      }
      (Some(_), None) => continue,
      (None, _) => None,
    };
    let _ = surface.present_cached_source_with_camera(pane, &settings, composition, camera);
  }
}

pub(super) fn redraw_magnifier(inner: &std::sync::Arc<SurfaceInner>, state: &mut SurfaceState) {
  let Some(selection) = state.selection else {
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
  let (Some(settings), Some(composition)) = (pane.settings.clone(), pane.last_composition) else {
    return;
  };
  // Redraw exactly what the last present composed: dropping a baked camera
  // here would blank it in crop mode and flicker it during gestures.
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
  redraw_stale_selection(inner, state);
}
