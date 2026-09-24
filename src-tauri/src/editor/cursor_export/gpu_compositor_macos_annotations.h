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

// Prepared records live in an MTLBuffer written in place: a list is as long
// as its document, far past what setBytes carries.

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
/// which no placement touches, so it is passed as it stands; a text box reads
/// its pointer, held against the box, out of `p1` and its text block's size
/// out of `p2` the same way.
static inline AnnotationArrowGeometry screenwide_prepare_annotation(
    const ScreenwideAnnotation *annotation, AnnotationVector a, AnnotationVector b,
    AnnotationVector c, AnnotationReveal reveal) {
  float p1x = annotation->kind == SCREENWIDE_ANNOTATION_COUNTER ? annotation->p1[0] : b.x;
  if (annotation->kind == SCREENWIDE_ANNOTATION_TEXT) {
    p1x = annotation->p1[0];
    b = annotation_vector(annotation->p1[0], annotation->p1[1]);
    c = annotation_vector(annotation->p2[0], annotation->p2[1]);
  }
  AnnotationArrowGeometry prepared;
  screenwide_annotation_prepare(annotation->kind, a.x, a.y, p1x, b.y, c.x, c.y,
                                annotation->width, annotation->head, reveal, &prepared);
  return prepared;
}

/// Prepare complete shapes along the exposure, keeping curve solves off the
/// GPU, and rasterise the counters' numbers and the text boxes' text at the
/// size they are drawn. Every per-annotation array is sized by the list it is
/// handed and kept off the stack, which a long document would overrun.
static inline void screenwide_bind_annotations(
    id<MTLComputeCommandEncoder> encoder, const ScreenwideAnnotations *annotations,
    const ScreenwideCanvas *canvas, uint32_t source_width, uint32_t source_height) {
  uint32_t count = annotations == NULL ? 0 : annotations->count;
  id<MTLDevice> device = encoder.device;
  // Metal will not make an empty buffer, so an empty list binds one zeroed record.
  NSUInteger slots = MAX(count, 1u);
  id<MTLBuffer> prepared_buffer = [device
      newBufferWithLength:slots * sizeof(ScreenwidePreparedAnnotation)
                  options:MTLResourceStorageModeShared];
  ScreenwidePreparedAnnotation *prepared = prepared_buffer.contents;
  memset(prepared, 0, slots * sizeof(ScreenwidePreparedAnnotation));
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
  id<MTLBuffer> buffer = total > 0 ? [device
      newBufferWithLength:total * sizeof(ScreenwideAnnotationSample)
      options:MTLResourceStorageModeShared] : nil;
  ScreenwideAnnotationSample *samples = buffer.contents;
  NSMutableData *radii_data = [NSMutableData dataWithLength:slots * sizeof(float)];
  float *radii = radii_data.mutableBytes;
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
    // A counter's number is set at its disc's radius and a text box's text at
    // its own type size, both as this frame draws them.
    if (annotation->kind == SCREENWIDE_ANNOTATION_COUNTER) radii[index] = draw->arrow.rounding;
    if (annotation->kind == SCREENWIDE_ANNOTATION_TEXT) radii[index] = draw->arrow.width;
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
  NSMutableData *text_values_data = [NSMutableData dataWithLength:slots * sizeof(const char *)];
  NSMutableData *text_lengths_data = [NSMutableData dataWithLength:slots * sizeof(uint32_t)];
  NSMutableData *text_sizes_data = [NSMutableData dataWithLength:slots * sizeof(float)];
  NSMutableData *text_styles_data = [NSMutableData dataWithLength:slots * sizeof(uint32_t)];
  NSMutableData *text_rects_data =
      [NSMutableData dataWithLength:slots * sizeof(ScreenwideAnnotationTextRect)];
  const char **text_values = text_values_data.mutableBytes;
  uint32_t *text_lengths = text_lengths_data.mutableBytes;
  float *text_sizes = text_sizes_data.mutableBytes;
  uint32_t *text_styles = text_styles_data.mutableBytes;
  ScreenwideAnnotationTextRect *text = text_rects_data.mutableBytes;
  for (uint32_t index = 0; index < count; index++) {
    const ScreenwideAnnotation *annotation = &annotations->items[index];
    BOOL typed = annotation->kind == SCREENWIDE_ANNOTATION_COUNTER ||
                 annotation->kind == SCREENWIDE_ANNOTATION_TEXT;
    if (!typed || annotation->data_count == 0 ||
        annotation->data_offset + annotation->data_count > annotations->data.text_len)
      continue;
    text_values[index] = (const char *)(annotations->data.text + annotation->data_offset);
    text_lengths[index] = annotation->data_count;
    text_sizes[index] = radii[index];
    text_styles[index] = annotation->kind == SCREENWIDE_ANNOTATION_TEXT
        ? 1u + (annotation->head & 3u)
        : SCREENWIDE_ANNOTATION_TEXT_STYLE_COUNTER;
  }
  ScreenwideAnnotationTextUniforms text_uniforms = {0};
  id<MTLBuffer> numbers = screenwide_annotation_text_atlas(
      device, text_values, text_lengths, text_sizes, text_styles, count, text,
      &text_uniforms);
  for (uint32_t index = 0; index < count; index++) {
    // Only a counter and a text box read these slots as a text rectangle; an
    // arrow with a head at both ends keeps its second head's triangle in them.
    uint32_t kind = annotations->items[index].kind;
    if (kind != SCREENWIDE_ANNOTATION_COUNTER && kind != SCREENWIDE_ANNOTATION_TEXT) continue;
    prepared[index].arrow.start_head.a = annotation_vector(text[index].x, text[index].y);
    prepared[index].arrow.start_head.b =
        annotation_vector(text[index].width, text[index].height);
  }
  // Only the used length is uploaded; an empty side buffer binds a zeroed word.
  const uint8_t empty_side[8] = {0};
  NSUInteger points_length =
      annotations == NULL ? 0 : (NSUInteger)annotations->data.point_count * sizeof(float[2]);
  if (points_length > 0)
    [encoder setBuffer:[device newBufferWithBytes:annotations->data.points
                                           length:points_length
                                          options:MTLResourceStorageModeShared]
                offset:0
               atIndex:18];
  else
    [encoder setBytes:empty_side length:sizeof(empty_side) atIndex:18];
  NSUInteger text_length = annotations == NULL ? 0 : annotations->data.text_len;
  if (text_length > 0)
    [encoder setBuffer:[device newBufferWithBytes:annotations->data.text
                                           length:text_length
                                          options:MTLResourceStorageModeShared]
                offset:0
               atIndex:19];
  else
    [encoder setBytes:empty_side length:sizeof(empty_side) atIndex:19];
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
