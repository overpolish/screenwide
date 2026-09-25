// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <Metal/Metal.h>
#include <stdint.h>

#import "gpu_compositor_macos_annotation_types.h"

/// How a redaction's covered pixels are painted.
enum {
  SCREENWIDE_REDACT_FLAT = 0u,
  SCREENWIDE_REDACT_PIXELATE = 1u,
  SCREENWIDE_REDACT_BLUR = 2u,
  SCREENWIDE_REDACT_MOSAIC = 3u,
};

/// One redaction in whole source pixels, `[x0, x1)` by `[y0, y1)`, as the
/// redaction kernels read it. The twin of `RedactUniforms`.
typedef struct {
  uint32_t x0, y0, x1, y1;
  uint32_t source_width;
  uint32_t mode;
  uint32_t seed;
  /// A pixelated box's block, or a blurred box's cell, in source pixels.
  float size;
  float color[4];
  /// The grid the box draws from: a pixelated box's zones across and blocks
  /// per zone, or a classically pixelated or blurred box's cells across and
  /// down; and how many zones follow this record.
  uint32_t grid[2];
  uint32_t entry_count;
  /// The corner radius in source pixels.
  float radius;
} ScreenwideRedaction;
_Static_assert(sizeof(ScreenwideRedaction) == 64, "Redaction uniforms ABI");

/// The kernels a redaction is applied with: the cells pass that averages a
/// classic pixelation's blocks or a blur's cells, and the paint pass, over an
/// RGBA source; and the same over a video frame's two planes.
@interface ScreenwideRedactPipelines : NSObject
@property(nonatomic, strong) id<MTLComputePipelineState> cells;
@property(nonatomic, strong) id<MTLComputePipelineState> paint;
@property(nonatomic, strong) id<MTLComputePipelineState> videoCells;
@property(nonatomic, strong) id<MTLComputePipelineState> videoLuma;
@property(nonatomic, strong) id<MTLComputePipelineState> videoChroma;
@end

/// The redaction kernels from `library`, or nil where any is missing.
ScreenwideRedactPipelines *screenwide_redact_pipelines(id<MTLLibrary> library);

/// The redactions in `annotations`, snapped outward to whole pixels of a
/// `width` by `height` source and clipped to it: each record followed by its
/// `entry_count` zones, two floats each. Empty when there are none, and
/// equal for two lists that redact the same pixels the same way, which is
/// what lets a retained source skip a pass that would change nothing.
NSData *screenwide_redactions(const ScreenwideAnnotations *annotations,
                              uint32_t width, uint32_t height);

/// Encodes the passes for every redaction over `pixels`, an RGBA source
/// buffer, in order: a box drawn over another averages what the first left.
void screenwide_encode_redactions(id<MTLCommandBuffer> command,
                                  ScreenwideRedactPipelines *pipelines,
                                  id<MTLBuffer> pixels, NSData *redactions);

/// Encodes the passes for every redaction over a video frame's `luma` and
/// `chroma` planes, which the passes read and write in place, in order.
void screenwide_encode_video_redactions(id<MTLCommandBuffer> command,
                                        ScreenwideRedactPipelines *pipelines,
                                        id<MTLTexture> luma, id<MTLTexture> chroma,
                                        NSData *redactions);

/// The surface a redaction's timeline of `count` entries gives `elapsed_ms`
/// into its clip, as three floats in `out`; zero where it gives none. Rust's
/// `redact::surface_timeline::surface_at`, which the preview reads too.
uint32_t screenwide_redaction_surface_at(const float (*entries)[2], uint32_t count,
                                         float elapsed_ms, float *out);
