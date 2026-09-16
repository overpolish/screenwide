// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An arrow drawing itself in and out, through the same Metal dispatch the
//! rest of the suite proves the static shape against.

use super::*;
use crate::editor::annotations::reveal::geometry::reveal_geometry;
use crate::editor::annotations::reveal::{reveal_window, AnnotationReveal};

/// A clip long enough for both phases to run at their full half second.
const CLIP_MS: f32 = 3_000.0;

/// A straight arrow and a strongly bent one, the two shapes the reveal has to
/// walk evenly: arc length and curve parameter agree on one and not the other.
const STRAIGHT: (AnnotationPoint, AnnotationPoint, AnnotationPoint) = (
  AnnotationPoint { x: 60.0, y: 300.0 },
  AnnotationPoint { x: 480.0, y: 270.0 },
  AnnotationPoint { x: 900.0, y: 240.0 },
);
const CURVED: (AnnotationPoint, AnnotationPoint, AnnotationPoint) = (
  AnnotationPoint { x: 100.0, y: 560.0 },
  AnnotationPoint { x: 60.0, y: -100.0 },
  AnnotationPoint { x: 620.0, y: 60.0 },
);
const SIZE: (u32, u32) = (960, 640);

fn shape(curve: (AnnotationPoint, AnnotationPoint, AnnotationPoint), width: f64) -> Annotation {
  arrow(curve.0, curve.1, curve.2, width, AnnotationHead::End)
}

/// The same mark, `elapsed` milliseconds into its clip.
fn at_reveal(mut annotation: Annotation, elapsed: f32, frame_ms: f32) -> Annotation {
  annotation.reveal = reveal_window(elapsed, CLIP_MS, frame_ms);
  annotation
}

/// The curve parameter `fraction` of the way along the path by arc length,
/// measured from the tail. Asking the reveal for it is the point: this is the
/// same inversion the compositor draws with.
fn parameter_at(curve: (AnnotationPoint, AnnotationPoint, AnnotationPoint), fraction: f32) -> f64 {
  let vector = |p: AnnotationPoint| [p.x as f32, p.y as f32];
  f64::from(
    reveal_geometry(
      vector(curve.0),
      vector(curve.1),
      vector(curve.2),
      0.0,
      0.0,
      AnnotationReveal {
        low: 0.0,
        high: fraction,
        scale: 1.0,
        opacity: 1.0,
        previous: [0.0, fraction, 1.0, 1.0],
      },
    )
    .high,
  )
}

/// How much of the mark's red ink reaches a point, which on a black canvas
/// is its coverage: the blur is a partial one, so it is only readable here.
///
/// The pixel sampled is the one the point falls inside, whose centre is the
/// canvas point the shader evaluated. Rounding instead lands half a pixel
/// away, which a mark drawn at a fraction of its weight can read as a gap.
fn red(image: &crate::screenshots::CapturedImage, at: AnnotationPoint) -> u8 {
  let (x, y) = (at.x.floor(), at.y.floor());
  if x < 0.0 || y < 0.0 || x >= f64::from(image.width) || y >= f64::from(image.height) {
    return 0;
  }
  image.rgba[(y as usize * image.width as usize + x as usize) * 4]
}

/// Whether the mark's red ink reaches a point, which is all the arrow leaves
/// on a black canvas.
fn inked(image: &crate::screenshots::CapturedImage, at: AnnotationPoint) -> bool {
  red(image, at) > 128
}

/// How long the path is, in canvas pixels, so a test can stop an absolute
/// distance short of a point rather than a fraction of what is drawn.
fn arc_length(curve: (AnnotationPoint, AnnotationPoint, AnnotationPoint)) -> f64 {
  let mut total = 0.0;
  let mut previous = curve.0;
  for step in 1..=4_096 {
    let point = curve_point(curve.0, curve.1, curve.2, f64::from(step) / 4_096.0);
    total += (point.x - previous.x).hypot(point.y - previous.y);
    previous = point;
  }
  total
}

/// Where the path is, `fraction` of its length from the tail.
fn along(
  curve: (AnnotationPoint, AnnotationPoint, AnnotationPoint),
  fraction: f32,
) -> AnnotationPoint {
  curve_point(curve.0, curve.1, curve.2, parameter_at(curve, fraction))
}

/// The stretch of the path a mark covers `elapsed` into its clip, as
/// fractions of the arc from the tail: the shaft's window, which runs over the
/// path less the head, and the head riding ahead of the window's end.
fn covered(
  curve: (AnnotationPoint, AnnotationPoint, AnnotationPoint),
  width: f64,
  elapsed: f32,
) -> (f32, f32) {
  let window = reveal_window(elapsed, CLIP_MS, 0.0);
  let head = (width * 4.0 / arc_length(curve)) as f32;
  (
    window.low * (1.0 - head),
    window.high * (1.0 - head) + head * window.scale,
  )
}

/// The stroke leaves the tail and runs to the head, which rides ahead of the
/// window's end: a reveal that walked the curve's parameter instead of its
/// length would run ahead of the bend on one arm and lag on the other.
#[test]
fn the_shaft_grows_from_the_tail_by_arc_length() {
  for (label, curve, width) in [("straight", STRAIGHT, 6.0), ("curved", CURVED, 4.0)] {
    // 500ms is half of the one-second opening phase, and the ease is well
    // past half the travel by then.
    let image = composed(SIZE, at_reveal(shape(curve, width), 500.0, 0.0));
    let (_, front) = covered(curve, width, 500.0);
    assert!(inked(&image, along(curve, 0.0)), "{label} tail");
    assert!(
      inked(&image, along(curve, front * 0.5)),
      "{label} middle of what is revealed"
    );
    assert!(
      !inked(&image, along(curve, (front + 1.0) / 2.0)),
      "{label} past the head"
    );
    assert!(!inked(&image, along(curve, 1.0)), "{label} the end");
  }
}

/// A bent curve's midpoint by length is not its midpoint by parameter, and
/// the two are further apart than a stroke is wide: revealing half the arrow
/// has to stop at the first, so this fails outright on the second.
#[test]
fn half_a_bent_arrow_stops_at_half_its_length() {
  let width = 4.0;
  let arc = along(CURVED, 0.5);
  let parameter = curve_point(CURVED.0, CURVED.1, CURVED.2, 0.5);
  assert!(
    (arc.x - parameter.x).hypot(arc.y - parameter.y) > width * 4.0,
    "the two midpoints are {arc:?} and {parameter:?}"
  );
  // A window of exactly half the arc, so the head sits on the arc midpoint.
  let mut annotation = shape(CURVED, width);
  annotation.reveal = AnnotationReveal {
    low: 0.0,
    high: 0.5,
    scale: 1.0,
    opacity: 1.0,
    previous: [0.0, 1.0, 1.0, 1.0],
  };
  let image = composed(SIZE, annotation);
  assert!(
    inked(&image, along(CURVED, 0.49)),
    "the head is not on the arc midpoint"
  );
  assert!(
    !inked(&image, along(CURVED, 0.75)),
    "ink past the arc midpoint"
  );
}

/// Nothing breaks open between the shaft and the head while the mark is still
/// growing: the seam is the whole reason the shaft is pulled back from it.
///
/// The sweep starts where the mark's stroke is a pixel wide or more. Below
/// that a mark is drawn as a fraction of a pixel of coverage everywhere, and
/// a gap in it is not a thing a threshold can tell from its own faintness.
#[test]
fn the_seam_never_gaps_while_the_mark_grows() {
  for (label, curve, width) in [("straight", STRAIGHT, 6.0), ("curved", CURVED, 4.0)] {
    for elapsed in [300.0, 500.0, 800.0, 2_400.0, 2_700.0] {
      let image = composed(SIZE, at_reveal(shape(curve, width), elapsed, 0.0));
      let (back, front) = covered(curve, width, elapsed);
      // Stopping just short of the tip: the head's apex is a point, and its
      // last pixel is a feathered fraction of one by design. The margin is an
      // absolute distance rather than a share of what is drawn, and it shrinks
      // with the mark, whose apex is as blunt as its size makes it.
      let scale = f64::from(reveal_window(elapsed, CLIP_MS, 0.0).scale);
      let margin = (width * 0.75 * scale / arc_length(curve)) as f32;
      let front = front - margin;
      for step in 0..=60_u8 {
        let fraction = back + (front - back) * f32::from(step) / 60.0;
        let point = along(curve, fraction);
        assert!(
          inked(&image, point),
          "{label} at {elapsed}ms broke open {fraction} along, at {point:?}"
        );
      }
    }
  }
}

/// The out phase collapses towards the head rather than away from it: the
/// tail catches up, and the arrow leaves from where it was pointing.
#[test]
fn the_tail_catches_up_to_the_head() {
  let width = 6.0;
  // Three quarters through the closing phase, where the in-out ease has
  // taken fifteen sixteenths of the tail's travel.
  let image = composed(SIZE, at_reveal(shape(STRAIGHT, width), 2_800.0, 0.0));
  let gone = reveal_window(2_800.0, CLIP_MS, 0.0).low;
  assert!(gone > 0.8, "the tail is only {gone} along");
  assert!(
    !inked(&image, along(STRAIGHT, gone * 0.5)),
    "the tail is still drawn"
  );
  assert!(
    inked(&image, along(STRAIGHT, 0.995)),
    "the head left before the tail arrived"
  );
  // And at the very end there is nothing left at all.
  let empty = composed(SIZE, at_reveal(shape(STRAIGHT, width), CLIP_MS, 0.0));
  for step in 0..=20_u8 {
    let fraction = f32::from(step) / 20.0;
    assert!(
      !inked(&empty, along(STRAIGHT, fraction)),
      "ink left at the end of the clip, {fraction} along"
    );
  }
}

/// How much of the canvas the mark inked, which is the only measure of a
/// frame that catches a jump anywhere in it at once.
fn ink(image: &crate::screenshots::CapturedImage) -> usize {
  image.rgba.chunks_exact(4).filter(|p| p[0] > 128).count()
}

/// Nothing appears before the reveal has anything to show, and what shows
/// first is a mark at the size the clip has earned it: a head with no
/// triangle worth building must not be drawn as one, and a triangle with no
/// winding reads as inside every pixel on the canvas.
#[test]
fn nothing_is_drawn_before_the_reveal_starts() {
  let frame_ms = 1_000.0 / 60.0;
  let whole = ink(&composed(SIZE, shape(STRAIGHT, 6.0)));
  assert_eq!(
    ink(&composed(
      SIZE,
      at_reveal(shape(STRAIGHT, 6.0), 0.0, frame_ms)
    )),
    0
  );
  for elapsed in [0.25, 0.5, 1.0, 2.0] {
    let drawn = ink(&composed(
      SIZE,
      at_reveal(shape(STRAIGHT, 6.0), elapsed, frame_ms),
    ));
    assert!(
      drawn < whole / 50,
      "{drawn} of {whole} inked {elapsed}ms into the clip"
    );
  }
}

/// The reveal only ever grows while it opens and only ever shrinks while it
/// closes, and never by a quarter of the arrow between two frames: a phase
/// boundary that changed how the mark is prepared would show up here as a
/// step, whichever direction it stepped in.
#[test]
fn the_reveal_never_steps_between_frames() {
  let width = 6.0;
  let whole = ink(&composed(SIZE, shape(STRAIGHT, width)));
  let sweep = |from: f32, opening: bool| {
    let mut previous = if opening { 0 } else { whole };
    for step in 0..=52_u8 {
      let elapsed = from + f32::from(step) * 20.0;
      let count = ink(&composed(
        SIZE,
        at_reveal(shape(STRAIGHT, width), elapsed, 0.0),
      ));
      let (low, high) = if opening {
        (previous, count)
      } else {
        (count, previous)
      };
      // The tolerance is the pullback's own width: while a mark is drawing
      // its shaft stops short of the head, and the fraction of a pixel that
      // costs where the two meet is not a step in the animation.
      assert!(
        low <= high + whole / 200,
        "{elapsed}ms went from {previous} to {count}"
      );
      assert!(
        high.saturating_sub(low) <= whole / 4,
        "{elapsed}ms stepped from {previous} to {count} of {whole}"
      );
      previous = count;
    }
    previous
  };
  assert_eq!(sweep(0.0, true), whole, "the clip holds the whole arrow");
  assert_eq!(sweep(2_250.0, false), 0, "the clip empties");
}

/// A mark that does not animate is drawn whole for the clip's whole length,
/// and a mark holding between its phases is drawn the same way.
#[test]
fn a_mark_that_does_not_animate_is_never_cut_short() {
  let width = 6.0;
  let whole = composed(SIZE, shape(STRAIGHT, width));
  let holding = composed(SIZE, at_reveal(shape(STRAIGHT, width), 1_500.0, 16.0));
  assert_eq!(whole.rgba, holding.rgba);
}

/// The prepared reveal of a mark `elapsed` into its clip, in the curve's own
/// parameter: the shaft's ends and where its heads' tips are.
fn prepared(
  curve: (AnnotationPoint, AnnotationPoint, AnnotationPoint),
  width: f64,
  heads: f32,
  elapsed: f32,
  frame_ms: f32,
) -> crate::editor::annotations::reveal::geometry::AnnotationRevealGeometry {
  let vector = |p: AnnotationPoint| [p.x as f32, p.y as f32];
  reveal_geometry(
    vector(curve.0),
    vector(curve.1),
    vector(curve.2),
    width as f32,
    heads,
    reveal_window(elapsed, CLIP_MS, frame_ms),
  )
}

/// The blur smears the mark along the path it travelled: between the still
/// frame's moving end and the end it reached, the moving frame lays down
/// partial ink where a still frame has none, and the smear ends there.
#[test]
fn the_mark_smears_along_the_path_it_travelled() {
  let frame_ms = 1_000.0 / 60.0;
  for (label, curve, width) in [("straight", STRAIGHT, 12.0), ("curved", CURVED, 8.0)] {
    for (phase, elapsed, towards_tail, head) in [
      ("in", 500.0, true, AnnotationHead::End),
      ("out", 2_550.0, false, AnnotationHead::End),
      ("out headed", 2_550.0, false, AnnotationHead::Both),
    ] {
      let heads = if head == AnnotationHead::Both {
        2.0
      } else {
        1.0
      };
      let now = prepared(curve, width, heads, elapsed, frame_ms);
      let start = prepared(curve, width, heads, elapsed - frame_ms, frame_ms);
      let (from, to) = if towards_tail {
        (f64::from(start.end_tip), f64::from(now.end_tip))
      } else {
        (f64::from(start.start_tip), f64::from(now.start_tip))
      };
      let drawn = |frame_ms| {
        let mut annotation = arrow(curve.0, curve.1, curve.2, width, head);
        annotation.reveal = reveal_window(elapsed, CLIP_MS, frame_ms);
        composed(SIZE, annotation)
      };
      let (moving, standing) = (drawn(frame_ms), drawn(0.0));
      // Coverage ramps across the travelled span, so its ends approach the
      // still frame while its middle is genuinely partial: the still frame is
      // unambiguous there - solid behind a drawing tip, empty behind a
      // departing tail - and the exposure is not.
      for share in [0.35, 0.5, 0.65] {
        let point = curve_point(curve.0, curve.1, curve.2, from + (to - from) * share);
        let smeared = red(&moving, point);
        let still = red(&standing, point);
        assert!(
          !(40..=250).contains(&still),
          "{label} {phase}: the still frame is itself partial at {share} of the travel"
        );
        assert!(
          smeared > 30 && smeared < 235,
          "{label} {phase}: {smeared} at {share} of the travel, at {point:?}"
        );
      }
    }
  }
}

/// The blur, rendered out for the eyeball pass: one 60fps frame of a
/// half-second draw-in on a long arrow, in both phases, with the same frame
/// standing still beside it. Each is blown up around the span the mark's
/// moving end crossed during the exposure.
#[test]
fn renders_the_smear_along_the_travelled_span() {
  let frame_ms = 1_000.0 / 60.0;
  for (label, curve, width) in [("straight", STRAIGHT, 12.0), ("curved", CURVED, 8.0)] {
    for (phase, elapsed, towards_tail, head) in [
      ("in", 500.0, true, AnnotationHead::End),
      ("out", 2_550.0, false, AnnotationHead::End),
      ("out-headed", 2_550.0, false, AnnotationHead::Both),
    ] {
      let heads = if head == AnnotationHead::Both {
        2.0
      } else {
        1.0
      };
      let now = prepared(curve, width, heads, elapsed, frame_ms);
      let start = prepared(curve, width, heads, elapsed - frame_ms, frame_ms);
      let (from, to) = if towards_tail {
        (f64::from(start.end_tip), f64::from(now.end_tip))
      } else {
        (f64::from(start.start_tip), f64::from(now.start_tip))
      };
      let middle = curve_point(curve.0, curve.1, curve.2, (from + to) * 0.5);
      for (blurred, frame_ms) in [("", frame_ms), ("-still", 0.0)] {
        let mut annotation = arrow(curve.0, curve.1, curve.2, width, head);
        annotation.reveal = reveal_window(elapsed, CLIP_MS, frame_ms);
        let image = composed(SIZE, annotation);
        let name = format!("i-reveal-blur-{label}-{phase}{blurred}");
        write_png(&name, &image);
        write_png_zoom(
          &format!("{name}-zoom"),
          &image,
          (
            (middle.x as u32).saturating_sub(70),
            (middle.y as u32).saturating_sub(70),
            140,
            140,
          ),
          4,
        );
      }
    }
  }
}

/// Every phase, rendered out for the eyeball pass. An arrow headed at both
/// ends wears one on the end that moves, so it gets its own frames.
#[test]
fn renders_every_reveal_phase() {
  let frame_ms = 1_000.0 / 60.0;
  for elapsed in [10.0, 100.0, 375.0, 2_400.0, 2_700.0, 2_950.0] {
    let mut both = arrow(CURVED.0, CURVED.1, CURVED.2, 4.0, AnnotationHead::Both);
    both.reveal = reveal_window(elapsed, CLIP_MS, frame_ms);
    write_png(&format!("i-reveal-both-{elapsed}"), &composed(SIZE, both));
  }
  for (label, curve, width) in [("straight", STRAIGHT, 6.0), ("curved", CURVED, 4.0)] {
    for elapsed in [
      0.0, 100.0, 200.0, 375.0, 750.0, 1_500.0, 2_400.0, 2_700.0, 2_900.0, 2_975.0,
    ] {
      let image = composed(SIZE, at_reveal(shape(curve, width), elapsed, frame_ms));
      write_png(&format!("i-reveal-{label}-{elapsed}"), &image);
    }
    // The head's first head-length of travel and its last, blown up: this is
    // where the mark grows in and leaves, and where a shallow pullback would
    // show the curve.
    for elapsed in [10.0, 100.0, 225.0, 2_850.0, 2_950.0] {
      let image = composed(SIZE, at_reveal(shape(curve, width), elapsed, frame_ms));
      let travel = prepared(curve, width, 1.0, elapsed, frame_ms);
      let head = curve_point(curve.0, curve.1, curve.2, f64::from(travel.end_tip));
      write_png_zoom(
        &format!("i-reveal-{label}-seam-{elapsed}"),
        &image,
        (
          (head.x as u32).saturating_sub(60),
          (head.y as u32).saturating_sub(40),
          80,
          80,
        ),
        8,
      );
    }
  }
}

/// The head is ahead of the stroke from the first frame, riding on its end
/// at full size, and the stroke is continuous into it: what shows while the
/// mark sets off is a whole arrow fading in, not an arrowhead with a stub of
/// stroke budding out of its base and not a stroke waiting on a head.
#[test]
fn the_head_rides_ahead_of_the_stroke_from_the_start() {
  let width = 12.0;
  let arc = arc_length(STRAIGHT);
  for elapsed in [80.0_f32, 160.0, 250.0, 400.0] {
    let image = composed(SIZE, at_reveal(shape(STRAIGHT, width), elapsed, 0.0));
    let (back, front) = covered(STRAIGHT, width, elapsed);
    let travel = prepared(STRAIGHT, width, 1.0, elapsed, 0.0);
    assert!(
      f64::from(travel.end_tip) > f64::from(travel.high),
      "at {elapsed}ms the head's tip is not ahead of the shaft"
    );
    assert_eq!(travel.scale, 1.0, "at {elapsed}ms the mark is not whole");
    // Solid for however far in the fade is: continuous from the tail to just
    // short of the tip, whose last pixel is a feathered fraction of one.
    let opacity = reveal_window(elapsed, CLIP_MS, 0.0).opacity;
    let solid = (254.0 * opacity * 0.5) as u8;
    let margin = (width * 0.75 / arc) as f32;
    for step in 0..=40_u8 {
      let fraction = back + (front - margin - back) * f32::from(step) / 40.0;
      let point = along(STRAIGHT, fraction);
      assert!(
        red(&image, point) > solid,
        "at {elapsed}ms the mark broke open {fraction} along, at {point:?}"
      );
    }
    // And nothing past the tip by more than a stroke.
    let stroke = (width / arc) as f32;
    assert!(
      red(&image, along(STRAIGHT, front + stroke)) <= solid / 4,
      "at {elapsed}ms there is ink a stroke past the head's tip"
    );
  }
}

/// A stroke's round end reaches half a width past where it stops, and where
/// nothing covers it it shows as a knob on the back of the head for as long
/// as the head is there. Leaving, the tail keeps its round end inside the
/// mark until there is stroke enough to carry it, at any stroke width.
/// Setting off there is nothing to hold in: the stroke has outrun its own
/// width, which is still growing in, before it is a pixel wide.
#[test]
fn a_short_stroke_keeps_its_cap_inside_the_mark() {
  let along_path = |from: AnnotationPoint, to: AnnotationPoint, strokes: f64, width: f64| {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let len = dx.hypot(dy).max(1e-6);
    point(
      from.x + dx / len * width * strokes,
      from.y + dy / len * width * strokes,
    )
  };
  for width in [8.0, 16.0, 48.0] {
    // Leaving, with the stroke still drawing itself in while the head goes:
    // nothing behind the stroke's own end.
    for elapsed in [2_800.0, 2_900.0] {
      let image = composed(SIZE, at_reveal(shape(STRAIGHT, width), elapsed, 0.0));
      let travel = prepared(STRAIGHT, width, 1.0, elapsed, 0.0);
      let window = reveal_window(elapsed, CLIP_MS, 0.0);
      let head = (width * 4.0 / arc_length(STRAIGHT)) as f32;
      let tail = along(STRAIGHT, window.low * (1.0 - head));
      let base = curve_point(STRAIGHT.0, STRAIGHT.1, STRAIGHT.2, f64::from(travel.high));
      let tip = curve_point(
        STRAIGHT.0,
        STRAIGHT.1,
        STRAIGHT.2,
        f64::from(travel.end_tip),
      );
      let inside = point((base.x + tip.x) * 0.5, (base.y + tip.y) * 0.5);
      assert!(
        inked(&image, inside),
        "out {elapsed}ms at width {width}: the head is not drawn"
      );
      for behind in [0.5, 1.0] {
        let at = along_path(tail, STRAIGHT.0, behind, width);
        assert!(
          !inked(&image, at),
          "out {elapsed}ms at width {width}: ink {behind} strokes behind the stroke's end, at {at:?}"
        );
      }
    }
  }
}
