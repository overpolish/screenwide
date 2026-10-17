// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Mirrors the content-aware ceiling used by the toolbar and the macOS
/// surface. A scrolling capture is fitted far below one point per output
/// pixel, so its usable ceiling has to grow with that fit rather than stop at
/// the ordinary 16x limit.
pub(super) fn maximum_editor_zoom(state: &SurfaceState) -> f64 {
  let scale = state.scale.max(0.000_001);
  if let Some((width, height)) = state.workspace_natural_size {
    let bounds = state
      .panes
      .iter()
      .flatten()
      .filter(|pane| pane.seen)
      .map(|pane| pane.base_rect)
      .reduce(union_rect);
    if let Some(bounds) = bounds.filter(|bounds| bounds.width > 0.0 && bounds.height > 0.0) {
      let native_scale = (f64::from(width) / (bounds.width * scale))
        .max(f64::from(height) / (bounds.height * scale));
      return MINIMUM_EDITOR_ZOOM_CEILING.max(NATIVE_PIXEL_ZOOM_HEADROOM * native_scale);
    }
  }

  // Screenshot workspaces retain their natural canvas size on the pane's
  // output settings instead of `workspace_natural_size`.
  state
    .panes
    .iter()
    .flatten()
    .filter(|pane| pane.seen)
    .filter_map(|pane| {
      let settings = pane.settings.as_ref()?;
      if pane.base_rect.width <= 0.0 || pane.base_rect.height <= 0.0 {
        return None;
      }
      Some(
        (f64::from(settings.width) / (pane.base_rect.width * scale))
          .max(f64::from(settings.height) / (pane.base_rect.height * scale)),
      )
    })
    .fold(MINIMUM_EDITOR_ZOOM_CEILING, |ceiling, native_scale| {
      ceiling.max(NATIVE_PIXEL_ZOOM_HEADROOM * native_scale)
    })
}

/// Mirrors the macOS surface's `auto_fit_selection_bounds`: the smallest
/// whole-pixel box, in mouse-down canvas units, holding the canvas and every
/// layer of the moved layer's pane with the moved layer at `moved`.
pub(super) fn auto_fit_selection_bounds(
  auto_fit: &MoveAutoFit,
  moved: PreviewSelection,
) -> PreviewSurfaceRect {
  let mut left = 0.0_f64;
  let mut top = 0.0_f64;
  let mut right = 1.0_f64;
  let mut bottom = 1.0_f64;
  let mut include = |target: PreviewSelection| {
    left = left.min(target.x);
    top = top.min(target.y);
    right = right.max(target.x + target.width);
    bottom = bottom.max(target.y + target.height);
  };
  for target in auto_fit
    .targets_start
    .iter()
    .filter(|target| target.pane_index == moved.pane_index && target.layer_id != moved.layer_id)
  {
    include(*target);
  }
  include(moved);
  if let Some((width, height)) = auto_fit.natural_size {
    let width = width.max(1.0);
    let height = height.max(1.0);
    left = (left * width).floor() / width;
    top = (top * height).floor() / height;
    right = (right * width).ceil() / width;
    bottom = (bottom * height).ceil() / height;
  }
  PreviewSurfaceRect {
    x: left,
    y: top,
    width: (right - left).max(0.000_001),
    height: (bottom - top).max(0.000_001),
  }
}

/// The immutable workspace a Frame resize or an auto-fit Move re-derives from.
pub(super) fn frame_resize_start(state: &SurfaceState) -> FrameResizeStart {
  FrameResizeStart {
    natural_size: state.workspace_natural_size,
    pane_rects: state
      .panes
      .iter()
      .enumerate()
      .filter_map(|(index, pane)| {
        pane
          .as_ref()
          .filter(|pane| pane.seen)
          .map(|pane| (index, pane_canvas_rect(pane, false)))
      })
      .collect(),
    transform: state.workspace_transform,
  }
}

/// Puts `pane`'s box at `rect`, viewport-local, in the window's physical
/// pixels, which the workspace draws its layer in.
pub(super) fn set_pane_geometry(
  pane: &mut Pane,
  viewport: PreviewSurfaceRect,
  rect: PreviewSurfaceRect,
  scale: f64,
) {
  let (x, right) = window::scaled_edges(viewport.x + rect.x, rect.width, scale);
  let (y, bottom) = window::scaled_edges(viewport.y + rect.y, rect.height, scale);
  pane.position = (x, y);
  pane.display_size = ((right - x).max(2), (bottom - y).max(2));
  pane.scale = scale;
}

/// Where the pane's composed canvas actually is inside the box it was laid
/// out in. A committed canvas resize keeps arriving from the DOM at the
/// session's source aspect, and the workspace centres the canvas inside it,
/// so selection geometry has to follow the canvas rather than the box to stay
/// on the pixels the user sees.
pub(super) fn pane_canvas_rect(pane: &Pane, owns_geometry: bool) -> PreviewSurfaceRect {
  // A live Frame resize already places the canvas itself, so nothing there
  // needs fitting - and fitting against the composition that is still one
  // present behind would make the overlay flicker for the whole drag.
  match pane.settings.as_ref().filter(|_| !owns_geometry) {
    Some(settings) => aspect_fit_rect(pane.base_rect, (settings.width, settings.height)),
    None => pane.base_rect,
  }
}

pub(super) fn display_selection(
  state: &SurfaceState,
  selection: PreviewSelection,
) -> Option<PreviewSurfaceRect> {
  // A pane the last layout hid (the camera while it is baked into the
  // primary) has no display rect: its stale box must not select or hit-test,
  // matching the macOS surface's active-pane check.
  let pane = state
    .panes
    .get(selection.pane_index as usize)?
    .as_ref()
    .filter(|pane| pane.seen)?;
  let pane = state.workspace_transform.apply(
    state.viewport,
    pane_canvas_rect(pane, state.frame_resize.is_some()),
  );
  Some(PreviewSurfaceRect {
    x: pane.x + selection.x * pane.width,
    y: pane.y + selection.y * pane.height,
    width: selection.width * pane.width,
    height: selection.height * pane.height,
  })
}
