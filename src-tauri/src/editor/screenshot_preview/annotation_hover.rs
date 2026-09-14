// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The halo that grows under the arrow the pointer is resting on.
//!
//! The native side owns the pulse - it knows when the pointer enters and
//! leaves, and it already runs a display-rate timer for the ruler's hover -
//! and reports how far through it is. This side turns that into the halo
//! width the compositor draws with, in the layer's own canvas pixels, and
//! re-presents. Nothing else re-presents on pointer movement.

use super::annotation::{hover_width_points, AnnotationHover};
use super::state::PreviewManager;

impl PreviewManager {
  /// One report from the native hover pulse. `index` is the arrow under the
  /// pointer, or negative for none; `image_points` is how wide the layer's
  /// picture is drawn on screen, which converts the halo's points into the
  /// canvas pixels the shader measures its distances in.
  pub(crate) fn handle_annotation_hover(&mut self, index: i32, progress: f64, image_points: f64) {
    let hover = self.annotation_hover_for(index, progress, image_points);
    if hover == self.annotation_hover {
      return;
    }
    self.annotation_hover = hover;
    let _ = self.present_batch();
  }

  fn annotation_hover_for(
    &self,
    index: i32,
    progress: f64,
    image_points: f64,
  ) -> Option<AnnotationHover> {
    let index = usize::try_from(index).ok()?;
    if !image_points.is_finite() || image_points <= 0.0 {
      return None;
    }
    // The chrome only ever hit-tests the selected pane, which is the one the
    // grips were published for.
    let pane_index = self.annotation_pane_index?;
    let item = self.output.as_ref()?.items.get(pane_index as usize)?;
    if index >= item.output.annotations.len() {
      return None;
    }
    let width = hover_width_points(progress) * item.output.image_width / image_points;
    Some(AnnotationHover {
      index,
      layer_id: item.id,
      width: width as f32,
    })
  }
}
