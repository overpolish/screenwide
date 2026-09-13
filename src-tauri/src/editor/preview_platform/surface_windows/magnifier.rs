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

/// The Gaussian sigma the suspended pane blurs with, in the pane's own
/// composed pixels, or zero when the editor is not suspended. Matches the
/// `filter: blur(var(--blur-lg))` the Windows chrome applies to itself, whose
/// sigma is the CSS length.
pub(super) fn suspended_blur_sigma(inner: &SurfaceInner, pane: &Pane) -> f64 {
  /// The CSS `--blur-lg` the DOM chrome blurs itself by, in CSS pixels.
  const SUSPENDED_BLUR_CSS_SIGMA: f64 = 5.0;
  if !inner.editor.is_suspended() {
    return 0.0;
  }
  // Two conversions: CSS pixels to device pixels by the window's scale, then
  // device pixels to pane pixels by the visual's own scale transform, which
  // maps the composed canvas onto its laid-out box.
  let device = SUSPENDED_BLUR_CSS_SIGMA * pane.scale;
  let content = f64::from(pane.content_size.0.max(1));
  let display = f64::from(pane.display_size.0.max(1) as u32);
  device * content / display
}

/// Redraws every visible pane from its cached source and composition, without
/// a decode. The suspension blur is applied inside the present path, and a
/// suspended editor presents nothing else, so toggling it has to re-present.
pub(super) fn redraw_composed_panes(
  inner: &std::sync::Arc<SurfaceInner>,
  state: &mut SurfaceState,
) {
  let camera_source = state.camera_source.clone();
  let surface = RecordingPreviewSurface {
    inner: std::sync::Arc::clone(inner),
  };
  for pane in state.panes.iter_mut().flatten().filter(|pane| pane.seen) {
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
