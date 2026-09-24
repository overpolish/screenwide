// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <CoreGraphics/CoreGraphics.h>
#import <Metal/Metal.h>
#include <stdint.h>

/// The annotations' type, rasterised: counters' numbers and text boxes' text.
///
/// Type is drawn by the text engine rather than approximated by the shader,
/// at the size it is actually drawn, into an atlas the annotation kernels
/// sample the way they sample the keyboard's artwork. The atlas persists per
/// thread and keeps what recent frames drew, so a frame rasterises only the
/// type it has not drawn before; where each goes is laid out in Rust, shared
/// with the D3D11 backend.
///
/// The atlas is rasterised at a density that follows how large the canvas is
/// drawn - `screenwide_annotation_text_scale`, shared with the D3D11 backend -
/// and each drawn pixel is read as four filtered taps over the atlas pixels it
/// spans, so type stays smooth on a Retina display, zoomed in or out, and
/// while it grows into place.

/// Where one annotation's type sits in the atlas, in atlas pixels. An
/// annotation with no type gets a zero rectangle, which the kernels skip.
typedef struct {
  float x, y, width, height;
} ScreenwideAnnotationTextRect;

/// The atlas's own size, which is what turns a rectangle into a uv, and how
/// many atlas pixels it holds per canvas pixel. The twin of the kernels'
/// `AnnotationTextAtlas`.
typedef struct {
  uint32_t width, height;
  float scale;
} ScreenwideAnnotationTextUniforms;

/// Atlas pixels per canvas pixel for a composition that draws one canvas
/// pixel over `1 / pixel_scale` drawn pixels, its largest type `largest`
/// canvas pixels in size. Decided in `annotations/counter/atlas_scale.rs`.
float screenwide_annotation_text_scale(float pixel_scale, float largest);

/// How much of the disc's diameter a single digit's cap height takes. Two
/// digits are narrowed to fit rather than being allowed to touch the edge.
static const float SCREENWIDE_COUNTER_TEXT_CAP_SHARE = 0.42f;
/// The widest the number may be drawn, as a share of the diameter.
static const float SCREENWIDE_COUNTER_TEXT_WIDTH_SHARE = 0.72f;
/// Inter's cap height, in ems: what turns a wanted cap height into a size.
static const float SCREENWIDE_COUNTER_CAP_HEIGHT = 0.727f;

/// What one entry's type is set as: a counter's number, or a text box's text
/// at one more than its alignment.
#define SCREENWIDE_ANNOTATION_TEXT_STYLE_COUNTER 0u

/// Lays out and rasterises `count` UTF-8 strings, writing each one's
/// rectangle into `rects`. `sizes` is a counter's disc radius or a text box's
/// type size, in atlas pixels, and `styles` says which each entry is. A NULL
/// value or a size under a pixel draws nothing.
id<MTLBuffer> screenwide_annotation_text_atlas(
    id<MTLDevice> device, const char *const *values, const uint32_t *lengths,
    const float *sizes, const uint32_t *styles, uint32_t count,
    ScreenwideAnnotationTextRect *rects, ScreenwideAnnotationTextUniforms *uniforms);

/// A cleared bitmap context the size of `rect`, set up to draw type the
/// kernels tint, and the copy of it into the atlas's pixels, which releases
/// the context.
CGContextRef screenwide_annotation_cell_context(ScreenwideAnnotationTextRect rect);
void screenwide_annotation_cell_copy(CGContextRef context, uint8_t *pixels,
                                     uint32_t atlas_width,
                                     ScreenwideAnnotationTextRect rect);

/// Registers the bundled Inter face with Core Text, once per process. The
/// keyboard artwork and the annotations' type both draw in it.
void screenwide_register_inter_font(void);
