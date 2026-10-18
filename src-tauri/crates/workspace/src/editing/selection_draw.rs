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
  // Mid-layout the panes are not placed yet, so the selection would be
  // drawn without its layer for a frame; Core Animation shows each present
  // as it lands, so that frame would flicker. The layout's finish draws it.
  if state.workspace.laying_out {
    return;
  }
  let scale = state.scale.max(0.1);
  let (viewport_x, viewport_right) =
    pixels::scaled_edges(state.viewport.x, state.viewport.width, scale);
  let (viewport_y, viewport_bottom) =
    pixels::scaled_edges(state.viewport.y, state.viewport.height, scale);
  let display = (state.editor_active && state.selection_visible)
    .then(|| {
      state
        .selection
        .and_then(|selection| display_selection(state, selection).map(|rect| (selection, rect)))
        .map(|(selection, rect)| {
          let (x, right) = pixels::scaled_edges(rect.x, rect.width, scale);
          let (y, bottom) = pixels::scaled_edges(rect.y, rect.height, scale);
          let frame = [x as f32, y as f32, (right - x) as f32, (bottom - y) as f32];
          let radius = if selection.crop_mode == 0 && selection.radius_disabled == 0 {
            let radius = radius_point(rect, selection.radius_percent);
            [
              pixels::pixel_center(radius.0 * scale),
              pixels::pixel_center(radius.1 * scale),
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
  // leaves the layer's own chrome standing. Playback and a scrub hide every
  // chrome, as nothing on screen can be taken hold of meanwhile.
  let snap = annotation::snap_chrome(state, scale);
  let annotation_handles = (state.selection_visible && annotation::owns_chrome(state)).then(|| {
    let mut grips = annotation::selected_grips(state, scale);
    grips.extend(snap.anchor);
    grips
  });
  // A chosen redaction wears the layer selection's own box, grips and radius
  // dot inside the annotation chrome, and a chosen image its own frame,
  // turned with it.
  let annotation_parts = annotation_handles.is_some().then(|| {
    (
      annotation::selected_redaction(state, scale),
      annotation::selected_image_frame(state, scale),
      annotation::group_frames(state, scale),
      annotation::marquee_frame(state, scale),
    )
  });
  let annotation_chrome = annotation_handles
    .as_deref()
    .zip(annotation_parts.as_ref())
    .map(
      |(handles, (box_frame, image_frame, group, marquee))| chrome::AnnotationChrome {
        handles,
        box_frame: *box_frame,
        image_frame: *image_frame,
        bounds: snap.bounds,
        gaps: &snap.gaps,
        group,
        marquee: *marquee,
      },
    );
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
        .map(|guide| pixels::pixel_center((pane.x + guide.guide * pane.width) * scale));
      let y = state
        .selection_snap_guide_y
        .map(|guide| pixels::pixel_center((pane.y + guide.guide * pane.height) * scale));
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
  // The loupe is drawn by the workspace in the overlay's own pixels, both
  // laid over the viewport.
  let magnifier_box = state.workspace.magnifier.map(|magnifier| {
    [
      magnifier.box_origin[0] as f32,
      magnifier.box_origin[1] as f32,
      magnifier.box_size[0] as f32,
      magnifier.box_size[1] as f32,
    ]
  });
  let objects = state
    .selection_visible
    .then(|| object_picking::chrome(state))
    .flatten();
  // A turned layer's frame and annotation chrome are drawn through its turn,
  // and the select tool's rings over it.
  let turned = turn::chrome(state, scale);
  let (size, vertices, constants) = chrome::chrome(
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
    annotation_chrome.as_ref(),
    objects.as_ref(),
    &turned,
    scale,
    luminance > 0.5,
  );
  inner.host.present_chrome(size, &vertices, &constants);
}
