// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The crop tool's loupe and the typing marks, which change what the
//! workspace scene draws without anything being presented again.

use super::*;
use screenwide_osc::gpu::magnifier::NativeMagnifier;

/// Puts the loupe where a crop resize, or a crop draw that has found its
/// corner, reads the picture; anything else takes it away.
pub(super) fn update_magnifier(state: &mut SurfaceState) {
  state.workspace.magnifier = None;
  let Some(ActiveGesture::Selection(gesture)) = state.gesture else {
    return;
  };
  // Only the side bits name the corner the loupe rides on. A crop draw has
  // no corner until its first sample places one.
  let edges = gesture.edges & 0b1111;
  let show = gesture.operation == SelectionGestureOperation::CropResize
    || (gesture.operation == SelectionGestureOperation::CropDraw && edges != 0);
  if !show {
    return;
  }
  let Some(selection) = state.selection else {
    return;
  };
  let owns_geometry = state.frame_resize.is_some();
  let Some(pane) = state
    .panes
    .get(selection.pane_index as usize)
    .and_then(Option::as_ref)
  else {
    return;
  };
  let rect = state
    .workspace_transform
    .apply(state.viewport, pane_canvas_rect(pane, owns_geometry));
  let display_point = crop_magnifier_anchor(
    [
      rect.x + selection.x * rect.width,
      rect.y + selection.y * rect.height,
      selection.width * rect.width,
      selection.height * rect.height,
    ],
    state.last_pointer,
    edges,
  );
  // A recording's camera baked into the screen's pane is a layer of its own
  // there, and the loupe reads its frame.
  let sample_camera = state.workspace.recording && selection.layer_id != selection.pane_index;
  state.workspace.magnifier = scene_magnifier(
    selection,
    rect,
    display_point,
    edges,
    state.scale,
    sample_camera,
  );
}

/// The loupe the workspace scene draws, as the macOS surface makes it: the
/// point it reads as a share of the uncropped picture, the crop's span of
/// that picture, and a 96-point box centred on `point`. `rect` is the canvas
/// on screen and `point` the anchor, both viewport-local.
fn scene_magnifier(
  selection: PreviewSelection,
  rect: PreviewSurfaceRect,
  point: (f64, f64),
  edges: u32,
  scale: f64,
  sample_camera: bool,
) -> Option<NativeMagnifier> {
  if rect.width <= 0.0
    || rect.height <= 0.0
    || selection.image_width <= 0.0
    || selection.image_height <= 0.0
  {
    return None;
  }
  let pane_x = (point.0 - rect.x) / rect.width;
  let pane_y = (point.1 - rect.y) / rect.height;
  let (raw_x, raw_y, raw_width, raw_height) =
    if selection.recenter_width > 0.0 && selection.recenter_height > 0.0 {
      (
        selection.recenter_x,
        selection.recenter_y,
        selection.recenter_width,
        selection.recenter_height,
      )
    } else {
      (
        selection.image_x,
        selection.image_y,
        selection.image_width,
        selection.image_height,
      )
    };
  let min_u = (selection.image_x - raw_x) / raw_width;
  let min_v = (selection.image_y - raw_y) / raw_height;
  let share = |value: f64| value as f32;
  // The draw stamps the system's light mode on, as the host reports it then.
  Some(NativeMagnifier::new(
    point,
    scale,
    edges,
    0,
    (
      selection.pane_index,
      selection.layer_id,
      u32::from(sample_camera),
    ),
    [
      share((pane_x - raw_x) / raw_width),
      share((pane_y - raw_y) / raw_height),
    ],
    [share(min_u), share(min_v)],
    [
      share(min_u + selection.image_width / raw_width),
      share(min_v + selection.image_height / raw_height),
    ],
  ))
}

/// Draws the workspace again with the typing marks as they are now: the
/// caret and selection of the box or caption being typed into.
pub(super) fn redraw_workspace(inner: &std::sync::Arc<SurfaceInner>, state: &mut SurfaceState) {
  annotation::sync_typing_marks(inner, state);
  draw_workspace(inner, state);
}
