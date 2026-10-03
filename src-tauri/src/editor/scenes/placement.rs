// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a scene puts the panes, and how one placement gives way to another.

use super::geometry::Rect;

/// How small a pane that appears or vanishes is drawn at the far end of its
/// fade, as a share of its own size.
const VANISHED_SCALE: f64 = 0.25;

/// Where the panes are, in the canvas's output pixels: the screen's box and
/// the whole screen image behind it by its corner and width, then the
/// camera's box and the camera picture behind it by its centre and width;
/// the screen's and the camera's corner radii in percent of their boxes'
/// shorter sides; how opaque each pane is drawn, zero for a pane the scene
/// hides; and how far the camera is drawn in front of the screen, one in
/// front and zero behind, between the two while a scene changes the order.
/// The image and the picture only ever move and scale, so annotations and the
/// cursor carried on them follow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Placement {
  pub screen: Rect,
  pub image: (f64, f64, f64),
  pub frame: Rect,
  pub camera: (f64, f64, f64),
  pub radius: (f64, f64),
  pub opacity: (f64, f64),
  pub camera_front: f64,
}

/// For a pane whose opacity goes from `from` to `to`, whether it vanishes,
/// drawn at the start's place, or appears, drawn at the end's, and how large
/// it is `t` of the way through; `None` for a pane shown, or hidden,
/// throughout.
fn settling(from: f64, to: f64, t: f64) -> Option<(bool, f64)> {
  if from > 0.0 && to == 0.0 {
    Some((true, 1.0 + (VANISHED_SCALE - 1.0) * t))
  } else if from == 0.0 && to > 0.0 {
    Some((false, VANISHED_SCALE + (1.0 - VANISHED_SCALE) * t))
  } else {
    None
  }
}

/// `point`, a corner or a middle, carried `factor` of the way towards the
/// size it has about `centre`.
fn about(point: (f64, f64), centre: (f64, f64), factor: f64) -> (f64, f64) {
  (
    centre.0 + (point.0 - centre.0) * factor,
    centre.1 + (point.1 - centre.1) * factor,
  )
}

impl Placement {
  /// The placement `t` of the way from this one to `to`. A pane present in
  /// both moves and resizes between them. A pane one of them hides has
  /// nowhere to move to, so it stays at its place in the other, shrinking to
  /// a quarter of its size about its middle as it fades out, or growing from
  /// there as it fades in. A change of order crosses over with the move, so
  /// where the panes overlap one fades through the other.
  pub fn toward(self, to: Self, t: f64) -> Self {
    let mix = |a: f64, b: f64| a + (b - a) * t;
    let rect = |a: Rect, b: Rect| Rect {
      x: mix(a.x, b.x),
      y: mix(a.y, b.y),
      width: mix(a.width, b.width),
      height: mix(a.height, b.height),
    };
    let triple =
      |a: (f64, f64, f64), b: (f64, f64, f64)| (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2));
    let mut next = Self {
      screen: rect(self.screen, to.screen),
      image: triple(self.image, to.image),
      frame: rect(self.frame, to.frame),
      camera: triple(self.camera, to.camera),
      radius: (
        mix(self.radius.0, to.radius.0),
        mix(self.radius.1, to.radius.1),
      ),
      opacity: (
        mix(self.opacity.0, to.opacity.0),
        mix(self.opacity.1, to.opacity.1),
      ),
      camera_front: mix(self.camera_front, to.camera_front),
    };
    if let Some((vanishes, factor)) = settling(self.opacity.0, to.opacity.0, t) {
      let from = if vanishes { self } else { to };
      let centre = from.screen.centre();
      let (x, y) = about((from.image.0, from.image.1), centre, factor);
      next.screen = from.screen.scaled(factor);
      next.image = (x, y, from.image.2 * factor);
    }
    if let Some((vanishes, factor)) = settling(self.opacity.1, to.opacity.1, t) {
      let from = if vanishes { self } else { to };
      let centre = from.frame.centre();
      let (x, y) = about((from.camera.0, from.camera.1), centre, factor);
      next.frame = from.frame.scaled(factor);
      next.camera = (x, y, from.camera.2 * factor);
    }
    next
  }
}
