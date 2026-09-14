// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stddef.h>
#include <stdint.h>

/// How many marks one layer can carry. The list travels inline through the
/// retained workspace scene, so it is a fixed array rather than a pointer.
#define SCREENWIDE_MAX_ANNOTATIONS 32

/// One drawn mark, in the layout of Rust's `NativeAnnotation` and Metal's
/// `AnnotationUniforms`. Points are in the source's own pixel space; the
/// shader maps them through the canvas' image placement. Every member is
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
} ScreenwideAnnotation;
_Static_assert(sizeof(ScreenwideAnnotation) == 60,
               "ScreenwideAnnotation ABI must match Rust and Metal");
_Static_assert(offsetof(ScreenwideAnnotation, color) == 16,
               "ScreenwideAnnotation.color ABI must match Rust and Metal");
_Static_assert(offsetof(ScreenwideAnnotation, p0) == 32,
               "ScreenwideAnnotation.p0 ABI must match Rust and Metal");
_Static_assert(offsetof(ScreenwideAnnotation, hover) == 56,
               "ScreenwideAnnotation.hover ABI must match Rust and Metal");

/// One layer's marks. `count` may be zero; the array is still valid memory,
/// so the kernels never bind a nil buffer.
typedef struct {
  ScreenwideAnnotation items[SCREENWIDE_MAX_ANNOTATIONS];
  uint32_t count;
} ScreenwideAnnotations;
