// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rotation gizmo over a layer a scene turns, and that layer's chrome
//! carried through its turn: the twin of the macOS overlay's `+turn.m`. The
//! chrome is laid out as for the upright layer, in display points, and carried
//! on through the turn the compositor draws the layer with; a press on it is
//! carried back the same way. The geometry is `screenwide_osc::turn`'s and
//! `screenwide_scenes::rotation`'s, so the rings, the chrome and the picture
//! cannot drift apart.

use super::*;
use screenwide_annotations::gesture::{MODE_NONE, MODE_SELECT};
use screenwide_osc::turn::{ring_at, ring_radius, RingDrag, RingHit};
use screenwide_scenes::rotation::{apply, inverted, turned_homography, Homography};
use screenwide_scenes::SceneRotation;

/// A press, drag and release of a ring.
mod drag;
pub(super) use drag::{cancel, down, pointer_move, up};

/// The turned frame, the rings, and the annotation chrome's geometry placed
/// through the turn.
mod chrome;
pub(super) use chrome::{
  carried_box, carried_point, carried_rect, carried_segment, chrome, TurnChrome, RING_KIND,
};

/// A ring held: the drag, the selection as the press found it, where the
/// press landed, and the resize edges whose cursor runs the way it turns
/// there.
#[derive(Clone, Copy)]
pub(super) struct TurnDrag {
  drag: RingDrag,
  start: PreviewSelection,
  origin: (f64, f64),
  pub(super) edges: u32,
}

/// The gizmo's pointer state: the ring held, a layer's or the chosen
/// image's, and the ring the pointer rests on, which is lit. Only one gizmo
/// shows at a time, so they share the lit ring.
#[derive(Default)]
pub(super) struct TurnState {
  pub(super) drag: Option<TurnDrag>,
  pub(super) image: Option<annotation::ImageTurnDrag>,
  pub(super) hovered: Option<usize>,
}

fn rotation(selection: PreviewSelection) -> SceneRotation {
  SceneRotation {
    x: selection.rotation_x,
    y: selection.rotation_y,
    z: selection.rotation_z,
  }
}

/// Whether `selection` is turned, which is when its chrome is carried.
fn is_turned(selection: PreviewSelection) -> bool {
  selection.turnable != 0 && !rotation(selection).is_upright()
}

/// The turn `selection`'s chrome goes through, in display points, and its
/// inverse. `None` where the layer stands upright, or is seen edge on, which
/// leaves nothing of it to press.
fn homographies(
  state: &SurfaceState,
  selection: PreviewSelection,
) -> Option<(Homography, Homography)> {
  if !is_turned(selection) {
    return None;
  }
  let frame = display_selection(state, selection)?;
  // The turn is seen from the canvas's own distance, so it is measured
  // against the canvas drawn on screen, the pane the layer is laid out in.
  let pane = state
    .workspace_transform
    .apply(state.viewport, selection_pane_rect(state, selection));
  let forward = turned_homography(
    rotation(selection),
    (frame.x + frame.width / 2.0, frame.y + frame.height / 2.0),
    pane.width.max(pane.height),
  );
  Some((forward, inverted(forward)?))
}

/// Where `h` carries `point`, or far off every layer where it lands behind
/// the eye.
fn carry(h: Homography, point: (f64, f64)) -> (f64, f64) {
  apply(h, point).unwrap_or((-1e7, -1e7))
}

/// `point` as it lands on `selection` standing upright: itself where the
/// layer stands upright.
pub(super) fn unturned_point(
  state: &SurfaceState,
  selection: PreviewSelection,
  point: (f64, f64),
) -> (f64, f64) {
  homographies(state, selection).map_or(point, |(_, inverse)| carry(inverse, point))
}

/// `point` as it lands on the selected layer standing upright.
pub(super) fn selected_layer_point(state: &SurfaceState, point: (f64, f64)) -> (f64, f64) {
  state
    .selection
    .map_or(point, |selection| unturned_point(state, selection, point))
}

/// Whether `selection`'s chrome is drawn and pressed through a turn.
pub(super) fn selection_is_turned(state: &SurfaceState, selection: PreviewSelection) -> bool {
  homographies(state, selection).is_some()
}

/// The selected layer's turn, and what carries its upright chrome through
/// it, where it is turned.
pub(super) fn layer_turn(state: &SurfaceState) -> Option<(SceneRotation, Homography)> {
  let selection = state.selection?;
  homographies(state, selection).map(|(forward, _)| (rotation(selection), forward))
}

/// `point` as it lands on the selected layer standing upright, read under the
/// surface's lock.
pub(super) fn layer_point(inner: &SurfaceInner, point: (f64, f64)) -> (f64, f64) {
  inner
    .state
    .lock()
    .map_or(point, |state| selected_layer_point(&state, point))
}

/// Whether the gizmo's rings are shown: the select tool's chrome over a
/// layer a scene turns.
fn rings_shown(state: &SurfaceState) -> bool {
  state.editor_active
    && state.selection_visible
    && state
      .selection
      .is_some_and(|selection| selection.turnable != 0 && selection.crop_mode == 0)
    && matches!(state.annotation.mode, MODE_NONE | MODE_SELECT)
    && !annotation::owns_chrome(state)
    && !state.object_picking.is_active()
}

/// The gizmo's middle, the selected layer's, which its turn leaves in place,
/// and the rings' radius, in display points.
fn ring_frame(state: &SurfaceState) -> Option<((f64, f64), f64)> {
  let frame = display_selection(state, state.selection?)?;
  Some((
    (frame.x + frame.width / 2.0, frame.y + frame.height / 2.0),
    ring_radius(frame.width.min(frame.height)),
  ))
}

/// The ring under `point`, and the rings' radius.
fn ring_hit(state: &SurfaceState, point: (f64, f64)) -> Option<(RingHit, f64)> {
  if !rings_shown(state) {
    return None;
  }
  let ((x, y), radius) = ring_frame(state)?;
  let hit = ring_at(
    rotation(state.selection?),
    radius,
    (point.0 - x, point.1 - y),
  )?;
  Some((hit, radius))
}

/// The cursor over a ring, or the one a held ring keeps: the resize cursor
/// running the way it turns there.
pub(super) fn cursor(state: &SurfaceState, point: (f64, f64)) -> Option<editor::CursorKind> {
  let edges = match (state.turn.drag, state.turn.image) {
    (Some(held), _) => held.edges,
    (None, Some(held)) => held.edges,
    (None, None) => match annotation::image_ring_hit(state, point) {
      Some(hit) => hit.edges,
      None => ring_hit(state, point)?.0.edges,
    },
  };
  Some(resize_cursor(edges))
}

/// Re-reads which ring the pointer rests on, the chosen image's or the
/// layer's, answering whether it changed.
pub(super) fn update_hover(state: &mut SurfaceState, point: (f64, f64)) -> bool {
  let ring = (state.turn.drag.is_none() && state.turn.image.is_none())
    .then(|| {
      annotation::image_ring_hit(state, point)
        .map(|hit| hit.axis)
        .or_else(|| ring_hit(state, point).map(|(hit, _)| hit.axis))
    })
    .flatten();
  let changed = ring != state.turn.hovered;
  state.turn.hovered = ring;
  changed
}

/// Lights the ring the resting pointer is over, redrawing when it changed.
pub(super) fn hover(inner: &SurfaceInner, point: (f64, f64)) {
  if let Ok(mut state) = inner.state.lock() {
    if update_hover(&mut state, point) {
      draw_selection(inner, &state);
    }
  }
}
