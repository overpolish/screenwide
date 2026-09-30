// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Several annotations chosen together, and the marquee that chooses them:
//! the boxes drawn round a group, where a press lands on one, and the band
//! drawn over empty picture. The twin of
//! `recording_preview_surface_macos+annotation_group.m`.

use super::picking::layer_image_rect;
use super::*;
use crate::editor::annotations::gesture::MODE_SELECT;
use crate::editor::annotations::group::{NativeAnnotationGroupBox, GROUP_BOX_GROUP};

impl RecordingPreviewSurface {
  /// Publishes the boxes drawn round the annotations chosen together: round
  /// each one shown, and round the whole group on each layer. An empty list
  /// is what puts the group chrome away.
  pub(crate) fn set_annotation_group(&self, boxes: &[NativeAnnotationGroupBox]) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    if state.annotation.group.as_slice() == boxes {
      return;
    }
    state.annotation.group.clear();
    state.annotation.group.extend_from_slice(boxes);
    draw_selection(&self.inner, &state);
  }
}

fn contains(rect: PreviewSurfaceRect, point: (f64, f64)) -> bool {
  point.0 >= rect.x
    && point.0 <= rect.x + rect.width
    && point.1 >= rect.y
    && point.1 <= rect.y + rect.height
}

/// A rectangle in display points as the overlay draws it, in device pixels.
fn device_rect(rect: PreviewSurfaceRect, scale: f64) -> [f32; 4] {
  let (x, right) = window::scaled_edges(rect.x, rect.width, scale);
  let (y, bottom) = window::scaled_edges(rect.y, rect.height, scale);
  [x as f32, y as f32, (right - x) as f32, (bottom - y) as f32]
}

/// One group box on screen, in display points, where its layer is laid out.
fn box_rect(
  state: &SurfaceState,
  group_box: &NativeAnnotationGroupBox,
) -> Option<PreviewSurfaceRect> {
  let image = layer_image_rect(state, group_box.layer_id)?;
  Some(PreviewSurfaceRect {
    x: image.x + image.width * group_box.left,
    y: image.y + image.height * group_box.top,
    width: image.width * (group_box.right - group_box.left),
    height: image.height * (group_box.bottom - group_box.top),
  })
}

/// Whether several annotations are chosen together, which is when the group
/// chrome is drawn and a press on a member carries them all.
pub(crate) fn has_group(state: &SurfaceState) -> bool {
  !state.annotation.group.is_empty() && state.annotation.mode != MODE_NONE
}

/// The layer whose whole-group box `point` lands inside, with only the
/// select tool in hand: a press there carries the group.
pub(super) fn group_layer_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<i32> {
  if state.annotation.mode != MODE_SELECT || !has_group(state) {
    return None;
  }
  state
    .annotation
    .group
    .iter()
    .filter(|group_box| group_box.kind == GROUP_BOX_GROUP)
    .find(|group_box| box_rect(state, group_box).is_some_and(|rect| contains(rect, point)))
    .map(|group_box| group_box.layer_id)
}

/// The layer a marquee pressed at `point` is drawn over: the laid-out layer
/// under it, or the selected one where no layer is.
pub(super) fn marquee_layer_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<i32> {
  state
    .selection
    .iter()
    .chain(state.selection_targets.iter())
    .map(|layer| layer.layer_id as i32)
    .find(|layer| layer_image_rect(state, *layer).is_some_and(|rect| contains(rect, point)))
    .or_else(|| state.selection.map(|selection| selection.layer_id as i32))
}

/// The group's boxes in device pixels, for the overlay to draw as the layer
/// selection's own frame without handles: every whole-group box first, then
/// one round each member shown.
pub(crate) fn group_frames(state: &SurfaceState, scale: f64) -> Vec<[f32; 4]> {
  if !has_group(state) {
    return Vec::new();
  }
  let whole = |group_box: &&NativeAnnotationGroupBox| group_box.kind == GROUP_BOX_GROUP;
  let group = &state.annotation.group;
  group
    .iter()
    .filter(whole)
    .chain(group.iter().filter(|group_box| !whole(group_box)))
    .filter_map(|group_box| box_rect(state, group_box))
    .filter(|rect| rect.width > 0.0 || rect.height > 0.0)
    .map(|rect| device_rect(rect, scale))
    .collect()
}

/// The marquee band being drawn, in device pixels.
pub(crate) fn marquee_frame(state: &SurfaceState, scale: f64) -> Option<[f32; 4]> {
  state
    .annotation
    .marquee
    .map(|band| device_rect(band, scale))
}
