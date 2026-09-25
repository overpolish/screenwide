// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where an annotation's picture is on screen, and what a press lands on.
//!
//! Annotations are published in their layer's image-normalised space, so
//! picking first has to find that image inside the pane the layer is drawn in.
//! The twin of `recording_preview_annotation_geometry_macos.h` and the layer
//! lookup in `recording_preview_annotation_layers_macos.h`.

use super::*;
use crate::editor::annotations::arrow::distance::prepared_arrow_distance;
use crate::editor::annotations::arrow::geometry::prepare_arrow;
use crate::editor::annotations::counter::silhouette::counter_distance;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::text::geometry::{prepare_text, text_distance};
use crate::editor::annotations::AnnotationPoint;

/// An annotation belongs to its image, independently of the selected layer. The
/// twin of `annotation_layer_selection`.
pub(super) fn layer_selection(state: &SurfaceState, layer: i32) -> Option<PreviewSelection> {
  if let Some(current) = state.selection {
    if layer < 0 || current.layer_id == layer as u32 {
      return Some(current);
    }
  }
  state
    .selection_targets
    .iter()
    .copied()
    .find(|target| layer >= 0 && target.layer_id == layer as u32)
}

/// The whole source image's rectangle on screen for one layer. Every
/// normalised handle is placed inside it.
fn layer_image_rect(state: &SurfaceState, layer: i32) -> Option<PreviewSurfaceRect> {
  let mut selection = layer_selection(state, layer)?;
  selection.x = selection.image_x;
  selection.y = selection.image_y;
  selection.width = selection.image_width;
  selection.height = selection.image_height;
  let rect = display_selection(state, selection)?;
  (rect.width > 0.0 && rect.height > 0.0).then_some(rect)
}

/// The image one arrow's grips and its halo are placed in. An annotation
/// belongs to its own layer, which is not always the selected one: the keyboard
/// shortcut can hold the selection while the pointer rests on an arrow over the
/// screen.
pub(super) fn item_image_frame(state: &SurfaceState, index: i32) -> Option<PreviewSurfaceRect> {
  let layer = usize::try_from(index)
    .ok()
    .and_then(|index| state.annotation.handles.get(index))
    .map_or(-1, |item| item.layer_id);
  layer_image_rect(state, layer)
}

/// The image the selected arrow's grips, and the snap chrome, are placed in.
pub(super) fn image_frame(state: &SurfaceState) -> Option<PreviewSurfaceRect> {
  item_image_frame(state, state.annotation.selected)
}

pub(super) fn selected_item(state: &SurfaceState) -> Option<&NativeAnnotationHandles> {
  usize::try_from(state.annotation.selected)
    .ok()
    .and_then(|index| state.annotation.handles.get(index))
}

fn display_point(image: PreviewSurfaceRect, x: f64, y: f64) -> (f64, f64) {
  (image.x + image.width * x, image.y + image.height * y)
}

/// A point on screen, back in the layer's image-normalised space. This is
/// exactly what Rust above the facade turns into source pixels.
pub(super) fn normalised_point(state: &SurfaceState, point: (f64, f64)) -> Option<(f64, f64)> {
  let image = image_frame(state)?;
  Some((
    (point.0 - image.x) / image.width,
    (point.1 - image.y) / image.height,
  ))
}

/// How wide the chosen annotation's picture is drawn, in display points. The
/// twin of the `image.size.width` the macOS view reports beside each sample,
/// and what turns a snap's reach in points into source pixels.
pub(super) fn image_extent(state: &SurfaceState) -> Option<f64> {
  Some(image_frame(state)?.width)
}

/// The grip of one arrow under `point`, in display points. Pure so the hit
/// box can be tested without a composition device.
fn grip_at_point(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<u32> {
  if item.shape_kind() == AnnotationKind::Redact {
    return super::redact_chrome::grip_at(image, item, point);
  }
  let grips = item_grips(image, item);
  let found = grips
    .iter()
    .position(|grip| {
      (point.0 - grip.0).abs() <= HANDLE_HIT && (point.1 - grip.1).abs() <= HANDLE_HIT
    })
    .map(|index| index as u32)?;
  Some(match item.shape_kind() {
    // A counter and a text box have one grip - the tip of the tail or the
    // pointer - rather than an arrow's three, so the grip it reports is the
    // tail rather than the start.
    AnnotationKind::Counter | AnnotationKind::Text => HANDLE_TAIL,
    AnnotationKind::Arrow | AnnotationKind::Redact => found,
  })
}

/// A text box prepared in display points, from the grips it was published
/// with. The twin of the text branch of `annotation_prepared`.
pub(super) fn text_geometry(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> ArrowGeometry {
  let origin = display_point(image, item.start_x, item.start_y);
  prepare_text(
    [origin.0 as f32, origin.1 as f32],
    [item.end_x as f32, item.end_y as f32],
    [
      (item.middle_x * image.width) as f32,
      (item.middle_y * image.width) as f32,
    ],
    (item.width * image.width) as f32,
    item.start_head as u32 & 3,
    AnnotationReveal::WHOLE,
  )
}

/// The grips one annotation shows as discs, in display points: an arrow's
/// three, and the tip of a counter's tail or of a text box's pointer. A
/// redaction's are the selection box's own, which `redact_chrome` draws and
/// hits. The tail is placed here rather than sent because a normalised
/// offset is a different length in each axis on a picture that is not square,
/// while display points are isotropic.
pub(super) fn item_grips(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> Vec<(f64, f64)> {
  let centre = display_point(image, item.start_x, item.start_y);
  match item.shape_kind() {
    AnnotationKind::Counter => {
      let radius = item.width * image.width / 2.0;
      let tip = crate::editor::annotations::counter::silhouette::counter_tail_tip(
        AnnotationPoint {
          x: centre.0,
          y: centre.1,
        },
        radius,
        item.start_head,
      );
      vec![(tip.x, tip.y)]
    }
    // The pointer's tip, drawn out of the box or tucked into it where it was
    // left.
    AnnotationKind::Text => {
      let geometry = text_geometry(image, item);
      vec![(f64::from(geometry.c[0]), f64::from(geometry.c[1]))]
    }
    AnnotationKind::Redact => Vec::new(),
    AnnotationKind::Arrow => vec![
      centre,
      display_point(image, item.middle_x, item.middle_y),
      display_point(image, item.end_x, item.end_y),
    ],
  }
}

/// The chosen arrow's grip under `point`.
pub(super) fn handle_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<u32> {
  let item = *selected_item(state)?;
  grip_at_point(image_frame(state)?, &item, point)
}

/// How far a point is from one arrow's drawn shape, in display points: its
/// shaft, and the heads on it. Zero anywhere the arrow is actually painted,
/// because the tolerance is measured from the stroke's edge rather than its
/// centreline. A press on a head is a press on the arrow - it is the part of it
/// the hand aims at. An annotation half-way through drawing itself in is still
/// picked by the whole of what it will be.
fn arrow_distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  let start = display_point(image, item.start_x, item.start_y);
  match item.shape_kind() {
    AnnotationKind::Counter => {
      return counter_distance(
        point,
        AnnotationPoint {
          x: start.0,
          y: start.1,
        },
        item.width * image.width / 2.0,
        item.start_head,
      ) as f32;
    }
    AnnotationKind::Text => {
      return text_distance(
        [point.0 as f32, point.1 as f32],
        &text_geometry(image, item),
      );
    }
    AnnotationKind::Redact => return super::redact_chrome::distance(image, item, point),
    AnnotationKind::Arrow => {}
  }
  let middle = display_point(image, item.middle_x, item.middle_y);
  let end = display_point(image, item.end_x, item.end_y);
  // The control point behind a reported middle handle, so the shaft can be
  // sampled without solving the curve again.
  let a = [start.0 as f32, start.1 as f32];
  let b = [
    (2.0 * middle.0 - (start.0 + end.0) / 2.0) as f32,
    (2.0 * middle.1 - (start.1 + end.1) / 2.0) as f32,
  ];
  let c = [end.0 as f32, end.1 as f32];
  let probe = [point.0 as f32, point.1 as f32];
  // The stroke rides on the annotation rather than being read back off its
  // heads, so a headless arrow is picked over the width it shows too.
  let width = (item.width * image.width) as f32;
  let heads = if item.start_head > 0.0 {
    2
  } else if item.end_head > 0.0 {
    1
  } else {
    0
  };
  let geometry = prepare_arrow(a, b, c, width, heads, AnnotationReveal::WHOLE);
  prepared_arrow_distance(probe, &geometry)
}

/// The topmost arrow whose drawn shape `point` lands on. There is no
/// tolerance around it: the arrow is picked, and haloed, exactly where it is
/// painted, which is what keeps the halo off the space beside an annotation.
pub(super) fn shaft_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<usize> {
  state
    .annotation
    .handles
    .iter()
    .enumerate()
    .rev()
    .find(|(_, item)| {
      layer_image_rect(state, item.layer_id)
        .is_some_and(|image| arrow_distance(image, item, point) <= 0.0)
    })
    .map(|(index, _)| index)
}

#[cfg(test)]
#[path = "picking_tests.rs"]
mod tests;
