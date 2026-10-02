// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// How far a press outside the crop travels before it draws a new one, in
/// DIPs. Anything shorter is a click, which leaves the crop as it was.
const DRAW_THRESHOLD: f64 = 4.0;
/// The shortest side a drawn crop starts with, in DIPs, matching the floor
/// the macOS surface holds a resized crop to.
const MINIMUM_SIZE: f64 = 36.0;

/// The selected pane as it is drawn on screen.
fn display_pane(state: &SurfaceState, pane_index: u32) -> Option<PreviewSurfaceRect> {
  let pane = state
    .panes
    .get(pane_index as usize)
    .and_then(Option::as_ref)?;
  let rect = state.workspace_transform.apply(
    state.viewport,
    pane_canvas_rect(pane, state.frame_resize.is_some()),
  );
  (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

fn image_rect(selection: PreviewSelection) -> NormalizedRect {
  NormalizedRect {
    x: selection.image_x,
    y: selection.image_y,
    width: selection.image_width,
    height: selection.image_height,
  }
}

/// Arms a crop draw for a press that landed on nothing while a crop is being
/// edited. The gesture carries the anchor's offset from the crop's origin,
/// which is what its Begin reports once the press becomes a drag.
pub(super) fn start(state: &mut SurfaceState, point: (f64, f64)) -> Option<EditorGesture> {
  let selection = state
    .selection
    // A scene's crop window keeps its box's shape, so it is never drawn afresh.
    .filter(|selection| selection.crop_mode != 0 && selection.framed == 0)
    .filter(|selection| selection.image_width > 0.0 && selection.image_height > 0.0)?;
  let pane = display_pane(state, selection.pane_index)?;
  let image = image_rect(selection);
  let anchor = (
    ((point.0 - pane.x) / pane.width).clamp(image.x, image.x + image.width),
    ((point.1 - pane.y) / pane.height).clamp(image.y, image.y + image.height),
  );
  state.crop_draw = Some(CropDrawStart {
    anchor,
    begun: false,
  });
  Some(EditorGesture {
    edges: 0,
    last_delta: (anchor.0 - selection.x, anchor.1 - selection.y),
    last_scale: 1.0,
    operation: SelectionGestureOperation::CropDraw,
    pane_start: selection_pane_rect(state, selection),
    pointer_start: point,
    selection_start: selection,
    keyboard_start: None,
  })
}

/// One pointer sample of a crop draw. Answers the Begin to report when this
/// sample is the one that turned the press into a drag, and the Update.
pub(super) fn drag(
  inner: &std::sync::Arc<SurfaceInner>,
  state: &mut SurfaceState,
  mut gesture: EditorGesture,
  point: (f64, f64),
  centered: bool,
) -> (Option<EditorGesture>, Option<EditorGesture>) {
  let Some(mut draw) = state.crop_draw else {
    return (None, None);
  };
  let mut began = None;
  if !draw.begun {
    let travel = (point.0 - gesture.pointer_start.0).hypot(point.1 - gesture.pointer_start.1);
    if travel < DRAW_THRESHOLD {
      return (None, None);
    }
    draw.begun = true;
    state.crop_draw = Some(draw);
    began = Some(gesture);
  }
  let start = gesture.selection_start;
  let Some(pane) = display_pane(state, start.pane_index) else {
    return (began, None);
  };
  let drawn = apply_crop_draw(
    draw.anchor,
    (
      (point.0 - pane.x) / pane.width,
      (point.1 - pane.y) / pane.height,
    ),
    image_rect(start),
    (MINIMUM_SIZE / pane.width, MINIMUM_SIZE / pane.height),
    centered,
  );
  let mut selection = start;
  selection.x = drawn.rect.x;
  selection.y = drawn.rect.y;
  selection.width = drawn.rect.width;
  selection.height = drawn.rect.height;
  gesture.edges = drawn.edges;
  gesture.last_delta = drawn.delta;
  clear_selection_snap_guides(state);
  state.selection = Some(selection);
  state.gesture = Some(ActiveGesture::Selection(gesture));
  update_magnifier(state);
  redraw_magnifier(inner, state);
  draw_selection(inner, state);
  let _ = unsafe { inner.gpu.composition.Commit() };
  (began, Some(gesture))
}

/// Takes the crop draw, if any, and answers whether `gesture` is one the
/// frontend has heard begin. A crop draw that never became a drag was never
/// reported, so its end or cancel must not be either.
pub(super) fn finish(state: &mut SurfaceState, gesture: &EditorGesture) -> bool {
  let draw = state.crop_draw.take();
  gesture.operation != SelectionGestureOperation::CropDraw || draw.is_some_and(|draw| draw.begun)
}
