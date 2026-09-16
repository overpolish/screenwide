// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stddef.h>
#include <stdint.h>
#include "../annotations/reveal.h"

/// How many marks one layer can carry. The list travels inline through the
/// retained workspace scene, so it is a fixed array rather than a pointer.
#define SCREENWIDE_MAX_ANNOTATIONS 32

/// One retained mark, matching Rust's `NativeAnnotation`. Points stay in
/// source pixels; the binding step prepares draw geometry for the current
/// canvas placement. Every member is
/// four bytes wide, so the struct packs the same way everywhere.
typedef struct {
  /// Zero is an arrow. The remaining shapes arrive with their tools.
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
  /// Whether this mark's clip draws itself in and out. Video export resolves
  /// `reveal` from it per frame; every other path arrives with it resolved.
  uint32_t animated;
  /// This frame's window and the size the mark is drawn at.
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

/// One layer's marks. `count` may be zero; the array is still valid memory,
/// so the kernels never bind a nil buffer.
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
