// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
#import "gpu_compositor_macos.h"
#import "gpu_compositor_macos_annotation_types.h"
#import "../annotations/geometry.h"
#import "gpu_compositor_macos_annotation_text.h"

/// A draw-ready annotation, distinct from the source-space document retained by
/// the presenter. Preparing at binding also covers native placement changes.
typedef struct {
  uint32_t kind, above_camera, flags;
  float params[3];
  float color[4];
  float hover;
  AnnotationArrowGeometry arrow;
  uint32_t sample_offset, sample_count;
  uint32_t data_offset, data_count;
} ScreenwidePreparedAnnotation;
_Static_assert(sizeof(ScreenwidePreparedAnnotation) == 152, "Prepared annotation ABI");

// Prepared records are backed by an MTLBuffer because the widened array no longer fits setBytes.

typedef struct {
  AnnotationArrowGeometry arrow;
  float opacity;
} ScreenwideAnnotationSample;
_Static_assert(sizeof(ScreenwideAnnotationSample) == 96, "Exposure sample ABI");

/// How many exposure samples one annotation needs: enough that consecutive
/// samples are under a pixel apart, and none at all for an annotation that has
/// not moved. The travel itself is measured in Rust, where the D3D11 backend
/// measures its own.
static inline uint32_t screenwide_annotation_sample_count(
    const ScreenwideAnnotation *annotation, float sx, float sy) {
  AnnotationReveal r = annotation->reveal;
  float travel = screenwide_annotation_travel(
      annotation->kind, annotation->p0[0], annotation->p0[1], annotation->p1[0],
      annotation->p1[1], annotation->p2[0], annotation->p2[1], sx, sy,
      annotation->width, r);
  if (travel < 1.5f && fabsf(r.opacity - r.previous[3]) < 0.01f) return 0;
  return (uint32_t)fminf(fmaxf(ceilf(travel / 0.75f) + 1, 8), 48);
}

/// One annotation's draw geometry for the canvas placement in `canvas`. An
/// arrow solves its curve; a counter places its disc and tail from the same
/// centre-and-angle the document holds - the aim rides in `p1[0]` as an angle,
/// which no placement touches, so it is passed as it stands.
static inline AnnotationArrowGeometry screenwide_prepare_annotation(
    const ScreenwideAnnotation *annotation, AnnotationVector a, AnnotationVector b,
    AnnotationVector c, AnnotationReveal reveal) {
  float p1x = annotation->kind == SCREENWIDE_ANNOTATION_COUNTER ? annotation->p1[0] : b.x;
  AnnotationArrowGeometry prepared;
  screenwide_annotation_prepare(annotation->kind, a.x, a.y, p1x, b.y, c.x, c.y,
                                annotation->width, annotation->head, reveal, &prepared);
  return prepared;
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
  // Exposure geometry exceeds Metal's 4 KiB inline limit. The encoder retains
  // this buffer.
  id<MTLBuffer> buffer = total > 0 ? [encoder.device
      newBufferWithLength:total * sizeof(ScreenwideAnnotationSample)
      options:MTLResourceStorageModeShared] : nil;
  ScreenwideAnnotationSample *samples = buffer.contents;
  float radii[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  for (uint32_t index = 0; index < count; index++) {
    const ScreenwideAnnotation *annotation = &annotations->items[index];
    ScreenwidePreparedAnnotation *draw = &prepared[index];
    draw->flags = annotation->flags;
    memcpy(draw->params, annotation->params, sizeof(draw->params));
    draw->data_offset = annotation->data_offset;
    draw->data_count = annotation->data_count;
    draw->kind = annotation->kind;
    draw->above_camera = annotation->above_camera;
    memcpy(draw->color, annotation->color, sizeof(draw->color));
    draw->hover = annotation->hover;
    AnnotationVector a = annotation_vector(canvas->image_x + annotation->p0[0] * scale_x, canvas->image_y + annotation->p0[1] * scale_y);
    AnnotationVector b = annotation_vector(canvas->image_x + annotation->p1[0] * scale_x, canvas->image_y + annotation->p1[1] * scale_y);
    AnnotationVector c = annotation_vector(canvas->image_x + annotation->p2[0] * scale_x, canvas->image_y + annotation->p2[1] * scale_y);
    draw->arrow = screenwide_prepare_annotation(annotation, a, b, c, annotation->reveal);
    if (annotation->kind == SCREENWIDE_ANNOTATION_COUNTER) radii[index] = draw->arrow.rounding;
    if (draw->sample_count == 0) {
      draw->color[3] *= fmaxf(fminf(annotation->reveal.opacity, 1), 0);
      continue;
    }
    for (uint32_t tap = 0; tap < draw->sample_count; ++tap) {
      float t = ((float)tap + 0.5f) / (float)draw->sample_count;
      AnnotationReveal r = annotation->reveal;
      r.low = r.previous[0] + (r.low - r.previous[0]) * t;
      r.high = r.previous[1] + (r.high - r.previous[1]) * t;
      r.scale = r.previous[2] + (r.scale - r.previous[2]) * t;
      r.opacity = r.previous[3] + (r.opacity - r.previous[3]) * t;
      ScreenwideAnnotationSample *sample = &samples[draw->sample_offset + tap];
      sample->arrow = screenwide_prepare_annotation(annotation, a, b, c, r);
      sample->opacity = fmaxf(fminf(r.opacity, 1), 0);
    }
  }
  const char *text_values[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  uint32_t text_lengths[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  float text_sizes[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  for (uint32_t index = 0; index < count; index++) {
    const ScreenwideAnnotation *annotation = &annotations->items[index];
    if (annotation->kind != SCREENWIDE_ANNOTATION_COUNTER || annotation->data_count == 0 ||
        annotation->data_offset + annotation->data_count > annotations->data.text_len)
      continue;
    text_values[index] = (const char *)(annotations->data.text + annotation->data_offset);
    text_lengths[index] = annotation->data_count;
    text_sizes[index] = radii[index];
  }
  ScreenwideAnnotationTextRect text[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  ScreenwideAnnotationTextUniforms text_uniforms = {0};
  id<MTLBuffer> numbers = screenwide_annotation_text_atlas(
      encoder.device, text_values, text_lengths, text_sizes, count, text, &text_uniforms);
  for (uint32_t index = 0; index < count; index++) {
    // Only a counter reads these slots as a text rectangle; an arrow with a
    // head at both ends keeps its second head's triangle in them.
    if (annotations->items[index].kind != SCREENWIDE_ANNOTATION_COUNTER) continue;
    prepared[index].arrow.start_head.a = annotation_vector(text[index].x, text[index].y);
    prepared[index].arrow.start_head.b =
        annotation_vector(text[index].width, text[index].height);
  }
  id<MTLBuffer> points_buffer = [encoder.device
      newBufferWithBytes:annotations->data.points
                   length:sizeof(annotations->data.points)
                  options:MTLResourceStorageModeShared];
  id<MTLBuffer> text_buffer = [encoder.device
      newBufferWithBytes:annotations->data.text
                   length:sizeof(annotations->data.text)
                  options:MTLResourceStorageModeShared];
  [encoder setBuffer:points_buffer offset:0 atIndex:18];
  [encoder setBuffer:text_buffer offset:0 atIndex:19];
  id<MTLBuffer> prepared_buffer = [encoder.device
      newBufferWithBytes:prepared length:sizeof(prepared)
      options:MTLResourceStorageModeShared];
  [encoder setBuffer:prepared_buffer offset:0 atIndex:12];
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
