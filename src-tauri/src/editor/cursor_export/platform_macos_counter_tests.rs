// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's counter annotations, through a real Metal dispatch.

use crate::editor::annotations::counter::new_counter;
use crate::editor::annotations::counter::reveal::counter_reveal_window;
use crate::editor::annotations::counter::silhouette::COUNTER_TAIL_REACH;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationStyle};

const SIZE: (u32, u32) = (480, 320);
const CENTER: AnnotationPoint = AnnotationPoint { x: 240.0, y: 160.0 };
const DIAMETER: f64 = 80.0;

/// A counter on a black canvas, in the palette's yellow so the number's dark
/// tint reads against its own disc.
fn counter(value: u32, angle: f64) -> Annotation {
  let mut annotation = new_counter("counter".to_owned(), CENTER, value, None, None);
  annotation.style = AnnotationStyle {
    color: "#ffcc00".to_owned(),
    head: crate::editor::annotations::AnnotationHead::None,
    width: DIAMETER,
  };
  if let crate::editor::annotations::AnnotationShape::Counter { angle: aim, .. } =
    &mut annotation.shape
  {
    *aim = angle;
  }
  annotation
}

fn composed(annotation: Annotation) -> crate::screenshots::CapturedImage {
  let source = crate::screenshots::CapturedImage {
    rgba: [0, 0, 0, 255].repeat(SIZE.0 as usize * SIZE.1 as usize),
    width: SIZE.0,
    height: SIZE.1,
  };
  let mut settings = crate::screenshots::test_output_settings(SIZE.0, SIZE.1);
  settings.background_color = "#000000".to_owned();
  settings.background_type = "solid".to_owned();
  settings.crop_height = f64::from(SIZE.1);
  settings.crop_width = f64::from(SIZE.0);
  settings.crop_x = 0.0;
  settings.crop_y = 0.0;
  settings.image_width = f64::from(SIZE.0);
  settings.image_x = 0.0;
  settings.image_y = 0.0;
  settings.annotations = vec![annotation];
  let rgba = crate::screenshots::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap();
  crate::screenshots::CapturedImage {
    rgba: rgba.rgba,
    width: SIZE.0,
    height: SIZE.1,
  }
}

fn pixel(image: &crate::screenshots::CapturedImage, x: f64, y: f64) -> [u8; 4] {
  let start = ((y.round() as usize) * SIZE.0 as usize + x.round() as usize) * 4;
  [
    image.rgba[start],
    image.rgba[start + 1],
    image.rgba[start + 2],
    image.rgba[start + 3],
  ]
}

/// Whether a pixel carries the disc's yellow.
fn yellow(sample: [u8; 4]) -> bool {
  sample[0] > 240 && sample[1] > 180 && sample[2] < 32
}

/// Writes a case out for the eyeball pass, the way the arrow suite does.
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

#[test]
fn a_counter_draws_a_disc_with_its_tail_and_number() {
  let image = composed(counter(1, 0.0));
  write_png("counter-1-east", &image);
  // Inside the disc, clear of the number.
  assert!(
    yellow(pixel(&image, CENTER.x - DIAMETER * 0.4, CENTER.y)),
    "the disc is {:?}",
    pixel(&image, CENTER.x - DIAMETER * 0.4, CENTER.y)
  );
  // The tail reaches past the disc on the side it points at, and nowhere
  // else: the same distance out to the left is background.
  let reach = DIAMETER * 0.5 * (COUNTER_TAIL_REACH - 0.15);
  assert!(
    yellow(pixel(&image, CENTER.x + reach, CENTER.y)),
    "the tail is {:?}",
    pixel(&image, CENTER.x + reach, CENTER.y)
  );
  assert_eq!(pixel(&image, CENTER.x - reach, CENTER.y)[0..3], [0, 0, 0]);
  // The number is inked over the disc: the centre column carries a pixel far
  // darker than the yellow it sits on.
  let mut inked = false;
  for step in 0..40 {
    let y = CENTER.y - 20.0 + f64::from(step);
    let sample = pixel(&image, CENTER.x, y);
    inked = inked || sample[0] < 128;
  }
  assert!(inked, "the number left no ink on the disc");
}

#[test]
fn the_tail_follows_the_angle_it_is_given() {
  let reach = DIAMETER * 0.5 * (COUNTER_TAIL_REACH - 0.15);
  for (label, angle, offset) in [
    ("south", std::f64::consts::FRAC_PI_2, (0.0, reach)),
    ("north", -std::f64::consts::FRAC_PI_2, (0.0, -reach)),
    ("west", std::f64::consts::PI, (-reach, 0.0)),
  ] {
    let image = composed(counter(2, angle));
    write_png(&format!("counter-2-{label}"), &image);
    let at = pixel(&image, CENTER.x + offset.0, CENTER.y + offset.1);
    assert!(yellow(at), "the {label} tail is {at:?}");
    let opposite = pixel(&image, CENTER.x - offset.0, CENTER.y - offset.1);
    assert_eq!(
      opposite[0..3],
      [0, 0, 0],
      "the {label} tail drew on both sides"
    );
  }
}

#[test]
fn a_two_digit_counter_keeps_its_number_inside_the_disc() {
  let image = composed(counter(42, 0.0));
  write_png("counter-42", &image);
  // The edge of the disc, a tenth of the radius in from the left, is still
  // the disc's own colour: a number too wide for its disc is narrowed.
  let at = pixel(&image, CENTER.x - DIAMETER * 0.45, CENTER.y);
  assert!(yellow(at), "the number reached the edge: {at:?}");
}

#[test]
fn a_counter_arrives_by_growing_into_place() {
  let mut annotation = counter(3, 0.0);
  annotation.reveal = counter_reveal_window(0.0, 2_000.0, 0.0);
  let arriving = composed(annotation);
  write_png("counter-3-arriving", &arriving);
  // At the very start it is transparent, and smaller than it will be: the
  // edge of the full-size disc is background.
  let edge = pixel(&arriving, CENTER.x - DIAMETER * 0.48, CENTER.y);
  assert_eq!(edge[0..3], [0, 0, 0], "the arrival started full size");
  assert!(
    pixel(&arriving, CENTER.x, CENTER.y)[3] > 0,
    "nothing was drawn at all"
  );
  let mut whole = counter(3, 0.0);
  whole.reveal = counter_reveal_window(1_000.0, 2_000.0, 0.0);
  let held = composed(whole);
  assert!(
    yellow(pixel(&held, CENTER.x - DIAMETER * 0.4, CENTER.y)),
    "the held frame is not whole"
  );
}

#[test]
fn a_counter_smears_over_the_sizes_it_grew_through() {
  // A frame's worth of growth part way through the arrival: the disc covered
  // every size between the two, so its edge is a band rather than a step.
  let window = counter_reveal_window(90.0, 2_000.0, 33.0);
  assert!(window.scale > window.previous[2]);
  let mut annotation = counter(4, 0.0);
  annotation.reveal = window;
  let moving = composed(annotation);
  write_png("counter-4-smear", &moving);
  let mut held = counter(4, 0.0);
  held.reveal = counter_reveal_window(90.0, 2_000.0, 0.0);
  let standing = composed(held);
  write_png("counter-4-standing", &standing);
  let radius = |scale: f32| DIAMETER * 0.5 * f64::from(scale);
  // Half way between the size it had a frame ago and the size it has now:
  // only the later samples reached this far, so the moving frame covers it
  // part way while the standing one covers it whole. Measured away from the
  // tail, which reaches far past the disc on the side it points at, and read
  // as ink over black - the canvas is opaque, so alpha says nothing.
  let band = (radius(window.previous[2]) + radius(window.scale)) / 2.0;
  let smeared = pixel(&moving, CENTER.x - band, CENTER.y)[0];
  let stepped = pixel(&standing, CENTER.x - band, CENTER.y)[0];
  assert!(
    smeared + 24 < stepped,
    "the exposure left no smear: {smeared} against {stepped}"
  );
  // Well inside the size it had a frame ago, every sample covers the pixel,
  // so the smear is a band at the edge rather than a hole in the middle.
  let inner = radius(window.previous[2]) * 0.5;
  assert!(pixel(&moving, CENTER.x - inner, CENTER.y)[0] > smeared + 24);
  // And the annotation is drawn at the opacity its reveal is at rather than
  // whole, which is what a video export was missing.
  let inside = pixel(&moving, CENTER.x - inner, CENTER.y)[0];
  assert!(inside > 16 && inside < 240, "the disc is solid: {inside}");
}
