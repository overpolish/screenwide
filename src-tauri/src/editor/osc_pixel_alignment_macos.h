// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos.h"

// Align axis-aligned OSC quads at the upload boundary, where the complete
// viewport and backing scale are known. Diagonal strokes retain their slope;
// captured image content (kind 33) retains its scene transform and UV mapping.
static inline void screenwide_osc_align_vertices(
    ScreenwideRegionOscVertex *vertices, NSUInteger count, NSSize size,
    CGFloat scale) {
  if (scale <= 0.0 || size.width <= 0.0 || size.height <= 0.0) return;
  CGFloat width = round(size.width * scale);
  CGFloat height = round(size.height * scale);
  if (width < 1.0 || height < 1.0) return;
  for (NSUInteger offset = 0; offset + 5 < count; offset += 6) {
    ScreenwideRegionOscVertex *quad = vertices + offset;
    if (quad[0].kind == 33) continue;
    CGFloat x[6], y[6];
    CGFloat minX = INFINITY, minY = INFINITY;
    CGFloat maxX = -INFINITY, maxY = -INFINITY;
    for (NSUInteger i = 0; i < 6; i++) {
      x[i] = (quad[i].position.x + 1.0) * width * 0.5;
      y[i] = (1.0 - quad[i].position.y) * height * 0.5;
      minX = MIN(minX, x[i]); maxX = MAX(maxX, x[i]);
      minY = MIN(minY, y[i]); maxY = MAX(maxY, y[i]);
    }
    BOOL axisAligned = YES;
    for (NSUInteger i = 0; i < 6; i++)
      if ((fabs(x[i] - minX) > 0.001 && fabs(x[i] - maxX) > 0.001) ||
          (fabs(y[i] - minY) > 0.001 && fabs(y[i] - maxY) > 0.001))
        axisAligned = NO;
    if (!axisAligned || maxX <= minX || maxY <= minY) continue;
    CGFloat left = round(minX), top = round(minY);
    CGFloat right = MAX(round(maxX), left + 1.0);
    CGFloat bottom = MAX(round(maxY), top + 1.0);
    for (NSUInteger i = 0; i < 6; i++) {
      CGFloat px = fabs(x[i] - minX) < 0.001 ? left : right;
      CGFloat py = fabs(y[i] - minY) < 0.001 ? top : bottom;
      quad[i].position.x = (float)(2.0 * px / width - 1.0);
      quad[i].position.y = (float)(1.0 - 2.0 * py / height);
    }
  }
}

static inline id<MTLBuffer> screenwide_osc_vertex_buffer(
    id<MTLDevice> device, ScreenwideRegionOscVertex *vertices, NSUInteger count,
    NSSize size, CGFloat scale) {
  screenwide_osc_align_vertices(vertices, count, size, scale);
  return [device newBufferWithBytes:vertices length:count * sizeof(*vertices)
                           options:MTLResourceStorageModeShared];
}
