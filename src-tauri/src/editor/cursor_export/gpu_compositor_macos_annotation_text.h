// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>
#include <stdint.h>

/// The counters' numbers, rasterised.
///
/// A counter's number is type, so it is drawn by the text engine rather than
/// approximated by the shader: every counter in a dispatch is rasterised at
/// the size it is actually drawn and stacked into one buffer, which the
/// annotation kernels sample the way they sample the keyboard's artwork.
///
/// The atlas is rasterised at [`SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE`] times
/// the drawn size and sampled with four taps, so a counter still reads while
/// it is growing into place rather than crawling with aliasing over the two
/// hundred milliseconds of its arrival.

/// Where one counter's number sits in the atlas, in atlas pixels. An annotation
/// that is not a counter gets a zero rectangle, which the kernels skip.
typedef struct {
  float x, y, width, height;
} ScreenwideAnnotationTextRect;

/// The atlas's own size, which is what turns a rectangle into a uv.
typedef struct {
  uint32_t width, height;
} ScreenwideAnnotationTextUniforms;

/// How much of the disc's diameter a single digit's cap height takes. Two
/// digits are narrowed to fit rather than being allowed to touch the edge.
static const float SCREENWIDE_COUNTER_TEXT_CAP_SHARE = 0.42f;
/// The widest the number may be drawn, as a share of the diameter.
static const float SCREENWIDE_COUNTER_TEXT_WIDTH_SHARE = 0.72f;
/// Inter's cap height, in ems: what turns a wanted cap height into a size.
static const float SCREENWIDE_COUNTER_CAP_HEIGHT = 0.727f;
/// How many atlas pixels are rasterised per drawn pixel.
static const float SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE = 2.0f;

/// Rasterises `count` numbers - `values[i]` at `radii[i]` drawn pixels, where a
/// zero value is an annotation that is not a counter - into one buffer, and
/// fills `rects` and `uniforms` with where each landed. Returns nil when there
/// is nothing to draw, which the kernels read as no counters.
///
/// The result is cached against the numbers and sizes it was built from, so a
/// preview that redraws an unchanged list does no work at all.
id<MTLBuffer> screenwide_annotation_text_atlas(
    id<MTLDevice> device, const uint32_t *values, const float *radii,
    uint32_t count, ScreenwideAnnotationTextRect *rects,
    ScreenwideAnnotationTextUniforms *uniforms);

/// Registers the bundled Inter face with Core Text, once per process. The
/// keyboard artwork and the counters' numbers both draw in it.
void screenwide_register_inter_font(void);
