// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>

#import "gpu_compositor_macos_annotation_types.h"

/// Binds one layer's marks for `compose_canvas_rgba` and `workspace_layer`.
/// The list is small enough to travel as inline bytes, and the zeroed array
/// keeps the binding valid when a layer has no marks at all.
static inline void screenwide_bind_annotations(
    id<MTLComputeCommandEncoder> encoder,
    const ScreenwideAnnotations *annotations) {
  uint32_t count = annotations == NULL ? 0
      : MIN(annotations->count, (uint32_t)SCREENWIDE_MAX_ANNOTATIONS);
  ScreenwideAnnotations empty = {0};
  const ScreenwideAnnotations *source = annotations ?: &empty;
  [encoder setBytes:source->items
             length:sizeof(source->items)
            atIndex:12];
  [encoder setBytes:&count length:sizeof(count) atIndex:13];
}
