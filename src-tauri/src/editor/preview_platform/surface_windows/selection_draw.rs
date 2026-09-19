// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Redraws the selection overlay if a present since the last draw changed a
/// pane's composed canvas size. Inside an open batch the flush does this once
/// for every pane, after the deferred geometry has been applied.
pub(super) fn redraw_stale_selection(inner: &SurfaceInner, state: &mut SurfaceState) {
  if inner.batch_depth.load(Ordering::Acquire) > 0 {
    return;
  }
  let mut stale = false;
  for pane in state.panes.iter_mut().flatten() {
    stale |= std::mem::take(&mut pane.selection_stale);
  }
  if stale {
    draw_selection(inner, state);
  }
}

pub(super) fn draw_selection(inner: &SurfaceInner, state: &SurfaceState) {
  if inner.batch_depth.load(Ordering::Acquire) > 0 {
    inner.selection_pending.store(true, Ordering::Release);
    return;
  }
  let scale = state.scale.max(0.1);
  let (viewport_x, viewport_right) =
    window::scaled_edges(state.viewport.x, state.viewport.width, scale);
  let (viewport_y, viewport_bottom) =
    window::scaled_edges(state.viewport.y, state.viewport.height, scale);
  let display = (state.editor_active && state.selection_visible)
    .then(|| {
      state
        .selection
        .and_then(|selection| display_selection(state, selection).map(|rect| (selection, rect)))
        .map(|(selection, rect)| {
          let (x, right) = window::scaled_edges(rect.x, rect.width, scale);
          let (y, bottom) = window::scaled_edges(rect.y, rect.height, scale);
          let frame = [x as f32, y as f32, (right - x) as f32, (bottom - y) as f32];
          let radius = if selection.crop_mode == 0 && selection.radius_disabled == 0 {
            let radius = radius_point(rect, selection.radius_percent);
            [
              window::pixel_center(radius.0 * scale),
              window::pixel_center(radius.1 * scale),
            ]
          } else {
            [f32::NAN, f32::NAN]
          };
          (frame, radius)
        })
    })
    .flatten();
  let frame = display.map(|value| value.0);
  let radius = display.map(|value| value.1);
  let crop_image = display.and_then(|_| {
    let selection = state.selection?;
    (selection.crop_mode != 0).then_some(())?;
    let image = state
      .panes
      .get(selection.pane_index as usize)
      .and_then(Option::as_ref)
      .map(|pane| {
        state.workspace_transform.apply(
          state.viewport,
          pane_canvas_rect(pane, state.frame_resize.is_some()),
        )
      })?;
    Some([
      ((image.x + selection.image_x * image.width) * scale) as f32,
      ((image.y + selection.image_y * image.height) * scale) as f32,
      (selection.image_width * image.width * scale) as f32,
      (selection.image_height * image.height * scale) as f32,
    ])
  });
  // Crop mode keeps the layer's own radius so the shade can round with it.
  let crop_radius_percent = state
    .selection
    .filter(|selection| selection.crop_mode != 0 && selection.radius_disabled == 0)
    .map_or(0.0, |selection| selection.radius_percent);
  // The arrow chrome, when it owns the screen: its grips, which may be none
  // at all with the arrow tool in hand and nothing chosen yet, plus the disc
  // over a snapped tip, which is drawn exactly like one of them. `None`
  // leaves the layer's own chrome standing.
  let snap = annotation::snap_chrome(state, scale);
  let annotation_handles = annotation::owns_chrome(state).then(|| {
    let mut grips = annotation::selected_grips(state, scale);
    grips.extend(snap.anchor);
    grips
  });
  // An annotation gesture owns the guides for as long as it owns the chrome:
  // its candidates are the source image's own lines, not the canvas's.
  let guides = if annotation_handles.is_some() {
    snap.guides
  } else {
    display.and_then(|_| {
      let selection = state.selection?;
      let pane = state.panes.get(selection.pane_index as usize)?.as_ref()?;
      let pane = state.workspace_transform.apply(
        state.viewport,
        pane_canvas_rect(pane, state.frame_resize.is_some()),
      );
      let x = state
        .selection_snap_guide_x
        .map(|guide| window::pixel_center((pane.x + guide.guide * pane.width) * scale));
      let y = state
        .selection_snap_guide_y
        .map(|guide| window::pixel_center((pane.y + guide.guide * pane.height) * scale));
      Some((
        x,
        y,
        state
          .selection_snap_guide_x
          .is_some_and(|guide| guide.object),
        state
          .selection_snap_guide_y
          .is_some_and(|guide| guide.object),
      ))
    })
  };
  let luminance =
    state.backdrop[0] * 0.2126 + state.backdrop[1] * 0.7152 + state.backdrop[2] * 0.0722;
  let magnifier_box = state
    .selection
    .and_then(|selection| state.panes.get(selection.pane_index as usize))
    .and_then(Option::as_ref)
    .and_then(|pane| pane.magnifier)
    .map(|magnifier| {
      let [x, y, width, height] = magnifier.display_box;
      [
        x * scale as f32,
        y * scale as f32,
        width * scale as f32,
        height * scale as f32,
      ]
    });
  if let Ok(mut overlay) = inner.gpu.selection.lock() {
    let _ = overlay.draw(
      &inner.gpu.device,
      &inner.gpu.context,
      (
        (viewport_right - viewport_x).max(2) as u32,
        (viewport_bottom - viewport_y).max(2) as u32,
      ),
      frame,
      radius.filter(|point| point[0].is_finite() && point[1].is_finite()),
      crop_image,
      crop_radius_percent,
      guides,
      magnifier_box,
      annotation_handles.as_deref(),
      snap.bounds,
      &snap.gaps,
      scale,
      luminance > 0.5,
    );
  }
}
