// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where an annotation's picture is on screen, and what a press lands on.
//!
//! Annotations are published in their layer's image-normalised space, so
//! picking first has to find that image inside the pane the layer is drawn in.
//! The twin of `recording_preview_annotation_geometry_macos.h` and the layer
//! lookup in `recording_preview_annotation_layers_macos.h`.

use super::picking_distance::{arrow_distance, draw_distance};
use super::*;
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::gesture::{
  drawing_layer, over_a_picture, MODE_DRAW, MODE_MARQUEE, MODE_SELECT,
};
use crate::editor::annotations::highlight::geometry::HighlightFlow;
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::text::geometry::prepare_text;
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
pub(in super::super) fn layer_image_rect(
  state: &SurfaceState,
  layer: i32,
) -> Option<PreviewSurfaceRect> {
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

/// Puts the picture under a drawing press in hand, as a press with the select
/// tool would, so the fresh annotation joins it and the panel and the chrome
/// follow: the topmost laid-out picture under `point`, by the rule
/// [`drawing_layer`] keeps. Over no picture the selected layer keeps the
/// annotation. Answers the layer to report where the choice changed.
pub(in super::super) fn take_drawing_layer(
  inner: &SurfaceInner,
  state: &mut SurfaceState,
  point: (f64, f64),
) -> Option<u32> {
  let pictures = state.selection_targets.iter().filter_map(|target| {
    let rect = layer_image_rect(state, target.layer_id as i32)?;
    Some((target.layer_id, [rect.x, rect.y, rect.width, rect.height]))
  });
  let target = layer_selection(state, drawing_layer(pictures, point)? as i32)?;
  if state.selection.is_some_and(|current| {
    current.pane_index == target.pane_index && current.layer_id == target.layer_id
  }) {
    return None;
  }
  state.selection = Some(target);
  state.annotation.selected = -1;
  clear_selection_snap_guides(state);
  draw_selection(inner, state);
  Some(target.layer_id)
}

/// Whether `point` lands on a laid-out picture a fresh annotation could
/// join, by the rule [`over_a_picture`] keeps.
pub(super) fn on_picture(state: &SurfaceState, point: (f64, f64)) -> bool {
  let pictures: Vec<_> = state
    .selection_targets
    .iter()
    .filter_map(|target| {
      let rect = layer_image_rect(state, target.layer_id as i32)?;
      Some((target.layer_id, [rect.x, rect.y, rect.width, rect.height]))
    })
    .collect();
  over_a_picture(&pictures, point)
}

pub(super) fn selected_item(state: &SurfaceState) -> Option<&NativeAnnotationHandles> {
  usize::try_from(state.annotation.selected)
    .ok()
    .and_then(|index| state.annotation.handles.get(index))
}

pub(super) fn display_point(image: PreviewSurfaceRect, x: f64, y: f64) -> (f64, f64) {
  (image.x + image.width * x, image.y + image.height * y)
}

/// The grip of one arrow under `point`, in display points. Pure so the hit
/// box can be tested without a composition device.
fn grip_at_point(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<u32> {
  if super::redact_chrome::is_box(item.shape_kind()) {
    return super::redact_chrome::grip_at(image, item, point);
  }
  let found = item_grips(image, item)
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
    // A highlight's two grips are the selection's start and end.
    AnnotationKind::Highlight if found == 0 => 0,
    AnnotationKind::Highlight => 2,
    AnnotationKind::Arrow
    | AnnotationKind::Redact
    | AnnotationKind::Shape
    | AnnotationKind::Spotlight
    | AnnotationKind::Draw
    | AnnotationKind::Magnify
    | AnnotationKind::Sticker => found,
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

/// A highlight's flow of bands in display points, from the grips it was
/// published with.
pub(super) fn highlight_flow(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> HighlightFlow {
  crate::editor::annotations::highlight::handles::flow(item, |x, y| {
    let (x, y) = display_point(image, x, y);
    [x as f32, y as f32]
  })
}

/// The grips one annotation shows as discs, in display points: an arrow's
/// three, the tip of a counter's tail or of a text box's pointer, and a
/// magnifier's loupe grip. A redaction's, a shape's and a spotlight's, and a
/// magnifier's zoom area's, are the selection box's own, which `redact_chrome`
/// draws and hits. The tail is placed here rather than sent because a
/// normalised offset is a different length in each axis on a picture that is
/// not square, while display points are isotropic.
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
    AnnotationKind::Redact
    | AnnotationKind::Shape
    | AnnotationKind::Spotlight
    | AnnotationKind::Draw => Vec::new(),
    AnnotationKind::Magnify => vec![super::magnify_chrome::loupe_grip(image, item)],
    // The grip that turns it stands above its top side.
    AnnotationKind::Sticker => vec![super::sticker_chrome::turn_grip(image, item)],
    AnnotationKind::Highlight => {
      let flow = highlight_flow(image, item);
      [flow.start_grip(), flow.end_grip()]
        .map(|[x, y]| (f64::from(x), f64::from(y)))
        .to_vec()
    }
    AnnotationKind::Arrow => vec![
      centre,
      display_point(image, item.middle_x, item.middle_y),
      display_point(image, item.end_x, item.end_y),
    ],
  }
}

/// The chosen arrow's grip under `point`. The pen holds nothing, so it offers
/// none.
pub(super) fn handle_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<u32> {
  if state.annotation.mode == MODE_DRAW {
    return None;
  }
  let item = *selected_item(state)?;
  grip_at_point(image_frame(state)?, &item, point)
}

/// The topmost arrow whose drawn shape `point` lands on. There is no
/// tolerance around it: the arrow is picked, and haloed, exactly where it is
/// painted, which is what keeps the halo off the space beside an annotation.
/// A shape or a stroke grabbed by its inside is `redact_chrome::grabs_inside`'s
/// call, and what is drawn wins over an inside, whatever the stacking: a
/// press on an annotation seen through a shape in front picks that
/// annotation, and only a press on nothing drawn falls to the inside it
/// lands in.
///
/// The pen picks nothing up, so every press under it draws. A stroke is picked
/// by the select tool alone: another drawing tool that chose one would hand
/// over to the pen, which never holds one. The twin of
/// `annotation_shaft_at_point`.
pub(super) fn shaft_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<usize> {
  let mode = state.annotation.mode;
  if mode == MODE_DRAW {
    return None;
  }
  let chrome = super::redact_chrome::grabs_inside;
  let pick = |insides: bool| {
    state
      .annotation
      .handles
      .iter()
      .enumerate()
      .rev()
      .filter(|(_, item)| {
        item.shape_kind() != AnnotationKind::Draw || matches!(mode, MODE_SELECT | MODE_MARQUEE)
      })
      .filter(|(index, item)| !insides || chrome(state, *index, item))
      .find(|(_, item)| {
        layer_image_rect(state, item.layer_id).is_some_and(|image| {
          let distance = if item.shape_kind() == AnnotationKind::Draw {
            draw_distance(state, image, item, point, insides)
          } else if insides {
            super::redact_chrome::shape_body_distance(image, item, point)
          } else {
            arrow_distance(image, item, point)
          };
          distance <= 0.0
        })
      })
      .map(|(index, _)| index)
  };
  pick(false).or_else(|| pick(true))
}

#[cfg(test)]
mod tests;
