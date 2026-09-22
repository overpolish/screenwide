// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-space annotations retained by the native compositor.

use super::reveal::AnnotationReveal;
use super::{Annotation, MAX_ANNOTATIONS, MAX_ANNOTATION_POINTS, MAX_ANNOTATION_TEXT};
#[cfg(target_os = "windows")]
use crate::editor::annotations::AnnotationKind;
use crate::editor::annotations::{annotation_colour, AnnotationHead};

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct NativeAnnotation {
  pub(crate) kind: u32,
  pub(crate) head: u32,
  pub(crate) above_camera: u32,
  pub(crate) flags: u32,
  pub(crate) width: f32,
  pub(crate) params: [f32; 3],
  pub(crate) color: [f32; 4],
  pub(crate) p0: [f32; 2],
  pub(crate) p1: [f32; 2],
  pub(crate) p2: [f32; 2],
  pub(crate) p3: [f32; 2],
  pub(crate) data_offset: u32,
  pub(crate) data_count: u32,
  pub(crate) hover: f32,
  pub(crate) animated: u32,
  pub(crate) reveal: AnnotationReveal,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotation>() == 128);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, color) == 32);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, p0) == 48);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, hover) == 88);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, animated) == 92);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, reveal) == 96);

#[cfg(target_os = "windows")]
impl NativeAnnotation {
  pub(crate) fn shape_kind(&self) -> AnnotationKind {
    AnnotationKind::from_raw(self.kind).unwrap_or(AnnotationKind::Arrow)
  }
}

/// The side buffers every kind's variable-length data lives in: a points
/// kind indexes `points`, a text kind indexes `text`, through its record's
/// `data_offset` and `data_count`. Kept apart from the items so a list that
/// is longer than one scene - the video export's clips - can share one set
/// of buffers and pass it beside the list.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NativeAnnotationData {
  pub(crate) points: [[f32; 2]; MAX_ANNOTATION_POINTS],
  pub(crate) text: [u8; MAX_ANNOTATION_TEXT],
  pub(crate) point_count: u32,
  pub(crate) text_len: u32,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotationData>() == 32 * 1024 + 4096 + 8);

impl Default for NativeAnnotationData {
  fn default() -> Self {
    Self {
      points: [[0.0; 2]; MAX_ANNOTATION_POINTS],
      text: [0; MAX_ANNOTATION_TEXT],
      point_count: 0,
      text_len: 0,
    }
  }
}

impl NativeAnnotationData {
  /// Write `annotation`'s variable-length data into the buffers and return
  /// its record, with the offsets pointing here. Data that would overflow a
  /// buffer is dropped, in order, the way `MAX_ANNOTATIONS` drops items.
  pub(crate) fn pack(&mut self, annotation: &Annotation) -> NativeAnnotation {
    let [p0, p1, p2] = annotation.shape.draw_points();
    let point_offset = self.point_count as usize;
    if point_offset + 3 <= MAX_ANNOTATION_POINTS {
      self.points[point_offset..point_offset + 3].copy_from_slice(&[p0, p1, p2]);
      self.point_count += 3;
    }
    let text = match &annotation.shape {
      super::shape::AnnotationShape::Counter { value, .. } => value.to_string(),
      super::shape::AnnotationShape::Arrow { .. } => String::new(),
    };
    let text_bytes = text.as_bytes();
    let data_offset = self.text_len as usize;
    let data_count = text_bytes
      .len()
      .min(MAX_ANNOTATION_TEXT.saturating_sub(data_offset));
    if data_count > 0 {
      self.text[data_offset..data_offset + data_count].copy_from_slice(&text_bytes[..data_count]);
      self.text_len += data_count as u32;
    }
    NativeAnnotation {
      kind: annotation.shape.kind().raw(),
      head: match annotation.style.head {
        AnnotationHead::None => 0,
        AnnotationHead::End => 1,
        AnnotationHead::Both => 2,
      },
      above_camera: u32::from(annotation.above_camera),
      flags: 0,
      width: annotation.style.width.max(0.0) as f32,
      params: [0.0; 3],
      color: annotation_colour(&annotation.style.color),
      p0,
      p1,
      p2,
      p3: [0.0; 2],
      data_offset: data_offset as u32,
      data_count: data_count as u32,
      hover: 0.0,
      animated: u32::from(annotation.animated),
      reveal: annotation.reveal,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NativeAnnotations {
  pub(crate) items: [NativeAnnotation; MAX_ANNOTATIONS],
  pub(crate) count: u32,
  pub(crate) data: NativeAnnotationData,
}

const _: () = assert!(
  std::mem::size_of::<NativeAnnotations>() == 128 * MAX_ANNOTATIONS + 4 + 32 * 1024 + 4096 + 8
);

impl Default for NativeAnnotations {
  fn default() -> Self {
    Self {
      items: [NativeAnnotation::default(); MAX_ANNOTATIONS],
      count: 0,
      data: NativeAnnotationData::default(),
    }
  }
}

pub(crate) fn native_annotations(annotations: &[Annotation]) -> NativeAnnotations {
  let mut native = NativeAnnotations::default();
  for (index, annotation) in annotations.iter().take(MAX_ANNOTATIONS).enumerate() {
    native.items[index] = native.data.pack(annotation);
    native.count = index as u32 + 1;
  }
  native
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle};

  fn annotation(shape: AnnotationShape) -> Annotation {
    Annotation {
      above_camera: true,
      animated: true,
      id: "a".to_owned(),
      reveal: Default::default(),
      shape,
      style: AnnotationStyle {
        color: "#0000ff".to_owned(),
        head: AnnotationHead::Both,
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
    let native = native_annotations(&annotations);
    assert_eq!(native.data.point_count, 3);
    assert_eq!(native.data.points[0], [1.0, 2.0]);
    assert_eq!(native.data.text_len, 2);
    assert_eq!(&native.data.text[..2], b"12");
    assert_eq!(native.items[0].data_offset, 0);
    assert_eq!(native.items[0].data_count, 2);
  }
}
