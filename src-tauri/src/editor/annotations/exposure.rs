// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How far an annotation's picture moves while the shutter is open.
//!
//! An annotation that moved during the exposure is drawn at every step it
//! took and the samples are averaged, so it smears along the path it actually
//! travelled rather than jumping. How many samples that takes is each
//! backend's own business - the Metal compositor packs them into a buffer, the
//! D3D11 one into its own - but how far the annotation went is the
//! annotation's, and is measured here for both.

use super::counter::geometry::COUNTER_TAIL_REACH;
use super::reveal::AnnotationReveal;
use super::AnnotationKind;

/// How far the annotation's picture moves between the shutter opening and
/// now, in the pixels it is drawn in. `scale` carries a point from the space
/// the points are given in into those pixels; a caller whose points are
/// placed already passes the identity.
///
/// An arrow's travel is its window sliding along its own path, plus the change
/// in its own size; a counter has no path, so all it can cover in a frame is
/// its disc's edge sweeping out as it grows, and a text box likewise its
/// box's corners, `c` being its text block's size.
pub(crate) fn annotation_travel(
  kind: AnnotationKind,
  a: [f32; 2],
  b: [f32; 2],
  c: [f32; 2],
  scale: [f32; 2],
  width: f32,
  reveal: AnnotationReveal,
) -> f32 {
  let previous = reveal.previous;
  match kind {
    AnnotationKind::Counter => {
      width * 0.5 * COUNTER_TAIL_REACH * (reveal.scale - previous[2]).abs()
    }
    // Half the box's width and height together, padding included, is as far
    // as a corner sits from the centre it grows about. The pointer, `b` as
    // the record encodes it, sweeps out from the edge by its own reach.
    AnnotationKind::Text => {
      let box_reach = (c[0] + c[1]) * 0.5 + width;
      let past = |encoded: f32| (encoded.abs() - 1.0).max(0.0);
      let pointer_reach = past(b[0]).hypot(past(b[1])) * width + box_reach;
      box_reach * (reveal.scale - previous[2]).abs()
        + pointer_reach * (reveal.high - previous[1]).abs()
    }
    AnnotationKind::Arrow => {
      let length = ((b[0] - a[0]) * scale[0]).hypot((b[1] - a[1]) * scale[1])
        + ((c[0] - b[0]) * scale[0]).hypot((c[1] - b[1]) * scale[1]);
      length
        * (reveal.low - previous[0])
          .abs()
          .max((reveal.high - previous[1]).abs())
        + width * 4.0 * (reveal.scale - previous[2]).abs()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn an_annotation_standing_still_covers_no_ground() {
    for kind in [AnnotationKind::Arrow, AnnotationKind::Counter] {
      assert_eq!(
        annotation_travel(
          kind,
          [100.0, 100.0],
          [220.0, 40.0],
          [300.0, 200.0],
          [1.0, 1.0],
          12.0,
          AnnotationReveal::WHOLE
        ),
        0.0
      );
    }
  }

  #[test]
  fn a_counter_growing_sweeps_its_tails_reach() {
    let arriving = AnnotationReveal {
      scale: 0.5,
      previous: [0.0, 1.0, 0.0, 1.0],
      ..AnnotationReveal::WHOLE
    };
    assert_eq!(
      annotation_travel(
        AnnotationKind::Counter,
        [100.0, 100.0],
        [0.0, 3.0],
        [100.0, 100.0],
        [1.0, 1.0],
        40.0,
        arriving
      ),
      40.0 * 0.5 * 1.5 * 0.5
    );
  }

  #[test]
  fn an_arrow_drawing_itself_in_covers_its_own_path_in_the_pixels_it_is_drawn_in() {
    let half = AnnotationReveal {
      high: 0.5,
      previous: [0.0, 0.0, 1.0, 1.0],
      ..AnnotationReveal::WHOLE
    };
    // The legs are 100 and 200 long in the space the points are given in, and
    // the axis scale doubles that before the window's half is taken off it.
    assert_eq!(
      annotation_travel(
        AnnotationKind::Arrow,
        [0.0, 0.0],
        [100.0, 0.0],
        [300.0, 0.0],
        [2.0, 2.0],
        0.0,
        half
      ),
      300.0
    );
  }
}
