// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Blur and rounded corners through a real Metal dispatch: a blur carries
//! nothing finer than its cells, and a rounded box covers every pixel its
//! outline touches whole, blending only pixels wholly outside it.

use super::redact_tests::{composed, identity, picture, redaction, resampled, HEIGHT, WIDTH};
use crate::editor::annotations::redact::cells::blur_cell;
use crate::editor::annotations::snap::source_per_output;
use crate::editor::annotations::{AnnotationRedaction, AnnotationShape};
use crate::screenshots::{CapturedImage, ScreenshotOutputSettings};

/// The box's pixels once the compositor snaps its corners outward.
const PAINTED: [u32; 4] = [30, 20, 82, 56];

/// `source` with the pixels inside each of the blur's cells put in reverse
/// order: every cell keeps exactly its average and loses everything else.
/// The cells are the compositor's: centred on the box, measured in floats.
fn shuffled_within_cells(
  source: &CapturedImage,
  output: &ScreenshotOutputSettings,
) -> CapturedImage {
  let [x0, y0, x1, y1] = PAINTED;
  let per_output = source_per_output((WIDTH, HEIGHT), output.image_width);
  let cell = blur_cell(3.0, per_output) as f32;
  let axis = |extent: u32| {
    let count = (extent as f32 / cell).ceil().max(1.0);
    ((extent as f32 - count * cell) * 0.5, count as u32 - 1)
  };
  let ((across_origin, last_x), (down_origin, last_y)) = (axis(x1 - x0), axis(y1 - y0));
  let index = |local: u32, origin: f32, last: u32| {
    (((local as f32 - origin) / cell).floor().max(0.0) as u32).min(last)
  };
  let cell_of = |x: u32, y: u32| {
    (
      index(x - x0, across_origin, last_x),
      index(y - y0, down_origin, last_y),
    )
  };
  let mut cells = std::collections::BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
  for y in y0..y1 {
    for x in x0..x1 {
      cells.entry(cell_of(x, y)).or_default().push((x, y));
    }
  }
  let mut shuffled = source.clone();
  let at = |x: u32, y: u32| ((y * WIDTH + x) * 4) as usize;
  for points in cells.values() {
    for (to, from) in points.iter().zip(points.iter().rev()) {
      shuffled.rgba[at(to.0, to.1)..][..4].copy_from_slice(&source.rgba[at(from.0, from.1)..][..4]);
    }
  }
  shuffled
}

#[test]
fn blur_carries_nothing_finer_than_its_cells() {
  let annotation = redaction(AnnotationRedaction::Blur);
  for output in [identity(), resampled()] {
    let source = picture(1);
    let shuffled = shuffled_within_cells(&source, &output);
    assert!(shuffled.rgba != source.rgba);
    let blurred = composed(&source, output.clone(), annotation.clone());
    assert!(blurred == composed(&shuffled, output.clone(), annotation.clone()));
    // Not a flat fill in disguise: the cells' colours show.
    assert!(blurred != composed(&source, output, redaction(AnnotationRedaction::Erase)));
  }
}

#[test]
fn blur_takes_a_new_nudge_from_a_new_seed() {
  let source = picture(1);
  let annotation = redaction(AnnotationRedaction::Blur);
  let blurred = composed(&source, identity(), annotation.clone());
  assert!(composed(&source, identity(), annotation.clone()) == blurred);
  let mut reseeded = annotation;
  if let AnnotationShape::Redact { seed, .. } = &mut reseeded.shape {
    *seed ^= 0x5bd1_e995;
  }
  assert!(composed(&source, identity(), reseeded) != blurred);
}

/// Colour kept under zero alpha is invisible in the picture, so it must stay
/// invisible under a blur or a classic pixelation: a transparent pixel is
/// averaged as the surface it shows against, whatever it carries.
#[test]
fn colour_hidden_under_zero_alpha_never_reaches_the_averages() {
  let hidden = |red: u8| {
    let mut image = picture(1);
    for y in 30..40 {
      for x in 40..60 {
        image.rgba[((y * WIDTH + x) * 4) as usize..][..4].copy_from_slice(&[red, 0, 255 - red, 0]);
      }
    }
    image
  };
  for mode in [
    AnnotationRedaction::Blur,
    AnnotationRedaction::PixelateClassic,
  ] {
    let annotation = redaction(mode);
    let first = composed(&hidden(0), identity(), annotation.clone());
    assert!(
      first == composed(&hidden(255), identity(), annotation),
      "{mode:?} shows what alpha hid"
    );
  }
}

/// How far the centre of pixel `(x, y)` falls outside the painted box with
/// its corners rounded by `radius`.
fn outside(x: u32, y: u32, radius: f64) -> f64 {
  let [x0, y0, x1, y1] = PAINTED.map(f64::from);
  let half = [(x1 - x0) / 2.0, (y1 - y0) / 2.0];
  let q = [
    (f64::from(x) + 0.5 - (x0 + x1) / 2.0).abs() - (half[0] - radius),
    (f64::from(y) + 0.5 - (y0 + y1) / 2.0).abs() - (half[1] - radius),
  ];
  q[0].max(0.0).hypot(q[1].max(0.0)) + q[0].max(q[1]).min(0.0) - radius
}

/// Every pixel the rounded outline touches is filled whole, every pixel more
/// than one pixel wholly outside it keeps its picture, and the pixels
/// between - each wholly outside the outline - lie between the two.
#[test]
fn a_rounded_box_covers_whole_pixels_and_blends_only_outside_itself() {
  const FILL: [u8; 3] = [0x12, 0x34, 0x56];
  let source = picture(1);
  let mut annotation = redaction(AnnotationRedaction::Color);
  annotation.style.radius = 50.0;
  // Half the painted box's shorter side, thirty-six pixels.
  let radius = 18.0;
  let [x0, y0, x1, y1] = PAINTED;
  let mut expected = source.clone();
  let mut fringe = Vec::new();
  for y in y0..y1 {
    for x in x0..x1 {
      let distance = outside(x, y, radius);
      if distance <= std::f64::consts::FRAC_1_SQRT_2 {
        expected.rgba[((y * WIDTH + x) * 4) as usize..][..3].copy_from_slice(&FILL);
      } else if distance < 1.0 + std::f64::consts::FRAC_1_SQRT_2 {
        fringe.push((x, y));
      }
    }
  }
  // The top-left corner pixel is far outside a radius this large.
  assert!(outside(x0, y0, radius) > 2.0);
  let rounded = composed(&source, identity(), annotation);
  let mut plain = identity();
  plain.annotations = Vec::new();
  let expected = crate::screenshots::compose_output_layers(
    &expected, &plain, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap()
  .rgba;
  // The fill as the compositor draws it, from the middle of the box.
  let centre = (((y0 + y1) / 2 * WIDTH + (x0 + x1) / 2) * 4) as usize;
  let fill = [rounded[centre], rounded[centre + 1], rounded[centre + 2]];
  for y in 0..HEIGHT {
    for x in 0..WIDTH {
      let offset = ((y * WIDTH + x) * 4) as usize;
      let (got, want) = (&rounded[offset..offset + 3], &expected[offset..offset + 3]);
      if !fringe.contains(&(x, y)) {
        assert_eq!(got, want, "pixel {x}, {y}");
        continue;
      }
      for (channel, got) in got.iter().enumerate() {
        let (low, high) = (
          want[channel].min(fill[channel]),
          want[channel].max(fill[channel]),
        );
        assert!(
          (low..=high).contains(got),
          "fringe pixel {x}, {y}: {got} not between {low} and {high}"
        );
      }
    }
  }
}
