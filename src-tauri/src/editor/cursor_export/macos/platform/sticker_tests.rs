// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's stickers, through a real GPU dispatch: an emoji drawn by
//! the system's emoji font, turned, the placeholder for a picture that cannot
//! be found, the shadow, and the frame a moving picture shows.

use crate::editor::annotations::sticker::model::new_sticker;
use crate::editor::annotations::sticker::StickerArt;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

const SIZE: (u32, u32) = (240, 160);
const CENTRE: (u32, u32) = (120, 80);

/// A sticker of `asset`, `aspect` times as wide as it is tall, its longer
/// side 100 source pixels, in the middle of the canvas.
fn sticker(asset: &str, aspect: f64, angle: f64) -> Annotation {
  let art = StickerArt {
    asset: asset.to_owned(),
    aspect,
    pixels: None,
    play: None,
  };
  let centre = AnnotationPoint {
    x: f64::from(CENTRE.0),
    y: f64::from(CENTRE.1),
  };
  let mut annotation = new_sticker("sticker".to_owned(), centre, Some(&art), None, 0.0);
  if let AnnotationShape::Sticker {
    size, angle: turn, ..
  } = &mut annotation.shape
  {
    *size = 100.0;
    *turn = angle;
  }
  annotation
}

/// `annotation` composed onto a canvas of one `colour`, placed one source
/// pixel to one output pixel.
fn composed(annotation: Annotation, colour: [u8; 4]) -> crate::screenshots::CapturedImage {
  let source = crate::screenshots::CapturedImage {
    rgba: colour.repeat(SIZE.0 as usize * SIZE.1 as usize),
    width: SIZE.0,
    height: SIZE.1,
  };
  let mut settings = super::tests::output(SIZE.0, SIZE.1);
  settings.annotations = vec![annotation];
  crate::screenshots::compose_output_layers(
    &source, &settings, 0.0, false, None, None, None, None, false, false,
  )
  .unwrap()
}

fn pixel(image: &crate::screenshots::CapturedImage, x: u32, y: u32) -> [u8; 4] {
  let at = ((y * image.width + x) * 4) as usize;
  [
    image.rgba[at],
    image.rgba[at + 1],
    image.rgba[at + 2],
    image.rgba[at + 3],
  ]
}

fn is_red(colour: [u8; 4]) -> bool {
  colour[0] > 180 && colour[1] < 90 && colour[2] < 90
}

#[test]
fn a_sticker_is_drawn_from_its_picture_with_its_clear_corners_left_alone() {
  let drawn = composed(sticker("emoji:🔴", 1.0, 0.0), [0, 0, 0, 255]);
  assert!(is_red(pixel(&drawn, CENTRE.0, CENTRE.1)));
  // The circle's own picture is clear in its corners, which show the canvas.
  assert_eq!(pixel(&drawn, CENTRE.0 - 46, CENTRE.1 - 46), [0, 0, 0, 255]);
}

#[test]
fn a_turned_sticker_stands_its_long_side_the_way_it_was_turned() {
  // Twice as wide as tall, a quarter turn round: tall and narrow. The card
  // a missing picture is drawn as fills its whole box, where an emoji keeps
  // its own proportions inside it.
  let drawn = composed(
    sticker("image:missing", 2.0, std::f64::consts::FRAC_PI_2),
    [0, 0, 0, 255],
  );
  assert!(pixel(&drawn, CENTRE.0, CENTRE.1 - 45)[0] > 20);
  assert_eq!(pixel(&drawn, CENTRE.0 - 38, CENTRE.1), [0, 0, 0, 255]);
}

#[test]
fn a_sticker_whose_picture_is_missing_is_drawn_as_a_grey_card() {
  let drawn = composed(sticker("image:missing", 1.0, 0.0), [0, 0, 0, 255]);
  let [red, green, blue, _] = pixel(&drawn, CENTRE.0, CENTRE.1);
  assert!((20..100).contains(&red), "{red}");
  assert_eq!((red, green), (green, blue));
}

#[test]
fn a_sticker_casts_its_shadow_only_when_asked() {
  // Just past the bottom of the circle, which fills its box.
  let below = (CENTRE.0, CENTRE.1 + 53);
  let plain = pixel(
    &composed(sticker("emoji:🔴", 1.0, 0.0), [255; 4]),
    below.0,
    below.1,
  );
  // Clear of the circle the canvas shows through untouched.
  assert!(plain[0] >= 253 && plain[0] == plain[2], "{plain:?}");
  let mut shadowed = sticker("emoji:🔴", 1.0, 0.0);
  shadowed.style.shadow = true;
  let under = pixel(&composed(shadowed, [255; 4]), below.0, below.1);
  assert!(
    u16::from(under[0]) + 10 < u16::from(plain[0]) && under[0] == under[2],
    "{under:?}"
  );
}

#[test]
fn a_moving_sticker_shows_the_frame_its_moment_falls_in() {
  use crate::editor::stickers::animation::tests::gif;
  use crate::editor::stickers::store;

  let folder = std::env::temp_dir()
    .join("screenwide-tests")
    .join("sticker-frames");
  store::initialize(&folder, None);
  // Red for 100 ms, then blue for 100 ms.
  let art = store::import_bytes(&gif(&[[255, 0, 0, 255], [0, 0, 255, 255]], 100))
    .expect("a kept animation");
  let play = art.play.expect("it moves");
  assert_eq!((play.frames, play.cycle_ms), (2, 200.0));
  let at = |frame: u32, clock_ms: Option<f64>| {
    let mut annotation = sticker(&art.asset, art.aspect, 0.0);
    if let AnnotationShape::Sticker { play: shown, .. } = &mut annotation.shape {
      *shown = Some(crate::editor::annotations::sticker::StickerPlay {
        frame,
        clock_ms,
        ..play
      });
    }
    pixel(&composed(annotation, [0, 0, 0, 255]), CENTRE.0, CENTRE.1)
  };
  let blue = |colour: [u8; 4]| colour[2] > 180 && colour[0] < 90;
  // A still shows the frame chosen for it.
  assert!(is_red(at(0, None)));
  assert!(blue(at(1, None)));
  // Playing, the frame follows the clip's clock and loops.
  assert!(is_red(at(0, Some(50.0))));
  assert!(blue(at(0, Some(150.0))));
  assert!(is_red(at(0, Some(250.0))));
  // Starting on the second frame shifts the run.
  assert!(is_red(at(1, Some(150.0))));
}
