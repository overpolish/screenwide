// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rotation gizmo over the chosen image: its rings, and a drag of them
//! that turns the image in depth and flat.
//!
//! The rings show the turn the image is seen at. On a layer that is turned
//! itself, that is the image's own turn within the layer's, centred where
//! the layer's turn carries the image's middle; a drag works the image's own
//! turn back out of the turn the rings reach. Each sample is reported as the
//! annotation gesture's turn target, which carries that turn in place of a
//! point.

use super::*;
use screenwide_annotations::gesture::{MODE_IMAGE, MODE_SELECT};
use screenwide_osc::geometry::Point;
use screenwide_osc::turn::{composed, ring_at, ring_radius, within, RingDrag, RingHit};
use screenwide_scenes::SceneRotation;
/// A drag of the rings round the image at `index`, matching
/// `ScreenwideAnnotationTargetTurn`.
const TARGET_TURN: u32 = 7;

/// A ring of the chosen image's gizmo held: the drag, the image's place
/// among the published handles, its own turn and the turn the rings show as
/// the last sample left them, where the press landed, and the resize edges
/// whose cursor runs the way the ring turns there.
#[derive(Clone, Copy)]
pub(in super::super) struct ImageTurnDrag {
  drag: RingDrag,
  item: usize,
  own: SceneRotation,
  seen: SceneRotation,
  origin: (f64, f64),
  pub(in super::super) edges: u32,
}

impl ImageTurnDrag {
  pub(in super::super) fn axis(&self) -> usize {
    self.drag.axis as usize
  }
}

/// The chosen image's gizmo: the rings' middle and radius in display
/// points, the turn they show, the image's own turn and its place among the
/// published handles.
pub(in super::super) struct ImageGizmo {
  pub(in super::super) centre: (f64, f64),
  pub(in super::super) radius: f64,
  pub(in super::super) seen: SceneRotation,
  own: SceneRotation,
  item: usize,
}

/// An image's own turn as its handles publish it: its tilt, and its flat
/// turn, which runs counter-clockwise as its angle runs clockwise.
fn own_turn(item: &NativeAnnotationHandles) -> SceneRotation {
  SceneRotation {
    x: item.tilt_x,
    y: item.tilt_y,
    z: -item.start_head.to_degrees(),
  }
}

/// The gizmo over the chosen image, where its rings are shown: one image
/// chosen alone, with the select or image tool in hand.
pub(in super::super) fn gizmo(state: &SurfaceState) -> Option<ImageGizmo> {
  if !state.editor_active
    || !state.selection_visible
    || !matches!(state.annotation.mode, MODE_SELECT | MODE_IMAGE)
    || has_group(state)
    || state.object_picking.is_active()
  {
    return None;
  }
  let index = usize::try_from(state.annotation.selected).ok()?;
  let item = state.annotation.handles.get(index)?;
  if item.shape_kind() != AnnotationKind::Image {
    return None;
  }
  let image = image_frame(state)?;
  let (x, y) = super::picking::display_point(image, item.middle_x, item.middle_y);
  // The half extents ride as shares of the drawn width.
  let short = 2.0 * item.end_head.min(item.width) * image.width;
  let layer = super::super::turn::layer_turn(state);
  let (own, seen) = match state.turn.image.filter(|held| held.item == index) {
    Some(held) => (held.own, held.seen),
    None => {
      let own = own_turn(item);
      (own, layer.map_or(own, |(turn, _)| composed(turn, own)))
    }
  };
  let centre = layer.map_or((x, y), |(_, forward)| {
    let carried = super::super::turn::carried_point(forward, Point { x, y });
    (carried.x, carried.y)
  });
  Some(ImageGizmo {
    centre,
    radius: ring_radius(short),
    seen,
    own,
    item: index,
  })
}

/// The ring of the chosen image's gizmo under `point`.
pub(in super::super) fn ring_hit(state: &SurfaceState, point: (f64, f64)) -> Option<RingHit> {
  let gizmo = gizmo(state)?;
  ring_at(
    gizmo.seen,
    gizmo.radius,
    (point.0 - gizmo.centre.0, point.1 - gizmo.centre.1),
  )
}

/// A ring drag's sample: the layer and index the image is addressed by, and
/// its own turn.
struct TurnSample {
  phase: SelectionGesturePhase,
  layer: u32,
  index: u32,
  own: SceneRotation,
}

/// Resolves a sample of `held` under the surface lock.
fn resolve(
  state: &SurfaceState,
  phase: SelectionGesturePhase,
  held: &ImageTurnDrag,
) -> Option<TurnSample> {
  let item = state.annotation.handles.get(held.item)?;
  // An annotation is measured on its own layer; a still's carry none, and
  // the selection's is theirs.
  let layer = u32::try_from(item.layer_id)
    .ok()
    .or_else(|| state.selection.map(|selection| selection.layer_id))?;
  Some(TurnSample {
    phase,
    layer,
    index: item.index,
    own: held.own,
  })
}

/// Reports a sample: the turn about X and Y as the point, about Z as the
/// picture's width. MUST be called with no surface state held: the callback
/// re-enters the surface to publish what it changed.
fn report(inner: &SurfaceInner, sample: Option<TurnSample>) {
  let Some(sample) = sample else {
    return;
  };
  let Ok(mut callbacks) = inner.callbacks.lock() else {
    return;
  };
  if let Some(callback) = callbacks.annotation_gesture.as_mut() {
    callback(
      sample.phase,
      sample.layer,
      TARGET_TURN,
      sample.index,
      0,
      sample.own.x,
      sample.own.y,
      0,
      sample.own.z,
    );
  }
}

/// Takes a press on a ring of the chosen image's gizmo.
pub(in super::super) fn down(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let sample = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(gizmo) = gizmo(&state) else {
      return false;
    };
    let offset = (point.0 - gizmo.centre.0, point.1 - gizmo.centre.1);
    let Some(hit) = ring_at(gizmo.seen, gizmo.radius, offset) else {
      return false;
    };
    let held = ImageTurnDrag {
      drag: RingDrag::begin(gizmo.seen, gizmo.radius, hit.axis, hit.angle),
      item: gizmo.item,
      own: gizmo.own,
      seen: gizmo.seen,
      origin: point,
      edges: hit.edges,
    };
    state.turn.image = Some(held);
    state.turn.hovered = None;
    state.last_pointer = point;
    draw_selection(inner, &state);
    resolve(&state, SelectionGesturePhase::Begin, &held)
  };
  report(inner, sample);
  refresh_cursor_for(inner);
  true
}

/// Turns the held ring as far as `point` has carried it from the press.
pub(in super::super) fn pointer_move(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let sample = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(mut held) = state.turn.image else {
      return false;
    };
    state.last_pointer = point;
    let image = state.annotation.handles.get(held.item);
    if image.is_none_or(|item| item.shape_kind() != AnnotationKind::Image) {
      // The image went away under the ring: the turn it reached goes with it.
      state.turn.image = None;
      draw_selection(inner, &state);
      let sample = Some(TurnSample {
        phase: SelectionGesturePhase::Cancel,
        layer: state.selection.map_or(0, |selection| selection.layer_id),
        index: 0,
        own: held.own,
      });
      drop(state);
      report(inner, sample);
      return true;
    }
    let seen = held.drag.turned(
      (point.0 - held.origin.0, point.1 - held.origin.1),
      state.modifiers.shift,
    );
    held.own = match super::super::turn::layer_turn(&state) {
      Some((turn, _)) => within(turn, seen, held.own),
      None => seen,
    };
    held.seen = seen;
    state.turn.image = Some(held);
    draw_selection(inner, &state);
    resolve(&state, SelectionGesturePhase::Update, &held)
  };
  report(inner, sample);
  true
}

/// Lets go of the held ring, keeping the turn it reached.
pub(in super::super) fn up(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  // A release can land past the last drag sample; it is the turn kept.
  if !pointer_move(inner, point) {
    return false;
  }
  let sample = inner.state.lock().ok().and_then(|mut state| {
    let held = state.turn.image.take()?;
    let sample = resolve(&state, SelectionGesturePhase::End, &held);
    super::super::turn::update_hover(&mut state, point);
    draw_selection(inner, &state);
    sample
  });
  report(inner, sample);
  refresh_cursor_for(inner);
  true
}

/// Lets go of the held ring without keeping the turn it reached.
pub(in super::super) fn cancel(inner: &SurfaceInner) -> bool {
  let sample = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(held) = state.turn.image.take() else {
      return false;
    };
    draw_selection(inner, &state);
    resolve(&state, SelectionGesturePhase::Cancel, &held)
  };
  report(inner, sample);
  refresh_cursor_for(inner);
  true
}
