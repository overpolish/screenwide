// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>
#include "reveal.h"

/// A kind's draw geometry and the distance that picks it. The maths is in
/// Rust - `src-tauri/src/editor/annotations/ffi.rs` and each kind's own
/// `geometry` module - so the Metal compositor, the macOS chrome and the
/// D3D11 backend all draw and pick one geometry. The shaders carry the only
/// other copies, because a per-pixel SDF belongs in the shader language.

/// Prepared in the caller's pixel space. Plain float pairs keep the same
/// four-byte alignment in C and Metal's packed_float2.
typedef struct { float x, y; } AnnotationVector;
typedef struct { AnnotationVector a, b, c; } AnnotationTriangle;
typedef struct {
  AnnotationVector a, b, c;
  float width, low, high;
  AnnotationTriangle start_head, end_head;
  float rounding;
  uint32_t head;
} AnnotationArrowGeometry;
_Static_assert(sizeof(AnnotationArrowGeometry) == 92, "Prepared arrow ABI");

static inline AnnotationVector annotation_vector(float x, float y) {
  return (AnnotationVector){x, y};
}

/// One annotation's draw geometry, in the pixel space its points are given
/// in. An arrow solves its curve from the three points; a counter places its
/// disc at `p0` and aims its tail with `p1x`, which is the angle the retained
/// record keeps there rather than a point, so no placement touches it. A
/// counter's prepared record reads back as its disc's centre in `a`, the
/// tail's tip in `b`, the radius in `rounding`, the radius the tip is rounded
/// to in `low` and the diameter in `width`; where its number was rasterised
/// rides in `start_head`. A number no kind owns prepares nothing.
void screenwide_annotation_prepare(uint32_t kind, float p0x, float p0y,
                                   float p1x, float p1y, float p2x, float p2y,
                                   float width, uint32_t head,
                                   AnnotationReveal reveal,
                                   AnnotationArrowGeometry *out);

/// How far a point falls from a prepared annotation's drawn shape, in the
/// space it was prepared in. Zero anywhere the annotation is painted, which
/// is what picks it and what the halo is measured from.
float screenwide_annotation_distance(uint32_t kind, float px, float py,
                                     const AnnotationArrowGeometry *geometry);

/// How far a point falls outside a prepared shape's stroke or the box it
/// outlines: what picks a shape from anywhere inside it.
float screenwide_shape_body_distance(float px, float py,
                                     const AnnotationArrowGeometry *geometry);

/// How far a point falls outside a stroke's drawn line, from its fitted chain
/// of `count` points, each an `x` and a `y` in the caller's pixels. With
/// `body`, the box from `low` to `high` holding the stroke picks it too.
float screenwide_freehand_distance(float px, float py, const float *chain, uint32_t count,
                                   float width, uint32_t body, float low_x, float low_y,
                                   float high_x, float high_y);

/// The standard deviation of the spotlights' blur over a `width` by `height`
/// source as it arrives with `strength`, widening from nothing.
float screenwide_spotlight_blur_deviation(float strength, uint32_t width, uint32_t height);

/// How far the annotation's picture moves between the shutter opening and
/// now, which is the length its exposure is sampled along. `sx` and `sy`
/// carry a point from the space the points are given in into the pixels the
/// travel is measured in.
float screenwide_annotation_travel(uint32_t kind, float p0x, float p0y,
                                   float p1x, float p1y, float p2x, float p2y,
                                   float sx, float sy, float width,
                                   AnnotationReveal reveal);

/// How far a highlight's fastest line end moves between the shutter opening
/// and now. `p0` and `p1` are its record's source origin and pixel `(1, 1)`,
/// `sx` and `sy` carry them into the pixels the travel is measured in, and
/// `sweep` is its record's `params[2]`.
float screenwide_highlight_travel(float p0x, float p0y, float p1x, float p1y,
                                  float sx, float sy, float sweep,
                                  AnnotationReveal reveal);

/// How far a point falls outside a highlight, from what its grips' record
/// carries placed in display points: its first band's top-left corner and
/// bottom, its last band's top and bottom-right corner, and how far left and
/// right its block of bands reaches. Negative inside.
float screenwide_highlight_distance(float px, float py, float start_x, float start_y,
                                    float first_bottom, float last_top, float end_x,
                                    float end_y, float block_left, float block_right);
