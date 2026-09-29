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
  /// The arrow the halo is on, by name. The pulse reports an index into the
  /// layer the grips were published for; keyboard shortcuts live in React,
  /// which knows arrows only by their id.
  pub(super) fn hovered_annotation_id(&self) -> Option<String> {
    self.annotation_hover.as_ref().map(|hover| hover.id.clone())
  }

  /// Whether the annotation the halo was put on is no longer where the halo
  /// is placed: deleted, or moved along its layer's list.
  pub(super) fn hover_lost(&self) -> bool {
    self.annotation_hover.as_ref().is_some_and(|hover| {
      self
        .output
        .as_ref()
        .and_then(|output| output.items.iter().find(|item| item.id == hover.layer_id))
        .and_then(|item| item.output.annotations.get(hover.index))
        .is_none_or(|annotation| annotation.id != hover.id)
    })
  }

  /// Lets the halo go, reporting whether there was one to let go of. Putting
  /// the tool down retires it: the pointer may never move again to do it.
  pub(super) fn clear_annotation_hover(&mut self) -> bool {
    self.annotation_hover.take().is_some()
  }

  /// One report from the native hover pulse. `index` is the arrow under the
  /// pointer, or negative for none; `image_points` is how wide the layer's
  /// picture is drawn on screen, which converts the halo's points into the
  /// canvas pixels the shader measures its distances in.
  ///
  /// The answer is the arrow the pointer has moved onto, and only when that
  /// changed: every frame of the pulse widens the halo, but the arrow under
  /// it is the same one, and React is told about the arrow rather than the
  /// animation.
  pub(crate) fn handle_annotation_hover(
    &mut self,
    index: i32,
    progress: f64,
    image_points: f64,
  ) -> Option<Option<String>> {
    let hover = self.annotation_hover_for(index, progress, image_points);
    if hover == self.annotation_hover {
      return None;
    }
    let previous = self.hovered_annotation_id();
    self.annotation_hover = hover;
    let _ = self.present_batch();
    let current = self.hovered_annotation_id();
    (previous != current).then_some(current)
  }

  fn annotation_hover_for(
    &self,
    index: i32,
    progress: f64,
    image_points: f64,
  ) -> Option<AnnotationHover> {
    // With no tool in hand, or the pen, there is no halo, whatever a pulse
    // report already on its way says: one landing after the tool changed would
    // bring back the halo that changing it retired.
    if crate::editor::annotations::gesture::hovers_nothing(self.annotation_mode) {
      return None;
    }
    let index = usize::try_from(index).ok()?;
    if !image_points.is_finite() || image_points <= 0.0 {
      return None;
    }
    // The chrome only ever hit-tests the selected pane, which is the one the
    // grips were published for.
    let pane_index = self.annotation_pane_index?;
    let item = self.output.as_ref()?.items.get(pane_index as usize)?;
    let annotation = item.output.annotations.get(index)?;
    let width = hover_width_points(progress) * item.output.image_width / image_points;
    Some(AnnotationHover {
      id: annotation.id.clone(),
      index,
      layer_id: item.id,
      width: width as f32,
    })
  }
}
