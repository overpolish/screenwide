// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>
#include <math.h>
#include <string.h>
#import "gpu_compositor_macos.h"
#import "gpu_compositor_macos_annotation_types.h"
#import "../annotations/geometry.h"
#import "gpu_compositor_macos_annotation_text.h"

/// A draw-ready mark, distinct from the source-space document retained by the
/// presenter. Preparing at binding also covers native placement changes.
typedef struct {
  uint32_t kind, above_camera;
  float color[4];
  float hover;
  AnnotationArrowGeometry arrow;
  uint32_t sample_offset, sample_count;
} ScreenwidePreparedAnnotation;
_Static_assert(sizeof(ScreenwidePreparedAnnotation) == 128, "Prepared annotation ABI");
_Static_assert(sizeof(ScreenwidePreparedAnnotation) * SCREENWIDE_MAX_ANNOTATIONS <= 4096,
               "Metal inline annotation bytes must fit setBytes");

typedef struct {
  AnnotationArrowGeometry arrow;
  float opacity;
} ScreenwideAnnotationSample;
_Static_assert(sizeof(ScreenwideAnnotationSample) == 96, "Exposure sample ABI");

/// How many exposure samples one mark needs: enough that consecutive samples
/// are under a pixel apart, and none at all for a mark that has not moved.
///
/// An arrow's travel is its window sliding along its own path; a counter has
/// no path, so all it can cover in a frame is the change in its own size -
/// the disc's edge sweeping out as it grows.
static inline uint32_t screenwide_annotation_sample_count(
    const ScreenwideAnnotation *mark, float sx, float sy) {
  AnnotationReveal r = mark->reveal;
  float travel = 0;
  if (mark->kind == SCREENWIDE_ANNOTATION_COUNTER) {
    travel = mark->width * 0.5f * ANNOTATION_COUNTER_TAIL_REACH *
             fabsf(r.scale - r.previous[2]);
  } else {
    float length = hypotf((mark->p1[0] - mark->p0[0]) * sx,
                          (mark->p1[1] - mark->p0[1]) * sy) +
                   hypotf((mark->p2[0] - mark->p1[0]) * sx,
                          (mark->p2[1] - mark->p1[1]) * sy);
    travel = length * fmaxf(fabsf(r.low - r.previous[0]),
                            fabsf(r.high - r.previous[1]));
    travel += mark->width * 4.0f * fabsf(r.scale - r.previous[2]);
  }
  if (travel < 1.5f && fabsf(r.opacity - r.previous[3]) < 0.01f) return 0;
  return (uint32_t)fminf(fmaxf(ceilf(travel / 0.75f) + 1, 8), 48);
}

/// One mark's draw geometry for the canvas placement in `canvas`. An arrow
/// solves its curve; a counter places its disc and tail from the same
/// centre-and-angle the document holds.
static inline AnnotationArrowGeometry screenwide_prepare_annotation(
    const ScreenwideAnnotation *mark, AnnotationVector a, AnnotationVector b,
    AnnotationVector c, AnnotationReveal reveal) {
  if (mark->kind == SCREENWIDE_ANNOTATION_COUNTER)
    return annotation_prepare_counter(a, mark->width, mark->p1[0], reveal);
  return annotation_prepare_arrow(a, b, c, mark->width, mark->head, reveal);
}

/// Prepare complete shapes along the exposure, keeping curve solves off the
/// GPU, and rasterise the counters' numbers at the size they are drawn.
static inline void screenwide_bind_annotations(
    id<MTLComputeCommandEncoder> encoder, const ScreenwideAnnotations *annotations,
    const ScreenwideCanvas *canvas, uint32_t source_width, uint32_t source_height) {
  uint32_t count = annotations == NULL ? 0
      : MIN(annotations->count, (uint32_t)SCREENWIDE_MAX_ANNOTATIONS);
  ScreenwidePreparedAnnotation prepared[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  float scale_x = (float)canvas->image_width / MAX(source_width, 1u);
  float scale_y = (float)canvas->image_height / MAX(source_height, 1u);
  uint32_t total = 0;
  for (uint32_t i = 0; i < count; ++i) {
    prepared[i].sample_offset = total;
    prepared[i].sample_count = screenwide_annotation_sample_count(
        &annotations->items[i], scale_x, scale_y);
    total += prepared[i].sample_count;
  }
  // Exposure geometry exceeds Metal's 4 KiB inline limit. The encoder retains this buffer.
  id<MTLBuffer> buffer = total > 0 ? [encoder.device
      newBufferWithLength:total * sizeof(ScreenwideAnnotationSample)
      options:MTLResourceStorageModeShared] : nil;
  ScreenwideAnnotationSample *samples = buffer.contents;
  uint32_t values[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  float radii[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  for (uint32_t index = 0; index < count; index++) {
    const ScreenwideAnnotation *mark = &annotations->items[index];
    ScreenwidePreparedAnnotation *draw = &prepared[index];
    draw->kind = mark->kind;
    draw->above_camera = mark->above_camera;
    memcpy(draw->color, mark->color, sizeof(draw->color));
    draw->hover = mark->hover;
    AnnotationVector a = annotation_vector(canvas->image_x + mark->p0[0] * scale_x,
        canvas->image_y + mark->p0[1] * scale_y);
    AnnotationVector b = annotation_vector(canvas->image_x + mark->p1[0] * scale_x,
        canvas->image_y + mark->p1[1] * scale_y);
    AnnotationVector c = annotation_vector(canvas->image_x + mark->p2[0] * scale_x,
        canvas->image_y + mark->p2[1] * scale_y);
    draw->arrow = screenwide_prepare_annotation(mark, a, b, c, mark->reveal);
    if (mark->kind == SCREENWIDE_ANNOTATION_COUNTER) {
      values[index] = (uint32_t)fmaxf(mark->p1[1], 0);
      radii[index] = draw->arrow.rounding;
    }
    if (draw->sample_count == 0) {
      draw->color[3] *= fmaxf(fminf(mark->reveal.opacity, 1), 0);
      continue;
    }
    for (uint32_t tap = 0; tap < draw->sample_count; ++tap) {
      float t = ((float)tap + 0.5f) / (float)draw->sample_count;
      AnnotationReveal r = mark->reveal;
      r.low = r.previous[0] + (r.low - r.previous[0]) * t;
      r.high = r.previous[1] + (r.high - r.previous[1]) * t;
      r.scale = r.previous[2] + (r.scale - r.previous[2]) * t;
      r.opacity = r.previous[3] + (r.opacity - r.previous[3]) * t;
      ScreenwideAnnotationSample *sample = &samples[draw->sample_offset + tap];
      sample->arrow = screenwide_prepare_annotation(mark, a, b, c, r);
      sample->opacity = fmaxf(fminf(r.opacity, 1), 0);
    }
  }
  // The numbers are type, so they are rasterised rather than approximated.
  // Where each one landed rides in the slots an arrow fills with its heads.
  ScreenwideAnnotationTextRect text[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  ScreenwideAnnotationTextUniforms text_uniforms = {0};
  id<MTLBuffer> numbers = screenwide_annotation_text_atlas(
      encoder.device, values, radii, count, text, &text_uniforms);
  for (uint32_t index = 0; index < count; index++) {
    // Only a counter reads these slots as a text rectangle; an arrow with a
    // head at both ends keeps its second head's triangle in them.
    if (annotations->items[index].kind != SCREENWIDE_ANNOTATION_COUNTER) continue;
    prepared[index].arrow.start_head.a = annotation_vector(text[index].x, text[index].y);
    prepared[index].arrow.start_head.b =
        annotation_vector(text[index].width, text[index].height);
  }
  [encoder setBytes:prepared length:sizeof(prepared) atIndex:12];
  [encoder setBytes:&count length:sizeof(count) atIndex:13];
  if (buffer) {
    [encoder setBuffer:buffer offset:0 atIndex:15];
  } else {
    ScreenwideAnnotationSample empty = {0};
    [encoder setBytes:&empty length:sizeof(empty) atIndex:15];
  }
  if (numbers) {
    [encoder setBuffer:numbers offset:0 atIndex:16];
  } else {
    const uint8_t empty[4] = {0, 0, 0, 0};
    [encoder setBytes:empty length:sizeof(empty) atIndex:16];
  }
  [encoder setBytes:&text_uniforms length:sizeof(text_uniforms) atIndex:17];
}
