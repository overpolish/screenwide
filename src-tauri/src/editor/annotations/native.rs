// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Source-space marks retained by the native compositor.
//!
//! The retained native workspace keeps a copy of every layer it presents and
//! redraws it without asking Rust again, so the marks travel inline rather
//! than as a borrowed pointer: a fixed array is the only shape that survives
//! a pan or a zoom. The same cap applies to the export so what the editor
//! previews is what the PNG gets.

use super::reveal::AnnotationReveal;
use super::{Annotation, MAX_ANNOTATIONS};
use crate::editor::annotations::{annotation_colour, AnnotationHead, AnnotationShape};

const KIND_ARROW: u32 = 0;

/// One retained mark matching C's `ScreenwideAnnotation`. The native binding
/// prepares separate draw geometry; every stored member is four bytes wide.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) struct NativeAnnotation {
  pub(crate) kind: u32,
  pub(crate) head: u32,
  pub(crate) above_camera: u32,
  pub(crate) width: f32,
  pub(crate) color: [f32; 4],
  pub(crate) p0: [f32; 2],
  pub(crate) p1: [f32; 2],
  pub(crate) p2: [f32; 2],
  /// The hover halo's width in canvas pixels, or zero for no halo. The halo
  /// is preview chrome: [`native_annotations`] never sets it, so nothing the
  /// export composes can carry one.
  pub(crate) hover: f32,
  /// Whether this mark's clip draws itself in and out. Video export reads it
  /// per frame; every other path has already resolved `reveal` from it.
  pub(crate) animated: u32,
  /// This frame's window and the size the mark is drawn at. The whole path at
  /// full size - a still, or a mark that does not animate - prepares the way
  /// it always has.
  pub(crate) reveal: AnnotationReveal,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotation>() == 96);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, color) == 16);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, p0) == 32);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, p2) == 48);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, hover) == 56);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, animated) == 60);
const _: () = assert!(std::mem::offset_of!(NativeAnnotation, reveal) == 64);

/// One layer's marks, bound as a single buffer. `count` may be zero; the
/// array is still valid memory so Metal never sees a nil buffer.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct NativeAnnotations {
  pub(crate) items: [NativeAnnotation; MAX_ANNOTATIONS],
  pub(crate) count: u32,
}

const _: () = assert!(std::mem::size_of::<NativeAnnotations>() == 96 * MAX_ANNOTATIONS + 4);

impl Default for NativeAnnotations {
  fn default() -> Self {
    Self {
      items: [NativeAnnotation::default(); MAX_ANNOTATIONS],
      count: 0,
    }
  }
}

/// Flatten a workspace's marks into the retained native scene.
pub(crate) fn native_annotations(annotations: &[Annotation]) -> NativeAnnotations {
  let mut native = NativeAnnotations::default();
  let annotations = annotations.iter();
  for (index, annotation) in annotations.take(MAX_ANNOTATIONS).enumerate() {
    let AnnotationShape::Arrow {
      start,
      control,
      end,
    } = &annotation.shape;
    native.items[index] = NativeAnnotation {
      kind: KIND_ARROW,
      head: match annotation.style.head {
        AnnotationHead::None => 0,
        AnnotationHead::End => 1,
        AnnotationHead::Both => 2,
      },
      above_camera: u32::from(annotation.above_camera),
      width: annotation.style.width.max(0.0) as f32,
      color: annotation_colour(&annotation.style.color),
      p0: [start.x as f32, start.y as f32],
      p1: [control.x as f32, control.y as f32],
      p2: [end.x as f32, end.y as f32],
      hover: 0.0,
      animated: u32::from(annotation.animated),
      reveal: annotation.reveal,
    };
    native.count = index as u32 + 1;
  }
  native
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationStyle};

  #[test]
  fn flattens_an_arrow() {
    let mut settings = crate::screenshots::test_output_settings(640, 360);
    settings.annotations = vec![Annotation {
      above_camera: true,
      animated: true,
      id: "a".to_owned(),
      reveal: Default::default(),
      shape: AnnotationShape::Arrow {
        start: AnnotationPoint { x: 1.0, y: 2.0 },
        control: AnnotationPoint { x: 3.0, y: 4.0 },
        end: AnnotationPoint { x: 5.0, y: 6.0 },
      },
      style: AnnotationStyle {
        color: "#0000ff".to_owned(),
        head: AnnotationHead::Both,
        width: 9.0,
      },
    }];
    let native = native_annotations(&settings.annotations);
    assert_eq!(native.count, 1);
    assert_eq!(native.items[0].head, 2);
    assert_eq!(native.items[0].above_camera, 1);
    assert_eq!(native.items[0].color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(native.items[0].p2, [5.0, 6.0]);
    assert_eq!(native.items[0].animated, 1);
    assert!(native.items[0].reveal.is_whole());
  }
}
