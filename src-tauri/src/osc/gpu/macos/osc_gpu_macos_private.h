// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_OSC_GPU_MACOS_PRIVATE_H
#define SCREENWIDE_OSC_GPU_MACOS_PRIVATE_H

#import "osc_gpu_macos.h"

/// Scene geometry arrives in view coordinates with a top-left origin, which
/// every builder converts to the clip space the vertex stage expects.
static inline ScreenwideRegionOscPoint ndc(NSSize size, CGFloat x, CGFloat y) {
  return (ScreenwideRegionOscPoint){
      (float)(2.0 * x / MAX(size.width, 1.0) - 1.0),
      (float)(1.0 - 2.0 * y / MAX(size.height, 1.0)),
  };
}

/// The handle artwork and the striped edges the selection frame and the crop
/// frame both draw, kept in `osc_gpu_macos+primitives.m`.
void screenwide_region_osc_add_pattern_quad(ScreenwideRegionOscVertex *vertices,
                                            NSUInteger *count, NSSize size,
                                            NSRect rect, uint32_t kind,
                                            BOOL horizontal, CGFloat scale,
                                            CGFloat origin);
void screenwide_region_osc_add_circle(ScreenwideRegionOscVertex *vertices,
                                      NSUInteger *count, NSSize size,
                                      NSPoint center, CGFloat radius,
                                      CGFloat margin, uint32_t kind);
void screenwide_region_osc_add_pill(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    NSPoint center, BOOL horizontal,
                                    CGFloat scale);
/// A handle centre lands on a whole pixel, unlike the half-pixel snapping a
/// hairline edge needs.
NSPoint screenwide_region_osc_snap_handle_point(NSPoint point, CGFloat scale);

#endif
