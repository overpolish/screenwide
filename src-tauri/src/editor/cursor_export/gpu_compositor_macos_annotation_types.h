// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stddef.h>
#include <stdint.h>
#include "../annotations/reveal.h"

/// How many annotations one layer can carry. The list travels inline through
/// the retained workspace scene, so it is a fixed array rather than a pointer.
#define SCREENWIDE_MAX_ANNOTATIONS 32

/// The shapes the compositor draws, matching Rust's `NativeAnnotation`.
#define SCREENWIDE_ANNOTATION_ARROW 0u
#define SCREENWIDE_ANNOTATION_COUNTER 1u

/// One retained annotation, matching Rust's `NativeAnnotation`. Points stay in
/// source pixels; the binding step prepares draw geometry for the current
/// canvas placement. Every member is four bytes wide, so the struct packs the
/// same way everywhere.
///
/// An arrow fills `p0`, `p1` and `p2` with its Bézier's start, control and
/// end, and `width` with its stroke. A counter puts its centre in `p0`, the
/// direction of its tail in `p1[0]` - radians clockwise from east - its
/// number in `p1[1]`, and its disc's diameter in `width`; `p2` repeats the
/// centre.
typedef struct {
  /// `SCREENWIDE_ANNOTATION_ARROW` or `SCREENWIDE_ANNOTATION_COUNTER`.
  uint32_t kind;
  /// 0 none, 1 the end, 2 both ends.
  uint32_t head;
  uint32_t above_camera;
  /// Stroke width in output pixels.
  float width;
  /// Straight (non-premultiplied) RGBA.
  float color[4];
  /// The quadratic Bézier's start, control and end.
  float p0[2];
  float p1[2];
  float p2[2];
  /// The hover halo's width in canvas pixels, or zero for no halo. Preview
  /// decoration only: the export path always sends this as zero.
  float hover;
  /// Whether this annotation's clip draws itself in and out. Video export
  /// resolves `reveal` from it per frame; every other path arrives with it
  /// resolved.
  uint32_t animated;
  /// This frame's window and the size the annotation is drawn at.
  AnnotationReveal reveal;
} ScreenwideAnnotation;
_Static_assert(sizeof(ScreenwideAnnotation) == 96,
               "ScreenwideAnnotation ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, color) == 16,
               "ScreenwideAnnotation.color ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, p0) == 32,
               "ScreenwideAnnotation.p0 ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, hover) == 56,
               "ScreenwideAnnotation.hover ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, reveal) == 64,
               "ScreenwideAnnotation.reveal ABI must match Rust");

/// One layer's annotations. `count` may be zero; the array is still valid
/// memory, so the kernels never bind a nil buffer.
typedef struct {
  ScreenwideAnnotation items[SCREENWIDE_MAX_ANNOTATIONS];
  uint32_t count;
} ScreenwideAnnotations;

/// Source-time clip bounds used by video export. A cut never restarts a clip.
typedef struct {
  ScreenwideAnnotation annotation;
  uint64_t start_ms, end_ms;
} ScreenwideTimedAnnotation;
_Static_assert(sizeof(ScreenwideTimedAnnotation) == 112, "Timed annotation ABI");
