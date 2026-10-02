// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How far an annotation's picture moves while the shutter is open.
//!
//! An annotation that moved during the exposure is drawn at every step it
//! took and the samples are averaged, so it smears along the path it actually
//! travelled rather than jumping. How many samples that takes is the
//! compositor's business, but how far the annotation went is the
//! annotation's, and is measured here.

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
    // A shape's window slides round its outline, `a` and `c` being its
    // box's corners. A hand-drawn stroke runs on past a lap, by at most a
    // quarter of one.
    AnnotationKind::Shape => {
      let across = ((c[0] - a[0]) * scale[0]).abs();
      let down = ((c[1] - a[1]) * scale[1]).abs();
      2.5
        * (across + down)
        * (reveal.low - previous[0])
          .abs()
          .max((reveal.high - previous[1]).abs())
    }
    // A redaction never moves over a clip: it is whole for as long as it is
    // shown, and a spotlight only fades. A highlight's bands and a stroke's
    // line are not in their records' points, so `highlight_travel` measures
    // both from the sweep each record keeps, and a magnifier's loupe size
    // rides beside its points, so `magnify_travel` measures it.
    AnnotationKind::Redact
    | AnnotationKind::Highlight
    | AnnotationKind::Spotlight
    | AnnotationKind::Draw
    | AnnotationKind::Magnify => 0.0,
  }
}

/// How far a highlight's fastest line end, or a stroke's drawing end, moves
/// between the shutter opening and now, in the pixels it is drawn in.
/// `origin` and `unit` are where the source's origin, or a stroke's box
/// corner, and that point moved one source pixel land in the space the
/// points are given in, `scale` carries that space into those pixels, and
/// `sweep` is how far the end runs over the whole reveal, in source pixels,
/// which the record keeps in `params[2]`.
pub(crate) fn highlight_travel(
  origin: [f32; 2],
  unit: [f32; 2],
  scale: [f32; 2],
  sweep: f32,
  reveal: AnnotationReveal,
) -> f32 {
  let previous = reveal.previous;
  let reach = ((unit[0] - origin[0]) * scale[0]).abs();
  let moved = (reveal.low - previous[0])
    .abs()
    .max((reveal.high - previous[1]).abs());
  sweep.max(0.0) * reach * moved
}

/// How far a magnifier's loupe moves between the shutter opening and now, in
/// the pixels it is drawn in. Its centre travels from the zoom area's to its
/// own and its half size grows from the zoom area's to its whole, both in
/// step with the reveal's `scale`, so no point of its rim covers more than
/// the two together. `start` and `end` are the zoom area's corners, `loupe`
/// the loupe's centre and `size` its longer side, all in the space `scale`
/// carries into those pixels.
pub(crate) fn magnify_travel(
  [start, loupe, end]: [[f32; 2]; 3],
  size: f32,
  scale: [f32; 2],
  reveal: AnnotationReveal,
) -> f32 {
  let moved = (reveal.scale - reveal.previous[2]).abs();
  let half = [
    (end[0] - start[0]).abs() * 0.5,
    (end[1] - start[1]).abs() * 0.5,
  ];
  let longest = half[0].max(half[1]) * 2.0;
  if !(moved > 0.0 && longest > 0.0 && size.is_finite()) {
    return 0.0;
  }
  let way = ((loupe[0] - (start[0] + end[0]) * 0.5) * scale[0])
    .hypot((loupe[1] - (start[1] + end[1]) * 0.5) * scale[1]);
  let growth = (size / longest - 1.0).abs() * (half[0] * scale[0]).hypot(half[1] * scale[1]);
  moved * (way + growth)
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

  #[test]
  fn a_highlight_covers_its_fastest_lines_sweep_in_the_pixels_it_is_drawn_in() {
    let drawing = AnnotationReveal {
      low: 0.2,
      high: 0.5,
      previous: [0.2, 0.4, 1.0, 1.0],
      ..AnnotationReveal::WHOLE
    };
    // One source pixel reaches six across in the space the points are given
    // in, which the axis scale doubles, and the window moved a tenth.
    let travel = |reveal| highlight_travel([10.0, 10.0], [16.0, 16.0], [2.0, 2.0], 600.0, reveal);
    assert!(
      (travel(drawing) - 720.0).abs() < 1e-3,
      "{}",
      travel(drawing)
    );
    assert_eq!(travel(AnnotationReveal::WHOLE), 0.0);
  }

  #[test]
  fn a_travelling_loupe_covers_its_way_and_its_growth() {
    // Half-way out of a zoom area 100 across at (100, 100), towards a loupe
    // twice its size 500 away: half of the way, and half of the corner's
    // reach from 50 to 100 along the diagonal.
    let reveal = AnnotationReveal {
      scale: 0.75,
      previous: [0.0, 1.0, 0.25, 0.25],
      ..AnnotationReveal::WHOLE
    };
    let points = [[50.0, 50.0], [500.0, 400.0], [150.0, 150.0]];
    let travel = magnify_travel(points, 200.0, [1.0, 1.0], reveal);
    assert!(
      (travel - 0.5 * (500.0 + 50.0 * 2f32.sqrt())).abs() < 1e-3,
      "{travel}"
    );
    assert_eq!(
      magnify_travel(points, 200.0, [1.0, 1.0], AnnotationReveal::WHOLE),
      0.0
    );
  }
}
