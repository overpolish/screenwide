// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stddef.h>
#include <stdint.h>
#include "../annotations/reveal.h"

#define SCREENWIDE_ANNOTATION_ARROW 0u
#define SCREENWIDE_ANNOTATION_COUNTER 1u
#define SCREENWIDE_ANNOTATION_TEXT 2u
#define SCREENWIDE_ANNOTATION_REDACT 3u
#define SCREENWIDE_ANNOTATION_FLAG_FILL (1u << 0)
#define SCREENWIDE_ANNOTATION_FLAG_MULTIPLY (1u << 1)
#define SCREENWIDE_ANNOTATION_FLAG_PIXELATE (1u << 2)
#define SCREENWIDE_ANNOTATION_FLAG_BLUR (1u << 3)
#define SCREENWIDE_ANNOTATION_FLAG_MOSAIC (1u << 4)
#define SCREENWIDE_ANNOTATION_FLAG_SURFACES (1u << 5)

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

/// The side buffers a list's records index through `data_offset` and
/// `data_count`. Borrowed: whoever hands this over owns the bytes, and a
/// retained scene copies them. The twin of Rust's `NativeAnnotationDataView`.
typedef struct {
  const float (*points)[2];
  const uint8_t *text;
  uint32_t point_count;
  uint32_t text_len;
} ScreenwideAnnotationData;
_Static_assert(sizeof(ScreenwideAnnotationData) == 24, "ScreenwideAnnotationData ABI must match Rust");

/// One composition's annotations, borrowed under the same rule as their
/// data. The twin of Rust's `NativeAnnotationsView`.
typedef struct {
  const ScreenwideAnnotation *items;
  uint32_t count;
  ScreenwideAnnotationData data;
} ScreenwideAnnotations;
_Static_assert(sizeof(ScreenwideAnnotations) == 40, "ScreenwideAnnotations ABI must match Rust");
_Static_assert(offsetof(ScreenwideAnnotations, data) == 16, "ScreenwideAnnotations.data ABI must match Rust");

typedef struct {
  ScreenwideAnnotation annotation;
  uint64_t start_ms, end_ms;
} ScreenwideTimedAnnotation;
_Static_assert(sizeof(ScreenwideTimedAnnotation) == 144, "Timed annotation ABI");
