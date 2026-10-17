// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The crop tool's loupe and the typing marks, which change what the
//! workspace scene draws without anything being presented again.

use super::*;
use crate::app_windows::overlay_surface::light_mode;
use crate::editor::preview_platform::CAPTIONS_LAYER_ID;
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
  let unit = |value: f64| value.clamp(0.0, 1.0) as f32;
  let size = (96.0 * scale).round().max(1.0) as i32;
  let center = (
    (point.0 * scale).round() as i32,
    (point.1 * scale).round() as i32,
  );
  Some(NativeMagnifier {
    active: 1,
    pane_index: selection.pane_index,
    layer_id: selection.layer_id,
    sample_camera: u32::from(sample_camera),
    edges,
    light_mode: u32::from(light_mode()),
    sample: [
      unit((pane_x - raw_x) / raw_width),
      unit((pane_y - raw_y) / raw_height),
    ],
    source_min: [unit(min_u), unit(min_v)],
    source_max: [
      unit(min_u + selection.image_width / raw_width),
      unit(min_v + selection.image_height / raw_height),
    ],
    box_origin: [center.0 - size / 2, center.1 - size / 2],
    box_size: [size as u32, size as u32],
  })
}

/// Draws the workspace again with the typing marks as they are now: the
/// caret and selection of the box or caption being typed into.
pub(super) fn redraw_workspace(inner: &std::sync::Arc<SurfaceInner>, state: &mut SurfaceState) {
  annotation::sync_typing_marks(state);
  let typing = state.panes.iter().enumerate().find_map(|(index, pane)| {
    let pane = pane.as_ref()?;
    if let Some(marks) = pane.caption_typing {
      return Some((CAPTIONS_LAYER_ID, 0, marks));
    }
    let (typed, marks) = pane.annotation_typing?;
    Some((u32::try_from(index).ok()?, typed, marks))
  });
  inner.gpu.workspace.scene.set_typing(typing);
  draw_workspace(inner, state);
}
