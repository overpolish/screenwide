// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stddef.h>
#include <stdint.h>
#include "../annotations/reveal.h"

#define SCREENWIDE_MAX_ANNOTATIONS 32
#define SCREENWIDE_MAX_ANNOTATION_POINTS 4096
#define SCREENWIDE_MAX_ANNOTATION_TEXT 4096
#define SCREENWIDE_ANNOTATION_ARROW 0u
#define SCREENWIDE_ANNOTATION_COUNTER 1u
#define SCREENWIDE_ANNOTATION_FLAG_FILL (1u << 0)
#define SCREENWIDE_ANNOTATION_FLAG_MULTIPLY (1u << 1)
#define SCREENWIDE_ANNOTATION_FLAG_PIXELATE (1u << 2)

typedef struct {
  uint32_t kind;
  uint32_t head;
  uint32_t above_camera;
  uint32_t flags;
  float width;
  float params[3];
  float color[4];
  float p0[2];
  float p1[2];
  float p2[2];
  float p3[2];
  uint32_t data_offset;
  uint32_t data_count;
  float hover;
  uint32_t animated;
  AnnotationReveal reveal;
} ScreenwideAnnotation;
_Static_assert(sizeof(ScreenwideAnnotation) == 128, "ScreenwideAnnotation ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, color) == 32, "ScreenwideAnnotation.color ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, p0) == 48, "ScreenwideAnnotation.p0 ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, hover) == 88, "ScreenwideAnnotation.hover ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotation, reveal) == 96, "ScreenwideAnnotation.reveal ABI must match Rust");

/// The side buffers a record's `data_offset`/`data_count` index: `points`
/// for a points kind, `text` for a text kind. Matches Rust's
/// `NativeAnnotationData`. Held apart from the items so the video export can
/// share one set across every clip and copy it into each frame's scene.
typedef struct {
  float points[SCREENWIDE_MAX_ANNOTATION_POINTS][2];
  uint8_t text[SCREENWIDE_MAX_ANNOTATION_TEXT];
  uint32_t point_count;
  uint32_t text_len;
} ScreenwideAnnotationData;
_Static_assert(sizeof(ScreenwideAnnotationData) == 32768 + 4096 + 8,
               "ScreenwideAnnotationData ABI must match Rust");

typedef struct {
  ScreenwideAnnotation items[SCREENWIDE_MAX_ANNOTATIONS];
  uint32_t count;
  ScreenwideAnnotationData data;
} ScreenwideAnnotations;

_Static_assert(sizeof(ScreenwideAnnotations) == 128 * SCREENWIDE_MAX_ANNOTATIONS + 4 + 32768 + 4096 + 8,
               "ScreenwideAnnotations ABI must match Rust");

typedef struct {
  ScreenwideAnnotation annotation;
  uint64_t start_ms, end_ms;
} ScreenwideTimedAnnotation;
_Static_assert(sizeof(ScreenwideTimedAnnotation) == 144, "Timed annotation ABI");
