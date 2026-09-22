// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-space annotations handed to the native compositors.
//!
//! The lists are owned here and grow with the document; the native side reads
//! them through a borrowed view of pointers and lengths, so no list size is
//! baked into the boundary.

use super::reveal::AnnotationReveal;
use super::Annotation;
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
#[derive(Clone, Default)]
pub(crate) struct NativeAnnotationData {
  pub(crate) points: Vec<[f32; 2]>,
  pub(crate) text: Vec<u8>,
}

impl NativeAnnotationData {
  /// Write `annotation`'s variable-length data into the buffers and return
  /// its record, with the offsets pointing here.
  pub(crate) fn pack(&mut self, annotation: &Annotation) -> NativeAnnotation {
    let [p0, p1, p2] = annotation.shape.draw_points();
    self.points.extend_from_slice(&[p0, p1, p2]);
    let text = match &annotation.shape {
      super::shape::AnnotationShape::Counter { value, .. } => value.to_string(),
      super::shape::AnnotationShape::Arrow { .. } => String::new(),
    };
    let data_offset = self.text.len();
    self.text.extend_from_slice(text.as_bytes());
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
      data_offset: ffi_len(data_offset),
      data_count: ffi_len(text.len()),
      hover: 0.0,
      animated: u32::from(annotation.animated),
      reveal: annotation.reveal,
    }
  }

  #[cfg(target_os = "macos")]
  pub(crate) fn view(&self) -> NativeAnnotationDataView {
    NativeAnnotationDataView {
      points: self.points.as_ptr(),
      text: self.text.as_ptr(),
      point_count: ffi_len(self.points.len()),
      text_len: ffi_len(self.text.len()),
    }
  }
}

/// One composition's annotations and the side buffers they index.
#[derive(Clone, Default)]
pub(crate) struct NativeAnnotations {
  pub(crate) items: Vec<NativeAnnotation>,
  pub(crate) data: NativeAnnotationData,
}

#[cfg(target_os = "macos")]
impl NativeAnnotations {
  pub(crate) fn view(&self) -> NativeAnnotationsView {
    NativeAnnotationsView {
      items: self.items.as_ptr(),
      count: ffi_len(self.items.len()),
      data: self.data.view(),
    }
  }
}

pub(crate) fn native_annotations(annotations: &[Annotation]) -> NativeAnnotations {
  let mut data = NativeAnnotationData::default();
  let items = annotations
    .iter()
    .map(|annotation| data.pack(annotation))
    .collect();
  NativeAnnotations { items, data }
}

/// A length as the native side counts it. Saturating keeps a list too long to
/// count short of its end rather than past it.
fn ffi_len(len: usize) -> u32 {
  u32::try_from(len).unwrap_or(u32::MAX)
}

/// [`NativeAnnotationData`] as the Metal side reads it: pointers into the
/// owner's buffers, valid while the owner is neither changed nor dropped.
/// The twin of `ScreenwideAnnotationData`. The D3D11 backend reads the Rust
/// lists directly and has no use for one.
#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NativeAnnotationDataView {
  pub(crate) points: *const [f32; 2],
  pub(crate) text: *const u8,
  pub(crate) point_count: u32,
  pub(crate) text_len: u32,
}

#[cfg(target_os = "macos")]
const _: () = assert!(std::mem::size_of::<NativeAnnotationDataView>() == 24);

/// [`NativeAnnotations`] as the native side reads it, under the same
/// lifetime rule as [`NativeAnnotationDataView`]. The twin of
/// `ScreenwideAnnotations`.
#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NativeAnnotationsView {
  pub(crate) items: *const NativeAnnotation,
  pub(crate) count: u32,
  pub(crate) data: NativeAnnotationDataView,
}

#[cfg(target_os = "macos")]
const _: () = assert!(std::mem::size_of::<NativeAnnotationsView>() == 40);
#[cfg(target_os = "macos")]
const _: () = assert!(std::mem::offset_of!(NativeAnnotationsView, data) == 16);

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
    let native = native_annotations(&annotations);
    assert_eq!(native.items.len(), 5_000);
    let last = native.items[4_999];
    assert_eq!(last.p0[0], 4_999.0);
    let start = last.data_offset as usize;
    assert_eq!(
      &native.data.text[start..start + last.data_count as usize],
      b"4999"
    );
  }
}
