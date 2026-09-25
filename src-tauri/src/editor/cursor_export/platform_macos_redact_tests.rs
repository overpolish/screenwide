// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Redactions through a real Metal dispatch: erase and colour carry nothing
//! of what is under the box, and pixelate carries its colours but not where
//! they were, whatever the picture and however it is resampled onto its
//! canvas.

use crate::editor::annotations::redact::new_redact;
use crate::editor::annotations::{
  Annotation, AnnotationPoint, AnnotationRedaction, AnnotationShape,
};
use crate::screenshots::{CapturedImage, ScreenshotOutputSettings};

pub(super) const WIDTH: u32 = 120;
pub(super) const HEIGHT: u32 = 90;
/// The box, in source pixels. Its corners are deliberately fractional: the
/// compositor snaps them outward, so the two half-covered columns and rows
/// are covered whole.
const BOX: [f64; 4] = [30.4, 20.6, 81.2, 55.5];
const GREY: u8 = 200;
const INK: [u8; 3] = [20, 40, 90];

/// A flat grey picture with `secret` texture under the box and a margin
/// round it, so both the covered pixels and the ones the box's snapped edges
/// swallow differ between two secrets.
pub(super) fn picture(secret: u32) -> CapturedImage {
  let mut rgba = [GREY, GREY, GREY, 255].repeat((WIDTH * HEIGHT) as usize);
  for y in 20..56 {
    for x in 30..82 {
      let value = (x * 7 + y * 13 + secret * 31) % 256;
      rgba[((y * WIDTH + x) * 4) as usize..][..3].copy_from_slice(&[
        value as u8,
        (value * 3 % 256) as u8,
        secret as u8,
      ]);
    }
  }
  CapturedImage {
    rgba,
    width: WIDTH,
    height: HEIGHT,
  }
}

/// Grey with `INK` text under the box, scattered by `layout`: two layouts
/// share the same two colours and differ in where every glyph pixel sits.
fn text(layout: u32) -> CapturedImage {
  let mut image = picture(0);
  for y in 20_u32..56 {
    for x in 30_u32..82 {
      let spot =
        (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ layout.wrapping_mul(83_492_791))
          % 7;
      let colour = if spot < 2 { INK } else { [GREY; 3] };
      image.rgba[((y * WIDTH + x) * 4) as usize..][..3].copy_from_slice(&colour);
    }
  }
  image
}

/// The box over `BOX`, with a fixed seed so every run lays the blocks out the
/// same way.
pub(super) fn redaction(mode: AnnotationRedaction) -> Annotation {
  let point = |x, y| AnnotationPoint { x, y };
  let mut annotation = new_redact("redaction".to_owned(), point(BOX[0], BOX[1]), None);
  if let AnnotationShape::Redact { end, seed, .. } = &mut annotation.shape {
    *end = point(BOX[2], BOX[3]);
    *seed = 0x1234_5678;
  }
  annotation.style.redaction = mode;
  annotation.style.color = "#123456".to_owned();
  annotation.style.width = 4.0;
  annotation
}

/// A canvas the size of the source, the picture drawn one pixel to one.
pub(super) fn identity() -> ScreenshotOutputSettings {
  let mut output = crate::screenshots::test_output_settings(WIDTH, HEIGHT);
  output.background_type = "solid".to_owned();
  output.crop_height = f64::from(HEIGHT);
  output.crop_width = f64::from(WIDTH);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(WIDTH);
  output.image_x = 0.0;
  output.image_y = 0.0;
  output
}

/// The same canvas with the picture shrunk and nudged off the pixel grid, so
/// every output pixel near the box blends several source pixels.
pub(super) fn resampled() -> ScreenshotOutputSettings {
  let mut output = identity();
  output.image_width = f64::from(WIDTH) * 0.73;
  output.image_x = 13.4;
  output.image_y = 9.7;
  output
}

pub(super) fn composed(
  source: &CapturedImage,
  mut output: ScreenshotOutputSettings,
  annotation: Annotation,
) -> Vec<u8> {
  output.annotations = vec![annotation];
  crate::screenshots::compose_output_layers(
    source, &output, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap()
  .rgba
}

pub(super) fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 3] {
  let offset = ((y * WIDTH + x) * 4) as usize;
  [rgba[offset], rgba[offset + 1], rgba[offset + 2]]
}

#[test]
fn erase_and_colour_carry_nothing_of_what_they_cover() {
  for mode in [AnnotationRedaction::Erase, AnnotationRedaction::Color] {
    let annotation = redaction(mode);
    for output in [identity(), resampled()] {
      let first = composed(&picture(1), output.clone(), annotation.clone());
      let second = composed(&picture(2), output, annotation.clone());
      assert!(first == second, "{mode:?} leaks what it covers");
    }
  }
}

fn distance(a: [u8; 3], b: [u8; 3]) -> i32 {
  a.iter()
    .zip(b)
    .map(|(a, b)| (i32::from(*a) - i32::from(b)).abs())
    .sum()
}

/// The depixelation guarantee: text laid out differently inside each zone,
/// in the same colours, pixelates to the very same pixels.
#[test]
fn pixelate_carries_each_zones_colours_and_none_of_the_layout_within_it() {
  let annotation = redaction(AnnotationRedaction::Pixelate);
  for output in [identity(), resampled()] {
    let first = composed(&text(1), output.clone(), annotation.clone());
    let second = composed(&text(2), output, annotation.clone());
    assert!(first == second, "pixelate leaks where the text was");
  }
}

/// Zones as tall as the box, thirty-six pixels, from its corner at x 30:
/// text in one colour over the first zone and another over the second lands
/// in each colour's own zone.
#[test]
fn pixelate_puts_each_colour_where_it_was_to_the_nearest_zone() {
  const SECOND: [u8; 3] = [230, 220, 40];
  let mut source = text(1);
  for y in 20_u32..56 {
    for x in 66_u32..82 {
      let offset = ((y * WIDTH + x) * 4) as usize;
      if source.rgba[offset..offset + 3] == INK {
        source.rgba[offset..offset + 3].copy_from_slice(&SECOND);
      }
    }
  }
  let pixelated = composed(
    &source,
    identity(),
    redaction(AnnotationRedaction::Pixelate),
  );
  let blocks = |from: u32, to: u32| {
    (20..56)
      .step_by(4)
      .flat_map(move |y| (from..to).step_by(4).map(move |x| (x, y)))
      .map(|(x, y)| pixel(&pixelated, x, y))
      .collect::<Vec<_>>()
  };
  // A block leans towards a colour when it is nearer that colour than the
  // surface and the other ink both.
  let leans = |shade: [u8; 3], towards: [u8; 3]| {
    [[GREY; 3], INK, SECOND]
      .into_iter()
      .filter(|other| *other != towards)
      .all(|other| distance(shade, towards) < distance(shade, other))
  };
  let (left, right) = (blocks(30, 66), blocks(66, 82));
  assert!(left.iter().any(|shade| leans(*shade, INK)));
  assert!(!left.iter().any(|shade| leans(*shade, SECOND)));
  assert!(right.iter().any(|shade| leans(*shade, SECOND)));
  assert!(!right.iter().any(|shade| leans(*shade, INK)));
}

/// The whole export, not just the box, matches a picture that never held the
/// secret: the box filled exactly over its snapped pixels, nothing else
/// touched. Compared through the compositor, whose own rounding applies to
/// both.
#[test]
fn erase_and_colour_fill_the_snapped_box_and_nothing_else() {
  let source = picture(1);
  let filled = |fill: [u8; 3]| {
    let mut plain = picture(1);
    for y in 20..56 {
      for x in 30..82 {
        plain.rgba[((y * WIDTH + x) * 4) as usize..][..3].copy_from_slice(&fill);
      }
    }
    let mut output = identity();
    output.annotations = Vec::new();
    crate::screenshots::compose_output_layers(
      &plain, &output, 0.0, false, None, None, None, None, false, false,
    )
    .unwrap()
    .rgba
  };
  let erased = composed(&source, identity(), redaction(AnnotationRedaction::Erase));
  assert!(erased == filled([GREY; 3]));
  let coloured = composed(&source, identity(), redaction(AnnotationRedaction::Color));
  assert!(coloured == filled([0x12, 0x34, 0x56]));
}

#[test]
fn pixelate_draws_blocks_that_a_new_seed_lays_out_again() {
  let source = text(1);
  let annotation = redaction(AnnotationRedaction::Pixelate);
  let pixelated = composed(&source, identity(), annotation.clone());
  // Blocks of four source pixels, counted from the snapped corner.
  assert_eq!(pixel(&pixelated, 30, 20), pixel(&pixelated, 33, 23));
  let shades: std::collections::HashSet<_> = (0..12)
    .flat_map(|block_y| (0..8).map(move |block_x| (30 + block_x * 4, 20 + block_y * 3)))
    .map(|(x, y)| pixel(&pixelated, x, y))
    .collect();
  assert!(shades.len() > 4, "{shades:?}");
  // The same seed draws the same blocks every time, and another seed others.
  assert!(composed(&source, identity(), annotation.clone()) == pixelated);
  let mut reseeded = annotation;
  if let AnnotationShape::Redact { seed, .. } = &mut reseeded.shape {
    *seed ^= 0x5bd1_e995;
  }
  assert!(composed(&source, identity(), reseeded) != pixelated);
}
