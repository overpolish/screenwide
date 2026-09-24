// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The compositor's text boxes, through a real Metal dispatch.

use crate::editor::annotations::text::geometry::box_size;
use crate::editor::annotations::text::metrics::text_block;
use crate::editor::annotations::text::model::TextPointer;
use crate::editor::annotations::text::new_text;
use crate::editor::annotations::{
  Annotation, AnnotationAlign, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

const SIZE: (u32, u32) = (480, 320);
const ORIGIN: AnnotationPoint = AnnotationPoint { x: 60.0, y: 60.0 };
const FONT: f64 = 28.0;

fn text_box(text: &str, pointer: TextPointer, color: &str) -> Annotation {
  let mut annotation = new_text("text".to_owned(), ORIGIN, None);
  annotation.style = AnnotationStyle {
    align: AnnotationAlign::Left,
    color: color.to_owned(),
    head: AnnotationHead::None,
    width: FONT,
  };
  annotation.shape = AnnotationShape::Text {
    origin: ORIGIN,
    pointer,
    text: text.to_owned(),
  };
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

fn pixel(image: &crate::screenshots::CapturedImage, x: f64, y: f64) -> [u8; 3] {
  let start = ((y.round() as usize) * image.width as usize + x.round() as usize) * 4;
  [
    image.rgba[start],
    image.rgba[start + 1],
    image.rgba[start + 2],
  ]
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

/// The box's size in the canvas, which is the output here.
fn box_extent(text: &str) -> [f64; 2] {
  box_size(text_block(text, FONT), FONT)
}

/// Whether any pixel of a row across the box's text band is `ink`.
fn inked(
  image: &crate::screenshots::CapturedImage,
  text: &str,
  ink: impl Fn([u8; 3]) -> bool,
) -> bool {
  let extent = box_extent(text);
  let y = ORIGIN.y + extent[1] * 0.5;
  (0..extent[0] as u32).any(|step| {
    let sample = pixel(image, ORIGIN.x + f64::from(step), y);
    ink(sample)
  })
}

#[test]
fn a_text_box_draws_its_box_and_inks_its_text_to_read_on_it() {
  let text = "Click here";
  let extent = box_extent(text);
  let light = composed(text_box(text, TextPointer::default(), "#ffcc00"));
  write_png("text-yellow", &light);
  // The box's padding is the box's colour, and outside it is the canvas.
  let padding = pixel(&light, ORIGIN.x + 4.0, ORIGIN.y + extent[1] * 0.5);
  assert!(
    padding[0] > 240 && padding[2] < 32,
    "the box is {padding:?}"
  );
  assert_eq!(
    pixel(&light, ORIGIN.x - 4.0, ORIGIN.y + extent[1] * 0.5),
    [0, 0, 0]
  );
  assert_eq!(
    pixel(&light, ORIGIN.x + extent[0] + 4.0, ORIGIN.y + 4.0),
    [0, 0, 0]
  );
  // Yellow is light, so the text is inked dark.
  assert!(
    inked(&light, text, |sample| sample[0] < 64),
    "no dark ink on yellow"
  );
  // A dark box takes light ink instead.
  let dark = composed(text_box(text, TextPointer::default(), "#1c1c1e"));
  write_png("text-dark", &dark);
  assert!(
    inked(&dark, text, |sample| sample[0] > 192),
    "no light ink on a dark box"
  );
}

#[test]
fn a_pointer_reaches_the_tip_it_is_given_and_nowhere_else() {
  let text = "Menu";
  let extent = box_extent(text);
  // Four and a bit ems straight out of the middle of the right edge.
  let out = TextPointer {
    along: AnnotationPoint { x: 1.0, y: 0.0 },
    reach: AnnotationPoint {
      x: 120.0 / FONT,
      y: 0.0,
    },
  };
  let tip = AnnotationPoint {
    x: ORIGIN.x + extent[0] + 120.0,
    y: ORIGIN.y + extent[1] * 0.5,
  };
  let image = composed(text_box(text, out, "#ffcc00"));
  write_png("text-pointer", &image);
  // Just short of the tip is the pointer's colour.
  let near = pixel(&image, tip.x - 3.0, tip.y);
  assert!(
    near[0] > 200 && near[2] < 64,
    "the pointer's tip is {near:?}"
  );
  // Past it, and on the far side of the box, is the canvas.
  assert_eq!(pixel(&image, tip.x + 3.0, tip.y), [0, 0, 0]);
  assert_eq!(pixel(&image, ORIGIN.x - 20.0, tip.y), [0, 0, 0]);
  // Without the pointer the same place is canvas.
  let bare = composed(text_box(text, TextPointer::default(), "#ffcc00"));
  assert_eq!(pixel(&bare, tip.x - 3.0, tip.y), [0, 0, 0]);
}
