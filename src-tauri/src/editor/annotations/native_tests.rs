// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::{
  Annotation, AnnotationHead, AnnotationPoint, AnnotationShape, AnnotationStyle,
};

fn annotation(shape: AnnotationShape) -> Annotation {
  Annotation {
    above_camera: true,
    animated: true,
    id: "a".to_owned(),
    held: None,
    reveal: Default::default(),
    shape,
    style: AnnotationStyle {
      align: Default::default(),
      blur: false,
      color: "#0000ff".to_owned(),
      head: AnnotationHead::Both,
      hand_drawn: false,
      manual: false,
      radius: 0.0,
      redaction: Default::default(),
      softness: 0.0,
      strength: 0.0,
      width: 9.0,
    },
  }
}

#[test]
fn retains_counter_text_and_points() {
  let annotations = [annotation(AnnotationShape::Counter {
    center: AnnotationPoint { x: 1.0, y: 2.0 },
    value: 12,
    angle: 0.5,
  })];
  let native = native_annotations(&annotations, RedactSource::None, 1.0);
  assert_eq!(native.data.points.len(), 3);
  assert_eq!(native.data.points[0], [1.0, 2.0]);
  assert_eq!(native.data.text, b"12");
  assert_eq!(native.items[0].data_offset, 0);
  assert_eq!(native.items[0].data_count, 2);
}

/// The lists grow with the document: well past the thousand-odd a fixed
/// array once held, every annotation is still there, in order, with its
/// number where its record says.
#[test]
fn keeps_every_annotation_of_a_long_list() {
  let annotations: Vec<_> = (0..5_000_u32)
    .map(|value| {
      let mut item = annotation(AnnotationShape::Counter {
        center: AnnotationPoint {
          x: f64::from(value),
          y: 0.0,
        },
        value,
        angle: 0.0,
      });
      item.id = value.to_string();
      item
    })
    .collect();
  let native = native_annotations(&annotations, RedactSource::None, 1.0);
  assert_eq!(native.items.len(), 5_000);
  let last = native.items[4_999];
  assert_eq!(last.p0[0], 4_999.0);
  let start = last.data_offset as usize;
  assert_eq!(
    &native.data.text[start..start + last.data_count as usize],
    b"4999"
  );
}

/// A size is in points, so a 2x capture draws it twice as many pixels wide
/// and it looks the same as on a 1x one.
#[test]
fn a_size_is_drawn_at_the_capture_scale() {
  let counter = annotation(AnnotationShape::Counter {
    center: AnnotationPoint { x: 1.0, y: 2.0 },
    value: 1,
    angle: 0.0,
  });
  let width = |scale| {
    native_annotations(std::slice::from_ref(&counter), RedactSource::None, scale).items[0].width
  };
  assert_eq!(width(2.0), 2.0 * width(1.0));
}
