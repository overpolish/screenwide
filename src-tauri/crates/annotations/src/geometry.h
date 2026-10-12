// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>
#include "reveal.h"

/// A kind's draw geometry and the distance that picks it. The maths is in
/// Rust - `src-tauri/src/editor/annotations/ffi.rs` and each kind's own
/// `geometry` module - so the macOS chrome and the compositor draw and pick
/// one geometry. The shaders carry the only other copies, because a
/// per-pixel SDF belongs in the shader language.

/// Prepared in the caller's pixel space, as plain float pairs.
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

/// A magnifier's draw geometry: its zoom area from `p0` to `p2` and its loupe
/// centred on `p1`, `size` along its longer side, rounded by `radius` percent
/// of the shorter side, its rim `width` wide. The record reads back as
/// `magnify/geometry.rs` describes.
void screenwide_magnify_prepare(float p0x, float p0y, float p1x, float p1y,
                                float p2x, float p2y, float size, float radius,
                                float width, AnnotationReveal reveal,
                                AnnotationArrowGeometry *out);

/// Which part of a prepared magnifier a point is on: 1 for the loupe, 2 for
/// the zoom area, 0 for neither.
uint32_t screenwide_magnify_part(float px, float py,
                                 const AnnotationArrowGeometry *geometry);

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

/// How far a point falls outside a highlight, from what its grips' record
/// carries placed in display points: its first band's top-left corner and
/// bottom, its last band's top and bottom-right corner, and how far left and
/// right its block of bands reaches. Negative inside.
float screenwide_highlight_distance(float px, float py, float start_x, float start_y,
                                    float first_bottom, float last_top, float end_x,
                                    float end_y, float block_left, float block_right);

/// An image's own selection frame, in the display points its middle and half
/// extents are given in, as `image/frame.rs` places it: its four corners
/// clockwise from the picture's own top-left, then its nine grips - the
/// frame's eight, clockwise from that corner, and its radius dot - with the
/// handle each reports.
typedef struct {
  AnnotationVector corners[4];
  AnnotationVector grips[9];
  uint32_t handles[9];
} AnnotationImageFrame;
_Static_assert(sizeof(AnnotationImageFrame) == 140, "Image frame ABI");

/// The frame of an image whose middle is at `center`, turned `angle` radians
/// clockwise and rounded `radius` percent of its shorter side.
void screenwide_image_frame(float center_x, float center_y, float half_width,
                            float half_height, float angle, float radius,
                            AnnotationImageFrame *out);

/// The upright sides whose resize cursor runs the way a grip moving `sides`
/// of an image turned `angle` does on screen.
uint32_t screenwide_image_cursor_sides(uint32_t sides, float angle);
