// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Blur and rounded corners through a real Metal dispatch: a blur is a
//! Gaussian over the box's own pixels, and a rounded box covers every pixel
//! its outline touches whole, blending only pixels wholly outside it.

use super::redact_tests::{composed, identity, picture, redaction, HEIGHT, WIDTH};
use crate::editor::annotations::redact::cells::blur_deviation;
use crate::editor::annotations::redact::native::source_per_capture_point;
use crate::editor::annotations::AnnotationRedaction;
use crate::screenshots::CapturedImage;

/// The box's pixels once the compositor snaps its corners outward.
const PAINTED: [u32; 4] = [30, 20, 82, 56];

/// `source` blurred by a Gaussian of deviation `sigma` over the painted box
/// alone, rows then columns, each tap renormalised within the box: what the
/// box's pixels should become, in 0 to 255.
fn gaussian(source: &CapturedImage, sigma: f64) -> Vec<[f64; 3]> {
  let [x0, y0, x1, y1] = PAINTED;
  let (wide, tall) = ((x1 - x0) as i64, (y1 - y0) as i64);
  let reach = (sigma * 3.0).ceil() as i64;
  let weight = |offset: i64| (-((offset * offset) as f64) / (2.0 * sigma * sigma)).exp();
  let blur = |at: i64, extent: i64, read: &dyn Fn(i64) -> [f64; 3]| {
    let (mut sum, mut total) = ([0.0; 3], 0.0);
    for tap in (at - reach).max(0)..=(at + reach).min(extent - 1) {
      let value = read(tap);
      let w = weight(tap - at);
      for channel in 0..3 {
        sum[channel] += w * value[channel];
      }
      total += w;
    }
    sum.map(|channel| channel / total)
  };
  let source_at = |x: i64, y: i64| {
    let offset = (((y0 as i64 + y) * WIDTH as i64 + x0 as i64 + x) * 4) as usize;
    [0, 1, 2].map(|channel| f64::from(source.rgba[offset + channel]))
  };
  let rows: Vec<[f64; 3]> = (0..tall)
    .flat_map(|y| (0..wide).map(move |x| (x, y)))
    .map(|(x, y)| blur(x, wide, &|tap| source_at(tap, y)))
    .collect();
  (0..tall)
    .flat_map(|y| (0..wide).map(move |x| (x, y)))
    .map(|(x, y)| blur(y, tall, &|tap| rows[(tap * wide + x) as usize]))
    .collect()
}

#[test]
fn blur_is_a_gaussian_over_the_box_alone() {
  let source = picture(1);
  let output = identity();
  let sigma = blur_deviation(
    3.0,
    source_per_capture_point(WIDTH, output.capture_width_points),
  );
  let blurred = composed(&source, output, redaction(AnnotationRedaction::Blur));
  let expected = gaussian(&source, sigma);
  let [x0, y0, x1, y1] = PAINTED;
  let mut worst = 0.0_f64;
  for y in y0..y1 {
    for x in x0..x1 {
      let offset = ((y * WIDTH + x) * 4) as usize;
      let want = expected[((y - y0) * (x1 - x0) + x - x0) as usize];
      for channel in 0..3 {
        worst = worst.max((f64::from(blurred[offset + channel]) - want[channel]).abs());
      }
    }
  }
  // Half-float rows and two roundings to 8 bits leave a level or two.
  assert!(worst <= 2.0, "off a Gaussian by {worst}");
  // Outside the box the picture is as it is with no redaction at all.
  let plain = crate::screenshots::compose_output_layers(
    &source,
    &identity(),
    0.0,
    false,
    None,
    None,
    None,
    None,
    false,
    false,
  )
  .unwrap()
  .rgba;
  for (x, y) in [(x0 - 1, y0), (x1, y1 - 1), (x0, y1), (x1 - 1, y0 - 1)] {
    let offset = ((y * WIDTH + x) * 4) as usize;
    assert_eq!(blurred[offset..offset + 3], plain[offset..offset + 3]);
  }
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
