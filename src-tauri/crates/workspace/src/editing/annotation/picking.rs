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
use screenwide_annotations::gesture::{drawing_layer, hidden_by_layer_above};
use screenwide_annotations::magnify::lens::{pick_through, LensView};

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

/// Every laid-out picture, back to front, with its upright image rect and
/// `display` as it lands on it standing upright, for [`drawing_layer`]. Each
/// picture is read in its own space: a turned picture is pressed where its
/// turn draws it, and the picture over it, upright, where it stands.
pub(in super::super) fn drawing_pictures(
  state: &SurfaceState,
  display: (f64, f64),
) -> Vec<(u32, [f64; 4], (f64, f64))> {
  state
    .selection_targets
    .iter()
    .filter_map(|target| {
      let selection = layer_selection(state, target.layer_id as i32)?;
      let rect = layer_image_rect(state, target.layer_id as i32)?;
      let point = super::super::turn::unturned_point(state, selection, display);
      Some((
        target.layer_id,
        [rect.x, rect.y, rect.width, rect.height],
        point,
      ))
    })
    .collect()
}

/// Puts the picture a drawing press at `display` joins in hand, as a press
/// with the select tool would, so the fresh annotation joins it and the panel
/// and the chrome follow: the topmost laid-out picture under the press, or
/// the nearest one off every picture, by the rule [`drawing_layer`] keeps.
/// With no picture laid out the selected layer keeps the annotation. Answers
/// the layer to report where the choice changed.
pub(in super::super) fn take_drawing_layer(
  inner: &SurfaceInner,
  state: &mut SurfaceState,
  display: (f64, f64),
) -> Option<u32> {
  let layer = drawing_layer(drawing_pictures(state, display).into_iter())?;
  let target = layer_selection(state, layer as i32)?;
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

pub(super) fn selected_item(state: &SurfaceState) -> Option<&NativeAnnotationHandles> {
  usize::try_from(state.annotation.selected)
    .ok()
    .and_then(|index| state.annotation.handles.get(index))
}

pub(super) fn display_point(image: PreviewSurfaceRect, x: f64, y: f64) -> (f64, f64) {
  (image.x + image.width * x, image.y + image.height * y)
}

/// The grips an annotation is held by, read where they show.
mod grips;
pub(super) use grips::{grip_seen, highlight_flow, item_grips};

/// The chosen annotation's grip under `point`, read on the selected layer.
pub(super) fn handle_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<u32> {
  grip_pick(state, point).map(|(handle, _)| handle)
}

/// The chosen annotation's grip under `point`, where it shows: through the
/// lenses over it, which a drag of it is read back through.
pub(super) fn grip_pick(state: &SurfaceState, point: (f64, f64)) -> Option<(u32, LensView)> {
  let index = usize::try_from(state.annotation.selected).ok()?;
  let item = *selected_item(state)?;
  super::magnify_chrome::grip_through(
    state,
    index,
    image_frame(state)?,
    &item,
    point_on_item_layer(state, &item, point),
  )
}

/// What a press at `point` takes hold of: the chosen annotation's grip, or
/// else the annotation drawn there, each with the lenses it is seen through.
/// The pen takes hold of the chosen annotation's grips alone, which show
/// as something to take hold of; anywhere else its press draws.
pub(super) fn press_pick(state: &SurfaceState, point: (f64, f64)) -> PressPick {
  if let Some(grip) = grip_pick(state, point) {
    return (Some(grip), None);
  }
  let picked = picks_annotations(state.annotation.mode)
    .then(|| pick_at_point(state, point))
    .flatten();
  (None, picked)
}

/// A grip and an annotation a press took hold of; at most one is set.
pub(super) type PressPick = (Option<(u32, LensView)>, Option<(usize, LensView)>);

/// A press read on the layer an annotation belongs to.
mod item_layer;
pub(super) use item_layer::display_on_item_layer;
use item_layer::{drawn_pictures, point_on_item_layer};

/// The topmost arrow whose drawn shape `point` lands on. There is no
/// tolerance around it: the arrow is picked, and haloed, exactly where it is
/// painted, which is what keeps the halo off the space beside an annotation.
/// A shape grabbed by its inside is `redact_chrome::grabs_inside`'s
/// call, and what is drawn wins over an inside, whatever the stacking: a
/// press on an annotation seen through a shape in front picks that
/// annotation, and only a press on nothing drawn falls to the inside it
/// lands in. An annotation a picture in front hides at the press is not
/// seen there, so it is not picked: the press falls to the picture in front.
///
/// Every tool that picks (`picks_annotations`) picks every kind: a press on
/// an annotation takes hold of it whatever is in hand, and only a press on
/// nothing drawn draws. The pen picks nothing, so its strokes always draw;
/// a context menu still finds what is under it here, whatever the tool.
pub(super) fn shaft_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<usize> {
  pick_at_point(state, point).map(|(index, _)| index)
}

/// What [`shaft_at_point`] picks, and the lenses it is seen through there,
/// which a press on it is read back through. A lens is pressed through: what
/// it shows drawn at the press is picked where it shows, and only a press on
/// nothing drawn there takes the lens (`magnify::lens::pick_through`).
pub(super) fn pick_at_point(state: &SurfaceState, point: (f64, f64)) -> Option<(usize, LensView)> {
  let chrome = super::redact_chrome::grabs_inside;
  let pictures = drawn_pictures(state, point);
  let seen = |layer: i32| {
    u32::try_from(layer).map_or(true, |layer| {
      !hidden_by_layer_above(pictures.iter().copied(), layer)
    })
  };
  let handles = &state.annotation.handles;
  let on_item = |index: usize| {
    let (x, y) = point_on_item_layer(state, &handles[index], point);
    [x, y]
  };
  pick_through(
    handles.len(),
    on_item,
    |index| {
      seen(handles[index].layer_id)
        .then(|| super::magnify_chrome::lens_of(state, index))
        .flatten()
    },
    |lens, index| super::magnify_chrome::under(state, lens, index),
    |index, [x, y], insides| {
      let item = &handles[index];
      if insides && !chrome(state, index, item) {
        return false;
      }
      let drawn = layer_image_rect(state, item.layer_id).is_some_and(|image| {
        let distance = if item.shape_kind() == AnnotationKind::Draw {
          draw_distance(state, image, item, (x, y))
        } else if insides {
          super::redact_chrome::shape_body_distance(image, item, (x, y))
        } else {
          arrow_distance(image, item, (x, y))
        };
        distance <= 0.0
      });
      drawn && seen(item.layer_id)
    },
  )
  .map(|pick| (pick.index, pick.view))
}

#[cfg(test)]
mod tests;
