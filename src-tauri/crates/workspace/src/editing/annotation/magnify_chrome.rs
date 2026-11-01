// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lenses in the chrome: where a lens is picked, and seeing through lenses
//! to the annotations under them. A lens is held by its box, as a shape is;
//! what lies under it on its picture is chosen, gripped and carried where
//! it shows enlarged, so the grips and the presses are read through every
//! lens over the annotation (`magnify::lens`).

use super::picking::display_point;
use super::*;
use screenwide_annotations::magnify::geometry::{magnify_distance, prepare_magnify};
use screenwide_annotations::magnify::lens::{shown_at, Lens, LensView};
use screenwide_annotations::reveal::AnnotationReveal;

/// How far `point` is from a lens, in display points: zero or less anywhere
/// on it.
pub(super) fn distance(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> f32 {
  let low = display_point(image, item.start_x, item.start_y);
  let high = display_point(image, item.end_x, item.end_y);
  let geometry = prepare_magnify(
    [low.0 as f32, low.1 as f32],
    [high.0 as f32, high.1 as f32],
    item.end_head as f32,
    item.start_head as f32,
    0.0,
    AnnotationReveal::WHOLE,
  );
  magnify_distance([point.0 as f32, point.1 as f32], &geometry)
}

/// The lens the annotation at `index` is, in display points on its own
/// picture standing upright, from the grips it was published with: its box
/// in `start` and `end`, its radius in `start_head` and its zoom in
/// `end_head`. `None` for anything else.
pub(super) fn lens_of(state: &SurfaceState, index: usize) -> Option<Lens> {
  let item = state.annotation.handles.get(index)?;
  if item.shape_kind() != AnnotationKind::Magnify {
    return None;
  }
  let image = super::picking::item_image_frame(state, index as i32)?;
  let low = display_point(image, item.start_x, item.start_y);
  let high = display_point(image, item.end_x, item.end_y);
  Some(Lens::from_box(
    [low.0, low.1],
    [high.0, high.1],
    item.start_head,
    item.end_head,
  ))
}

/// Whether the annotation at `index` lies under the lens at `lens`: on the
/// same picture, which is all a lens shows its own annotations of.
pub(super) fn under(state: &SurfaceState, lens: usize, index: usize) -> bool {
  let handles = &state.annotation.handles;
  index < lens
    && handles
      .get(lens)
      .zip(handles.get(index))
      .is_some_and(|(lens, item)| lens.layer_id == item.layer_id)
}

/// Where `point`, of the annotation at `index` on its own picture, shows
/// through the lenses over it, and the view a press there is read back
/// through; `None` where a lens covers it and shows something else there.
pub(super) fn shown(
  state: &SurfaceState,
  index: usize,
  point: (f64, f64),
) -> Option<((f64, f64), LensView)> {
  let above = (index + 1..state.annotation.handles.len())
    .filter(|&lens| under(state, lens, index))
    .filter_map(|lens| lens_of(state, lens));
  shown_at([point.0, point.1], above).map(|([x, y], view)| ((x, y), view))
}

/// The view the box at `index` is seen through whole: the one its middle
/// shows through, so a box seen through a lens is still one box, enlarged,
/// rather than one broken at the lens's edge. None where no lens shows its
/// middle.
pub(super) fn whole_view(
  state: &SurfaceState,
  index: usize,
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
) -> LensView {
  let middle = display_point(
    image,
    (item.start_x + item.end_x) / 2.0,
    (item.start_y + item.end_y) / 2.0,
  );
  shown(state, index, middle).map_or(LensView::IDENTITY, |(_, view)| view)
}

/// Where the grip at `grip`, of the annotation at `index`, shows, and the
/// view a drag of it reads back through. One a lens's rim covers is drawn
/// where it lies, over the lens, so it can still be taken hold of.
pub(super) fn grip_shown(
  state: &SurfaceState,
  index: usize,
  grip: (f64, f64),
) -> ((f64, f64), LensView) {
  shown(state, index, grip).unwrap_or((grip, LensView::IDENTITY))
}

/// The grip of the annotation at `index` under `point`, read where its grips
/// show through the lenses over it, and the view a drag of it reads back
/// through.
pub(super) fn grip_through(
  state: &SurfaceState,
  index: usize,
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<(u32, LensView)> {
  super::picking::grip_seen(
    image,
    item,
    point,
    whole_view(state, index, image, item),
    |grip| grip_shown(state, index, grip),
  )
}
