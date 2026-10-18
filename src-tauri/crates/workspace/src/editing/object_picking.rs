// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Objects picked in the camera on the workspace itself: a click on the
//! camera's picture picks the object under it and a drag paints a
//! line over one, Alt either way removing it, and the objects picked in the
//! frame shown are drawn over the camera in the selection overlay.

mod chrome;

use std::time::{Duration, Instant};

use super::*;
pub(super) use chrome::{add_objects, ObjectChrome};
use screenwide_compositor::object_picking::{
  ObjectPickCallback, ObjectPress, ObjectsShown, PickedObjectDot,
};

/// How far a press must travel, in points, before it paints a line.
const PAINT_SLOP: f64 = 3.0;
/// How far apart the points kept along a painted line are, in points.
const PAINT_STEP: f64 = 4.0;
/// How long a pick on its way waits to arrive before it goes.
const PENDING: Duration = Duration::from_secs(4);

/// Picking objects in the camera, as the surface holds it.
#[derive(Default)]
pub(super) struct ObjectPicking {
  active: bool,
  /// The selection target the camera's picture is.
  camera_layer: u32,
  objects: Vec<PickedObjectDot>,
  /// The points of the objects' painted lines, in turn.
  lines: Vec<[f32; 2]>,
  /// The pick on its way, drawn chosen until the objects arrive with it.
  pending: Option<Pending>,
  /// A press on the camera's picture, until it is let go.
  press: Option<Press>,
}

impl ObjectPicking {
  /// Whether objects are being picked, which takes every press.
  pub(super) fn is_active(&self) -> bool {
    self.active
  }
}

struct Pending {
  points: Vec<[f32; 2]>,
  removes: bool,
  /// How many objects there were when it was handed over, and when.
  sent: Option<(usize, Instant)>,
}

struct Press {
  origin: (f64, f64),
  last: (f64, f64),
  image: PreviewSurfaceRect,
  removes: bool,
  points: Vec<[f32; 2]>,
  painting: bool,
}

/// Where the camera's whole picture lies on the workspace, as the selection
/// targets place it: its own pane, the recording itself, or baked into the
/// screen. `None` while it is laid out nowhere.
fn camera_image(state: &SurfaceState) -> Option<PreviewSurfaceRect> {
  let layer = state.object_picking.camera_layer;
  let target = state
    .selection_targets
    .iter()
    .find(|target| target.layer_id == layer)?;
  let pane = state.panes.get(target.pane_index as usize)?.as_ref()?;
  let pane = state.workspace_transform.apply(
    state.viewport,
    pane_canvas_rect(pane, state.frame_resize.is_some()),
  );
  let image = PreviewSurfaceRect {
    x: pane.x + target.image_x * pane.width,
    y: pane.y + target.image_y * pane.height,
    width: target.image_width * pane.width,
    height: target.image_height * pane.height,
  };
  (image.width > 0.0 && image.height > 0.0).then_some(image)
}

fn inside(image: PreviewSurfaceRect, point: (f64, f64)) -> bool {
  (image.x..image.x + image.width).contains(&point.0)
    && (image.y..image.y + image.height).contains(&point.1)
}

/// `point` on the workspace, 0 to 1 across and down the camera's picture,
/// kept inside it.
fn picture_point(image: PreviewSurfaceRect, point: (f64, f64)) -> [f32; 2] {
  [
    ((point.0 - image.x) / image.width).clamp(0.0, 1.0) as f32,
    ((point.1 - image.y) / image.height).clamp(0.0, 1.0) as f32,
  ]
}

/// A press while objects are picked is theirs: on the camera's picture it
/// begins a pick, elsewhere it pans. False while they are not.
pub(super) fn down(inner: &SurfaceInner, point: (f64, f64), removes: bool) -> bool {
  let Ok(mut state) = inner.state.lock() else {
    return false;
  };
  if !state.object_picking.active {
    return false;
  }
  state.last_pointer = point;
  match camera_image(&state).filter(|image| inside(*image, point)) {
    Some(image) => {
      state.gesture = None;
      state.object_picking.press = Some(Press {
        origin: point,
        last: point,
        image,
        removes,
        points: vec![picture_point(image, point)],
        painting: false,
      });
    }
    None => {
      state.gesture = Some(ActiveGesture::Pan {
        pointer_start: point,
        transform_start: state.workspace_transform,
      });
    }
  }
  true
}

/// A press that travels over the camera paints a line through where it
/// goes, drawn as it is painted. False with no press of its own.
pub(super) fn pointer_move(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let Ok(mut state) = inner.state.lock() else {
    return false;
  };
  let state = &mut *state;
  let Some(press) = state.object_picking.press.as_mut() else {
    return false;
  };
  let travelled = |from: (f64, f64)| (point.0 - from.0).hypot(point.1 - from.1);
  if !press.painting && travelled(press.origin) < PAINT_SLOP {
    return true;
  }
  if press.painting && travelled(press.last) < PAINT_STEP {
    return true;
  }
  press.painting = true;
  press.last = point;
  press.points.push(picture_point(press.image, point));
  state.object_picking.pending = Some(Pending {
    points: press.points.clone(),
    removes: press.removes,
    sent: None,
  });
  draw_selection(inner, state);
  true
}

/// The press let go: a click or a line painted, picked. False with no press
/// of its own.
pub(super) fn up(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  let press = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let state = &mut *state;
    let Some(mut press) = state.object_picking.press.take() else {
      return false;
    };
    if press.painting && point != press.last {
      press.points.push(picture_point(press.image, point));
    }
    state.object_picking.pending = Some(Pending {
      points: press.points.clone(),
      removes: press.removes,
      sent: Some((state.object_picking.objects.len(), Instant::now())),
    });
    draw_selection(inner, state);
    press
  };
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.object_pick.as_mut() {
      callback(ObjectPress {
        points: press.points,
        keep: !press.removes,
      });
    }
  }
  true
}

/// A double-click on the camera's picture is the one pick its first press
/// made; elsewhere it fits the view as ever. True where it is taken.
pub(super) fn double_click(inner: &SurfaceInner, point: (f64, f64)) -> bool {
  inner.state.lock().is_ok_and(|state| {
    state.object_picking.active && camera_image(&state).is_some_and(|image| inside(image, point))
  })
}

/// What the overlay draws of the objects picked while they are, in its own
/// logical points.
pub(super) fn chrome(state: &SurfaceState) -> Option<ObjectChrome> {
  let picking = &state.object_picking;
  if !picking.active {
    return None;
  }
  let image = camera_image(state)?;
  let at = |point: [f32; 2]| screenwide_osc::geometry::Point {
    x: image.x + f64::from(point[0]) * image.width,
    y: image.y + f64::from(point[1]) * image.height,
  };
  // A pick on its way is the one chosen until it arrives.
  let pending = picking.pending.as_ref();
  let mut objects = picking
    .objects
    .iter()
    .map(|object| {
      let line = picking
        .lines
        .get(object.line_first as usize..(object.line_first + object.line_points) as usize)
        .filter(|line| !line.is_empty());
      chrome::Object {
        points: line.map_or_else(
          || vec![at([object.x, object.y])],
          |line| line.iter().copied().map(at).collect(),
        ),
        removed: object.removed != 0,
        chosen: object.chosen != 0 && pending.is_none(),
      }
    })
    .collect::<Vec<_>>();
  objects.extend(pending.map(|pending| chrome::Object {
    points: pending.points.iter().copied().map(at).collect(),
    removed: pending.removes,
    chosen: true,
  }));
  Some(ObjectChrome { objects })
}

/// Whether `objects`, with their `lines`, hold the pick on its way: one
/// starting where it starts, with as many points.
fn arrived(pending: &Pending, objects: &[PickedObjectDot], lines: &[[f32; 2]]) -> bool {
  let Some(start) = pending.points.first() else {
    return false;
  };
  objects.iter().any(|object| {
    let line = lines
      .get(object.line_first as usize..(object.line_first + object.line_points) as usize)
      .filter(|line| !line.is_empty());
    let (first, count) = line.map_or(([object.x, object.y], 1), |line| (line[0], line.len()));
    count == pending.points.len() && first == *start
  })
}

impl WorkspaceEditor {
  /// Picks objects in the camera as `picking` shows them, or stops with
  /// `None`. While picking, a press on the camera is handed to the pick
  /// callback rather than choosing or moving a layer.
  pub fn set_object_picking(&self, picking: Option<ObjectsShown<'_>>) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    let state = &mut *state;
    let shown = &mut state.object_picking;
    match picking {
      Some((camera_layer, objects, lines)) => {
        // The pick has arrived once the objects come back with one more, or
        // with one where it was made, or gone a while without.
        let landed = shown.pending.as_ref().is_some_and(|pending| {
          pending.sent.is_some_and(|(count, at)| {
            objects.len() > count || at.elapsed() > PENDING || arrived(pending, objects, lines)
          })
        });
        if landed {
          shown.pending = None;
        }
        shown.active = true;
        shown.camera_layer = camera_layer;
        shown.objects = objects.to_vec();
        shown.lines = lines.to_vec();
      }
      None => *shown = ObjectPicking::default(),
    }
    draw_selection(&self.inner, state);
  }

  /// Installs the callback a press on the camera is handed to while objects
  /// are picked.
  pub fn set_object_pick_callback(&self, callback: ObjectPickCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.object_pick = Some(callback);
    }
  }
}
