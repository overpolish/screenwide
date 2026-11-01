// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A press read on the layer an annotation belongs to. Each layer is turned
//! its own way, so an annotation on a layer other than the selected one is
//! measured on its own layer, not against the selected layer's reading.

use super::*;

/// `point`, read on the selected layer, as it lands on `item`'s own layer.
pub(super) fn point_on_item_layer(
  state: &SurfaceState,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> (f64, f64) {
  layer_selection(state, item.layer_id).map_or(point, |layer| {
    super::super::super::turn::point_on_layer(state, layer, point)
  })
}

/// The on-screen `display` as it lands on the layer of the annotation at
/// `index`, standing upright: where a press that chooses it, and so selects
/// its layer, starts the carry from.
pub(in super::super) fn display_on_item_layer(
  state: &SurfaceState,
  index: usize,
  display: (f64, f64),
) -> Option<(f64, f64)> {
  let item = state.annotation.handles.get(index)?;
  let layer = layer_selection(state, item.layer_id)?;
  Some(super::super::super::turn::unturned_point(
    state, layer, display,
  ))
}

/// A laid-out picture's layer, the rect it is drawn in, its corners' radius
/// and a press as it lands on it standing upright.
type DrawnPicture = (u32, [f64; 4], f64, (f64, f64));

/// Every laid-out picture, back to front, with the rect it is drawn in, its
/// corners' radius and `point`, read on the selected layer, as it lands on it
/// standing upright, for `hidden_by_layer_above`.
pub(super) fn drawn_pictures(state: &SurfaceState, point: (f64, f64)) -> Vec<DrawnPicture> {
  state
    .selection_targets
    .iter()
    .filter_map(|target| {
      let selection = layer_selection(state, target.layer_id as i32)?;
      let rect = display_selection(state, selection)?;
      let radius = selection.radius_percent.clamp(0.0, 50.0) / 100.0 * rect.width.min(rect.height);
      Some((
        target.layer_id,
        [rect.x, rect.y, rect.width, rect.height],
        radius,
        super::super::super::turn::point_on_layer(state, selection, point),
      ))
    })
    .collect()
}
