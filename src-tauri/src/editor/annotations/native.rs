// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-space annotations handed to the native compositors.
//!
//! The lists are owned here and grow with the document; the native side reads
//! them through a borrowed view of pointers and lengths, so no list size is
//! baked into the boundary.

use super::redact::native::RedactSource;
use super::reveal::AnnotationReveal;
use super::Annotation;
use crate::editor::annotations::annotation_colour;
#[cfg(target_os = "windows")]
use crate::editor::annotations::AnnotationKind;

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
/// kind - a pixelated redaction's zones - indexes `points`, a text kind
/// indexes `text`, through its record's `data_offset` and `data_count`. Kept
/// apart from the items so a list that is longer than one scene - the video
/// export's clips - can share one set of buffers and pass it beside the list.
#[derive(Clone, Default)]
pub(crate) struct NativeAnnotationData {
  pub(crate) points: Vec<[f32; 2]>,
  pub(crate) text: Vec<u8>,
}

impl NativeAnnotationData {
  /// Write `annotation`'s variable-length data into the buffers and return
  /// its record, with the offsets pointing here. `source` is where a
  /// redaction reads what is under and around its box.
  pub(crate) fn pack(
    &mut self,
    annotation: &Annotation,
    source: RedactSource<'_>,
  ) -> NativeAnnotation {
    let [mut p0, mut p1, mut p2] = annotation.shape.draw_points(&annotation.style);
    let fill =
      annotation
        .shape
        .redaction_fill(&annotation.style, source, annotation.held.as_deref());
    if fill.is_some() && matches!(source, RedactSource::Video { .. }) {
      // Outward to even pixels, as the box's colour is covered in whole
      // two-pixel samples.
      let even = |value: f32, far: bool| {
        let halved = value / 2.0;
        2.0 * if far { halved.ceil() } else { halved.floor() }
      };
      p0 = p0.map(|value| even(value, false));
      p2 = p2.map(|value| even(value, true));
      p1 = p2;
    }
    self.points.extend_from_slice(&[p0, p1, p2]);
    let text = annotation.shape.draw_text();
    let data_offset = self.text.len();
    self.text.extend_from_slice(text.as_bytes());
    let mut record = NativeAnnotation {
      kind: annotation.shape.kind().raw(),
      head: annotation.shape.draw_head(&annotation.style),
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
    };
    if let Some(fill) = fill {
      record.color = fill.color;
      record.flags = fill.flags;
      record.params = fill.params;
      record.width = fill.radius;
      record.p3 = fill.grid.map(|count| count as f32);
      record.data_offset = ffi_len(self.points.len());
      record.data_count = ffi_len(fill.entries.len());
      self.points.extend_from_slice(&fill.entries);
      // A recording's export resolves the surface frame by frame from the
      // timeline worked out ahead; the preview hands over each frame's own.
      if let Some(surfaces) = annotation
        .held
        .as_deref()
        .map(|held| &held.surfaces)
        .filter(|surfaces| !surfaces.is_empty())
      {
        record.flags |= super::flags::SURFACES;
        record.p1 = [self.points.len() as f32, surfaces.len() as f32];
        self.points.extend_from_slice(surfaces);
      }
    }
    record
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

/// One composition's records. `source` is where its redactions read what is
/// under and around each box.
pub(crate) fn native_annotations(
  annotations: &[Annotation],
  source: RedactSource<'_>,
) -> NativeAnnotations {
  let mut data = NativeAnnotationData::default();
  let items = annotations
    .iter()
    .map(|annotation| data.pack(annotation, source))
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
        color: "#0000ff".to_owned(),
        head: AnnotationHead::Both,
        radius: 0.0,
        redaction: Default::default(),
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
    let native = native_annotations(&annotations, RedactSource::None);
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
    let native = native_annotations(&annotations, RedactSource::None);
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
