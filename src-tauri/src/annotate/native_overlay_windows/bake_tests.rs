// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::model::new_arrow;
use crate::editor::annotations::{AnnotationHead, AnnotationPoint, AnnotationStyle};

const SIDE: u32 = 128;

fn still() -> CapturedImage {
  CapturedImage {
    width: SIDE,
    height: SIDE,
    rgba: (0..SIDE * SIDE).flat_map(|_| [0, 0, 0, 255]).collect(),
  }
}

fn arrow() -> Annotation {
  new_arrow(
    "baked".to_owned(),
    AnnotationPoint { x: 20.0, y: 20.0 },
    AnnotationPoint { x: 100.0, y: 100.0 },
    Some(&AnnotationStyle {
      color: "#ff0000".to_owned(),
      head: AnnotationHead::End,
      width: 8.0,
    }),
  )
}

fn pixel(image: &CapturedImage, x: u32, y: u32) -> &[u8] {
  let start = ((y * image.width + x) * 4) as usize;
  &image.rgba[start..start + 4]
}

#[test]
fn the_arrow_lands_in_the_still_in_its_own_colour() {
  let baked = bake(&still(), &[arrow()]).expect("the still bakes");
  // On the shaft, half way along it: the arrow's own red, in RGBA order, over
  // an opaque still.
  assert_eq!(pixel(&baked, 60, 60), [255, 0, 0, 255]);
  // Past the tip, and well away from the shaft: the still as it was.
  assert_eq!(pixel(&baked, 120, 120), [0, 0, 0, 255]);
  assert_eq!(pixel(&baked, 8, 118), [0, 0, 0, 255]);
}

#[test]
fn a_still_with_nothing_drawn_over_it_is_returned_as_it_was() {
  let original = still();
  let baked = bake(&original, &[]).expect("an empty list bakes");
  assert_eq!(baked.rgba, original.rgba);
}
