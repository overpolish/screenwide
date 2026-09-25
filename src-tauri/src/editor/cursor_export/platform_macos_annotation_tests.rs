// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's screenshot annotations, through a real Metal dispatch.

use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

/// A canvas the size of its source, placed one source pixel to one output
/// pixel, so an annotation's source coordinates are also its canvas ones.
fn annotation_output(width: u32, height: u32) -> crate::screenshots::ScreenshotOutputSettings {
  let mut output = crate::screenshots::test_output_settings(width, height);
  output.background_color = "#000000".to_owned();
  output.background_type = "solid".to_owned();
  output.crop_height = f64::from(height);
  output.crop_width = f64::from(width);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(width);
  output.image_x = 0.0;
  output.image_y = 0.0;
  output
}

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// One arrow on a black canvas of `size`, drawn in pure red so a channel test
/// reads as coverage.
fn arrow(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
  width: f64,
  head: AnnotationHead,
) -> Annotation {
  Annotation {
    above_camera: false,
    animated: true,
    id: "arrow".to_owned(),
    held: None,
    reveal: Default::default(),
    shape: AnnotationShape::Arrow {
      start,
      control,
      end,
    },
    style: AnnotationStyle {
      align: Default::default(),
      color: "#ff0000".to_owned(),
      head,
      radius: 0.0,
      redaction: Default::default(),
      strength: 0.0,
      width,
    },
  }
}

fn midpoint(start: AnnotationPoint, end: AnnotationPoint) -> AnnotationPoint {
  point((start.x + end.x) / 2.0, (start.y + end.y) / 2.0)
}

/// Composes one arrow onto a black canvas of `size` and hands back the RGBA.
fn composed(size: (u32, u32), annotation: Annotation) -> crate::screenshots::CapturedImage {
  let source = crate::screenshots::CapturedImage {
    rgba: [0, 0, 0, 255].repeat(size.0 as usize * size.1 as usize),
    width: size.0,
    height: size.1,
  };
  let mut settings = annotation_output(size.0, size.1);
  settings.annotations = vec![annotation];
  let rgba = crate::screenshots::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap();
  crate::screenshots::CapturedImage {
    rgba: rgba.rgba,
    width: size.0,
    height: size.1,
  }
}

/// Writes a rendered case out as a PNG when `SCREENWIDE_ARROW_PNG_DIR` names
/// a directory. The suite asserts on pixels; this is the eyeball pass that
/// proved the curve solve, kept so the next shape can reuse it.
fn write_png(name: &str, image: &crate::screenshots::CapturedImage) {
  let Ok(directory) = std::env::var("SCREENWIDE_ARROW_PNG_DIR") else {
    return;
  };
  let bytes = crate::screenshots::encode_png(image).unwrap();
  std::fs::create_dir_all(&directory).unwrap();
  std::fs::write(
    std::path::Path::new(&directory).join(format!("{name}.png")),
    bytes,
  )
  .unwrap();
}

/// A nearest-neighbour blow-up of one region, so a head's join reads at a
/// glance rather than as three ambiguous pixels.
fn write_png_zoom(
  name: &str,
  image: &crate::screenshots::CapturedImage,
  region: (u32, u32, u32, u32),
  zoom: u32,
) {
  let (left, top, width, height) = region;
  let mut rgba = Vec::with_capacity((width * zoom * height * zoom * 4) as usize);
  for row in 0..height * zoom {
    for column in 0..width * zoom {
      let x = (left + column / zoom).min(image.width - 1) as usize;
      let y = (top + row / zoom).min(image.height - 1) as usize;
      let offset = (y * image.width as usize + x) * 4;
      rgba.extend_from_slice(&image.rgba[offset..offset + 4]);
    }
  }
  write_png(
    name,
    &crate::screenshots::CapturedImage {
      rgba,
      width: width * zoom,
      height: height * zoom,
    },
  );
}

/// The curve's point at `t`, the same quadratic the shader walks.
fn curve_point(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
  t: f64,
) -> AnnotationPoint {
  let inverse = 1.0 - t;
  point(
    inverse * inverse * start.x + 2.0 * inverse * t * control.x + t * t * end.x,
    inverse * inverse * start.y + 2.0 * inverse * t * control.y + t * t * end.y,
  )
}

/// A region worth blowing up - a head's join, or a stretch of shaft a bug
/// ate - as a label and a left/top/width/height rectangle.
type ArrowZoom = (&'static str, (u32, u32, u32, u32));

/// One rendered case: a name, the canvas, the arrow, and its zooms.
struct ArrowCase {
  annotation: Annotation,
  name: &'static str,
  size: (u32, u32),
  zooms: &'static [ArrowZoom],
}

/// Every case the arrow shader is proved against.
fn cases() -> Vec<ArrowCase> {
  let straight = (point(60.0, 300.0), point(900.0, 240.0));
  let vertical = (point(480.0, 40.0), point(470.0, 600.0));
  let both = (point(100.0, 100.0), point(860.0, 520.0));
  vec![
    ArrowCase {
      annotation: arrow(
        straight.0,
        midpoint(straight.0, straight.1),
        straight.1,
        6.0,
        AnnotationHead::End,
      ),
      name: "a-straight",
      size: (960, 640),
      zooms: &[
        ("head", (820, 200, 140, 90)),
        ("middle", (440, 250, 120, 50)),
      ],
    },
    ArrowCase {
      annotation: arrow(
        straight.0,
        point(480.0, midpoint(straight.0, straight.1).y + 3.0),
        straight.1,
        6.0,
        AnnotationHead::End,
      ),
      name: "b-nearly-straight",
      size: (960, 640),
      zooms: &[
        ("head", (820, 200, 140, 90)),
        ("middle", (440, 250, 120, 50)),
      ],
    },
    ArrowCase {
      annotation: arrow(
        point(100.0, 560.0),
        point(60.0, -100.0),
        point(620.0, 60.0),
        4.0,
        AnnotationHead::End,
      ),
      name: "c-curved",
      size: (960, 640),
      zooms: &[("head", (520, 20, 140, 90)), ("notch", (70, 440, 90, 110))],
    },
    ArrowCase {
      annotation: arrow(
        vertical.0,
        midpoint(vertical.0, vertical.1),
        vertical.1,
        6.0,
        AnnotationHead::End,
      ),
      name: "d-vertical",
      size: (960, 640),
      zooms: &[("head", (410, 520, 120, 100))],
    },
    ArrowCase {
      annotation: arrow(
        both.0,
        midpoint(both.0, both.1),
        both.1,
        8.0,
        AnnotationHead::Both,
      ),
      name: "e-both-heads",
      size: (960, 640),
      zooms: &[
        ("start-head", (60, 60, 130, 110)),
        ("end-head", (770, 430, 130, 110)),
      ],
    },
    // The regime the reported bug lived in: a chord of three thousand pixels
    // with a bend of five. See `a_nearly_straight_shaft_is_continuous`.
    ArrowCase {
      annotation: arrow(
        point(200.0, 1200.0),
        point(1900.0, 1105.0),
        point(3600.0, 1000.0),
        6.0,
        AnnotationHead::End,
      ),
      name: "f-long-nearly-straight",
      size: (3840, 2400),
      zooms: &[
        ("head", (3400, 900, 260, 180)),
        ("middle", (1700, 1060, 400, 90)),
      ],
    },
    // The other extreme, where a derivative-led refinement walks off: the
    // control is far from the chord, so |a - 2b + c| is thousands of pixels.
    ArrowCase {
      annotation: arrow(HAIRPIN.0, HAIRPIN.1, HAIRPIN.2, 6.0, AnnotationHead::End),
      name: "g-hairpin",
      size: HAIRPIN_SIZE,
      zooms: &[
        ("apex", (2000, 1100, 140, 200)),
        ("bend", (1900, 1010, 150, 120)),
      ],
    },
    ArrowCase {
      annotation: arrow(CUSP.0, CUSP.1, CUSP.2, 6.0, AnnotationHead::End),
      name: "h-near-cusp",
      size: CUSP_SIZE,
      zooms: &[
        ("apex", (690, 460, 90, 90)),
        ("throat", (620, 780, 180, 140)),
      ],
    },
  ]
}

/// A hairpin: the control sits far off the chord, well past either tip. The
/// sag a bad refinement leaves grows with the curve, so this is the reported
/// shape at the size a full-resolution screenshot gives it.
const HAIRPIN_SIZE: (u32, u32) = (4800, 3000);
const HAIRPIN: (AnnotationPoint, AnnotationPoint, AnnotationPoint) = (
  AnnotationPoint { x: 0.0, y: 900.0 },
  AnnotationPoint {
    x: 4200.0,
    y: 900.0,
  },
  AnnotationPoint { x: 0.0, y: 2100.0 },
);
/// A near-cusp: the two tips almost touch and the control is more than ten
/// chords away, so |B'| very nearly falls to nothing while `a - 2b + c` is
/// enormous - exactly where a derivative-led refinement has no footing.
const CUSP_SIZE: (u32, u32) = (1600, 1000);
const CUSP: (AnnotationPoint, AnnotationPoint, AnnotationPoint) = (
  AnnotationPoint { x: 650.0, y: 900.0 },
  AnnotationPoint { x: 730.0, y: 100.0 },
  AnnotationPoint { x: 810.0, y: 900.0 },
);

#[test]
fn renders_every_arrow_case() {
  for case in cases() {
    let image = composed(case.size, case.annotation);
    write_png(case.name, &image);
    for (label, region) in case.zooms {
      write_png_zoom(&format!("{}-{label}", case.name), &image, *region, 6);
    }
  }
}

#[test]
fn composites_an_arrow_annotation_into_a_gpu_still() {
  // The image is placed one source pixel to one output pixel, so the arrow's
  // source-space control points are also its canvas coordinates.
  let image = composed(
    (640, 360),
    arrow(
      point(100.0, 80.0),
      point(300.0, 40.0),
      point(500.0, 80.0),
      12.0,
      AnnotationHead::End,
    ),
  );
  let pixel = |x: usize, y: usize| {
    let start = (y * 640 + x) * 4;
    image.rgba[start..start + 4].to_vec()
  };
  // The curve's midpoint, and a corner the arrow comes nowhere near.
  let shaft = pixel(300, 60);
  assert!(
    shaft[0] > 240 && shaft[1] < 16 && shaft[2] < 16,
    "the shaft is {shaft:?}"
  );
  // Halfway along the head, which reaches four widths back from the tip.
  let head = pixel(476, 75);
  assert!(
    head[0] > 240 && head[1] < 16 && head[2] < 16,
    "the head is {head:?}"
  );
  // Past the tip, where the shaft must not poke through.
  let beyond = pixel(508, 82);
  assert!(beyond[0] < 16, "the tip overshoots into {beyond:?}");
  let away = pixel(20, 300);
  assert!(away[0] < 16, "the background is {away:?}");
}

/// The curve's unit tangent at `t`.
fn curve_tangent(
  start: AnnotationPoint,
  control: AnnotationPoint,
  end: AnnotationPoint,
  t: f64,
) -> AnnotationPoint {
  let (x, y) = (
    2.0 * ((1.0 - t) * (control.x - start.x) + t * (end.x - control.x)),
    2.0 * ((1.0 - t) * (control.y - start.y) + t * (end.y - control.y)),
  );
  let length = x.hypot(y).max(1e-9);
  point(x / length, y / length)
}

/// Where the shaft's ink lies across the curve at `at`, as the offset of the
/// ink's unbroken run along the curve's normal and that run's width. The run
/// has to be unbroken, or a tight bend's other arm - which may pass within a
/// few tens of pixels - would be averaged in; where the two arms do touch the
/// run comes back wider than a stroke, and only the ink's presence means
/// anything there.
fn shaft_offset(
  image: &crate::screenshots::CapturedImage,
  at: AnnotationPoint,
  tangent: AnnotationPoint,
) -> (f64, f64) {
  const STEP: f64 = 0.25;
  const REACH: i32 = 40;
  let normal = point(-tangent.y, tangent.x);
  let covered = |step: i32| {
    let offset = f64::from(step) * STEP;
    let x = (at.x + normal.x * offset).round();
    let y = (at.y + normal.y * offset).round();
    if x < 0.0 || y < 0.0 || x >= f64::from(image.width) || y >= f64::from(image.height) {
      return false;
    }
    let index = (y as usize * image.width as usize + x as usize) * 4;
    image.rgba[index] > 128
  };
  assert!(covered(0), "no shaft at {at:?}");
  let mut first = 0;
  while first > -REACH && covered(first - 1) {
    first -= 1;
  }
  let mut last = 0;
  while last < REACH && covered(last + 1) {
    last += 1;
  }
  (
    f64::from(first + last) * STEP / 2.0,
    f64::from(last - first) * STEP,
  )
}

/// The shaft should follow its quadratic centreline up to the point where a
/// head takes over.
#[test]
fn the_shaft_stays_centred_on_its_curve() {
  let curves = [
    (
      "straight",
      point(60.0, 300.0),
      point(480.0, 270.0),
      point(900.0, 240.0),
      6.0,
    ),
    (
      "nearly straight",
      point(60.0, 300.0),
      point(480.0, 273.0),
      point(900.0, 240.0),
      6.0,
    ),
    (
      "curved",
      point(100.0, 560.0),
      point(60.0, -100.0),
      point(620.0, 60.0),
      4.0,
    ),
    (
      "vertical",
      point(480.0, 40.0),
      point(475.0, 320.0),
      point(470.0, 600.0),
      6.0,
    ),
  ];
  for (name, start, control, end, width) in curves {
    let image = composed(
      (960, 640),
      arrow(start, control, end, width, AnnotationHead::End),
    );
    // The head reaches four widths back from the tip, so the samples stop at
    // nine tenths of the curve rather than walking into it.
    for step in 1..=18 {
      let t = f64::from(step) / 20.0;
      let at = curve_point(start, control, end, t);
      let (offset, _) = shaft_offset(&image, at, curve_tangent(start, control, end, t));
      assert!(
        offset.abs() <= 1.0,
        "the {name} shaft at t = {t} is {offset} px off its curve"
      );
    }
  }
}

/// The bug this file was extended for. A fresh arrow's control point sits on
/// the midpoint, and nudging a tip leaves it almost there, so "nearly
/// straight" is the ordinary case rather than a corner one. The closed-form
/// Bezier solve divides by |a - 2b + c|², which is then vanishingly small
/// beside the chord: at a canvas' scale in fp32 it put the closest point in
/// the wrong place, and long stretches of shaft simply did not draw. A bend
/// of five pixels over a chord of three thousand lost a fifth of the curve.
#[test]
fn a_nearly_straight_shaft_is_continuous() {
  let (start, end) = (point(200.0, 1200.0), point(3600.0, 1000.0));
  for bend in [3.0_f64, 5.0, 6.0, 10.0, 24.0] {
    let control = point(1900.0, 1100.0 + bend);
    let image = composed(
      (3840, 2400),
      arrow(start, control, end, 6.0, AnnotationHead::End),
    );
    // The head reaches four widths back from the tip, so the walk stops at
    // nine tenths of the curve rather than stepping into it.
    for step in 0..=200 {
      let t = f64::from(step) / 200.0 * 0.9;
      let at = curve_point(start, control, end, t);
      let (offset, _) = shaft_offset(&image, at, curve_tangent(start, control, end, t));
      assert!(
        offset.abs() <= 1.5,
        "the shaft of a {bend} px bend is {offset} px off its curve at t = {t}"
      );
    }
  }
}

/// The other half of the curve solve. A tight bend defeats Newton on
/// f(t) = dot(B(t) - p, B'(t)): f'(t) carries 2·dot(B(t) - p, a - 2b + c),
/// which on a hairpin is thousands of pixels and swamps |B'|², so the step
/// runs towards a maximum and the coarse sample it falls back on sags half a
/// step off the curve - small slanted gaps with round caps, at every sample
/// midpoint along the bend.
#[test]
fn a_tight_bend_has_no_gaps() {
  let curves = [
    ("hairpin", HAIRPIN_SIZE, HAIRPIN),
    ("near cusp", CUSP_SIZE, CUSP),
  ];
  for (name, size, (start, control, end)) in curves {
    let image = composed(size, arrow(start, control, end, 6.0, AnnotationHead::End));
    // The head reaches four widths back from the tip, so the walk stops at
    // nine tenths of the curve rather than stepping into it.
    for step in 0..=200 {
      let t = f64::from(step) / 200.0 * 0.9;
      let at = curve_point(start, control, end, t);
      // `shaft_offset` asserts the curve point has ink at all, which is what
      // the gaps broke; the centring only means anything where the bend's two
      // arms have not run into one another.
      let (offset, width) = shaft_offset(&image, at, curve_tangent(start, control, end, t));
      assert!(
        width > 12.0 || offset.abs() <= 1.5,
        "the {name} shaft at t = {t} is {offset} px off its curve"
      );
    }
  }
}

/// The stroke's edge is feathered over one drawn pixel. The still composes
/// one canvas pixel to one output pixel, so the ramp is about a pixel each
/// side however large the canvas is - what it must never be is a hard step,
/// which is what an edge feathered in canvas pixels looks like once a layer
/// is drawn smaller than its canvas.
#[test]
fn the_stroke_edge_is_feathered() {
  let (start, control, end) = (
    point(120.0, 600.0),
    point(960.0, 540.0),
    point(1800.0, 480.0),
  );
  let image = composed(
    (1920, 1200),
    arrow(start, control, end, 12.0, AnnotationHead::End),
  );
  write_png("a-straight-2x", &image);
  // Straight down through the shaft, over a stretch of columns: an edge that
  // happens to land on a pixel boundary is a full pixel on one side and none
  // on the other, and says nothing either way.
  let at = curve_point(start, control, end, 0.5);
  let column_partials = |column: usize| {
    (0..1200)
      .filter(|row| {
        let red = image.rgba[(row * 1920 + column) * 4];
        red > 16 && red < 240
      })
      .count()
  };
  let columns = (at.x.round() as usize - 20..at.x.round() as usize + 20)
    .map(column_partials)
    .collect::<Vec<_>>();
  let feathered = columns.iter().filter(|count| **count > 0).count();
  assert!(
    feathered * 2 >= columns.len(),
    "only {feathered} of {} columns cross a feathered edge",
    columns.len()
  );
  assert!(
    columns.iter().all(|count| *count <= 4),
    "the edge ramps over more than a pixel each side: {columns:?}"
  );
  let row = at.y.round() as usize;
  assert!(
    image.rgba[(row * 1920 + at.x.round() as usize) * 4] > 240,
    "the shaft never reaches full colour"
  );
}

#[cfg(test)]
#[path = "platform_macos_annotation_regressions.rs"]
mod annotation_regressions;

#[path = "platform_macos_annotation_reveal_tests.rs"]
mod annotation_reveal;
