// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>
#include <string.h>
#import "gpu_compositor_macos.h"
#import "gpu_compositor_macos_annotation_types.h"
#import "../annotations/geometry.h"

/// A draw-ready mark, distinct from the source-space document retained by the
/// presenter. Preparing at binding also covers native placement changes.
typedef struct {
  uint32_t kind, above_camera;
  float color[4];
  float hover;
  AnnotationArrowGeometry arrow;
} ScreenwidePreparedAnnotation;
_Static_assert(sizeof(ScreenwidePreparedAnnotation) == 120, "Prepared annotation ABI");
_Static_assert(sizeof(ScreenwidePreparedAnnotation) * SCREENWIDE_MAX_ANNOTATIONS <= 4096,
               "Metal inline annotation bytes must fit setBytes");

/// Binds prepared geometry for both still export and retained workspace draws.
static inline void screenwide_bind_annotations(
    id<MTLComputeCommandEncoder> encoder, const ScreenwideAnnotations *annotations,
    const ScreenwideCanvas *canvas, uint32_t source_width, uint32_t source_height) {
  uint32_t count = annotations == NULL ? 0
      : MIN(annotations->count, (uint32_t)SCREENWIDE_MAX_ANNOTATIONS);
  ScreenwidePreparedAnnotation prepared[SCREENWIDE_MAX_ANNOTATIONS] = {0};
  float scale_x = (float)canvas->image_width / MAX(source_width, 1u);
  float scale_y = (float)canvas->image_height / MAX(source_height, 1u);
  for (uint32_t index = 0; index < count; index++) {
    const ScreenwideAnnotation *mark = &annotations->items[index];
    ScreenwidePreparedAnnotation *draw = &prepared[index];
    draw->kind = mark->kind;
    draw->above_camera = mark->above_camera;
    memcpy(draw->color, mark->color, sizeof(draw->color));
    draw->hover = mark->hover;
    draw->arrow = annotation_prepare_arrow(
        annotation_vector(canvas->image_x + mark->p0[0] * scale_x,
                          canvas->image_y + mark->p0[1] * scale_y),
        annotation_vector(canvas->image_x + mark->p1[0] * scale_x,
                          canvas->image_y + mark->p1[1] * scale_y),
        annotation_vector(canvas->image_x + mark->p2[0] * scale_x,
                          canvas->image_y + mark->p2[1] * scale_y), mark->width, mark->head);
  }
  [encoder setBytes:prepared length:sizeof(prepared) atIndex:12];
  [encoder setBytes:&count length:sizeof(count) atIndex:13];
}
