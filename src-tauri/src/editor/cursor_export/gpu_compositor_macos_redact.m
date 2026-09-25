// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_redact.h"

#include <math.h>

@implementation ScreenwideRedactPipelines
@end

/// A pipeline for the kernel `name` in `library`, or nil.
static id<MTLComputePipelineState> redact_pipeline(id<MTLLibrary> library, NSString *name) {
  id<MTLFunction> function = [library newFunctionWithName:name];
  if (function == nil) return nil;
  NSError *error = nil;
  return [library.device newComputePipelineStateWithFunction:function error:&error];
}

ScreenwideRedactPipelines *screenwide_redact_pipelines(id<MTLLibrary> library) {
  ScreenwideRedactPipelines *pipelines = [ScreenwideRedactPipelines new];
  pipelines.cells = redact_pipeline(library, @"redact_cells_rgba");
  pipelines.paint = redact_pipeline(library, @"redact_source_rgba");
  pipelines.videoCells = redact_pipeline(library, @"redact_cells_video");
  pipelines.videoLuma = redact_pipeline(library, @"redact_video_luma");
  pipelines.videoChroma = redact_pipeline(library, @"redact_video_chroma");
  return pipelines.cells != nil && pipelines.paint != nil && pipelines.videoCells != nil &&
                 pipelines.videoLuma != nil && pipelines.videoChroma != nil
             ? pipelines
             : nil;
}

/// A corner coordinate as a whole pixel inside `[0, limit]`. The box is
/// snapped outward - down at its top-left, up at its bottom-right - so no
/// covered pixel is left partly showing along the edge. The twin of Rust's
/// `redact::native::painted_bounds`, which samples the fill colour just
/// outside these same pixels.
static uint32_t redaction_edge(float value, BOOL far, uint32_t limit) {
  float edge = far ? ceilf(value) : floorf(value);
  if (!(edge > 0.0f)) return 0;
  return edge >= (float)limit ? limit : (uint32_t)edge;
}

/// A whole count a record carries as a float, or zero for one it cannot.
static uint32_t redaction_count(float value) {
  return value >= 1.0f && value < 16777216.0f ? (uint32_t)value : 0;
}

/// The most cells the cells pass averages for one box. Rust sizes the cells
/// well under this; a ramp's first small cells are grown to stay within it.
static const uint64_t MAX_CELLS = 1u << 20;

/// How far an animated redaction has arrived, from its reveal: 0 at the
/// start of its clip and 1 once whole, which every still and every
/// redaction that does not animate is.
static float redaction_arrived(const ScreenwideAnnotation *item) {
  float arrived = item->reveal.opacity;
  return isfinite(arrived) ? fminf(fmaxf(arrived, 0.0f), 1.0f) : 1.0f;
}

NSData *screenwide_redactions(const ScreenwideAnnotations *annotations,
                              uint32_t width, uint32_t height) {
  NSMutableData *redactions = [NSMutableData data];
  if (annotations == NULL || annotations->items == NULL) return redactions;
  for (uint32_t index = 0; index < annotations->count; ++index) {
    const ScreenwideAnnotation *item = &annotations->items[index];
    if (item->kind != SCREENWIDE_ANNOTATION_REDACT) continue;
    // `p0` is the top-left corner and `p2` the bottom-right.
    uint32_t mode = (item->flags & SCREENWIDE_ANNOTATION_FLAG_PIXELATE) != 0
        ? SCREENWIDE_REDACT_PIXELATE
        : (item->flags & SCREENWIDE_ANNOTATION_FLAG_BLUR) != 0   ? SCREENWIDE_REDACT_BLUR
        : (item->flags & SCREENWIDE_ANNOTATION_FLAG_MOSAIC) != 0 ? SCREENWIDE_REDACT_MOSAIC
                                                                 : SCREENWIDE_REDACT_FLAT;
    ScreenwideRedaction redaction = {
      .x0 = redaction_edge(item->p0[0], NO, width),
      .y0 = redaction_edge(item->p0[1], NO, height),
      .x1 = redaction_edge(item->p2[0], YES, width),
      .y1 = redaction_edge(item->p2[1], YES, height),
      .source_width = width,
      .mode = mode,
      // The seed rides in two float halves, each exact.
      .seed = ((uint32_t)item->params[1] << 16) | ((uint32_t)item->params[2] & 0xffffu),
      .size = item->params[0],
    };
    if (redaction.x1 <= redaction.x0 || redaction.y1 <= redaction.y0) continue;
    memcpy(redaction.color, item->color, sizeof(redaction.color));
    // The record's width is the radius as a share of the painted box's
    // shorter side, so the halo drawn from the unsnapped box rounds alike.
    uint32_t wide = redaction.x1 - redaction.x0, tall = redaction.y1 - redaction.y0;
    float share = isfinite(item->width) ? fminf(fmaxf(item->width, 0.0f), 50.0f) : 0.0f;
    redaction.radius = (float)MIN(wide, tall) * share / 100.0f;
    // An animated redaction arrives by covering more: a classic pixelation's
    // blocks and a blur's cells grow from a pixel to their size, and every
    // other fill fades in, its share riding in the colour's alpha, which an
    // opaque fill has no other use for.
    float arrived = redaction_arrived(item);
    const float(*zones)[2] = NULL;
    if (mode == SCREENWIDE_REDACT_BLUR || mode == SCREENWIDE_REDACT_MOSAIC) {
      redaction.color[3] = 1.0f;
      // Cells are averaged on the GPU from the pixels themselves, so their
      // grid follows from the snapped box and the cell's size.
      float size = isfinite(redaction.size) ? fmaxf(redaction.size, 1.0f) : 1.0f;
      size = 1.0f + (size - 1.0f) * arrived;
      size = fmaxf(size, sqrtf((float)wide * (float)tall / (float)MAX_CELLS));
      redaction.size = size;
      redaction.grid[0] = (uint32_t)MAX(ceilf((float)wide / size), 1.0f);
      redaction.grid[1] = (uint32_t)MAX(ceilf((float)tall / size), 1.0f);
    } else {
      redaction.color[3] = arrived;
      if (mode == SCREENWIDE_REDACT_PIXELATE && annotations->data.points != NULL &&
          (uint64_t)item->data_offset + item->data_count <= annotations->data.point_count) {
        // A pixelated box's zones, which a list whose side buffer does not
        // hold them all goes without.
        redaction.grid[0] = redaction_count(item->p3[0]);
        redaction.grid[1] = redaction_count(item->p3[1]);
        redaction.entry_count =
            redaction.grid[0] == 0 || redaction.grid[1] == 0 ? 0 : item->data_count;
        zones = annotations->data.points + item->data_offset;
      }
    }
    [redactions appendBytes:&redaction length:sizeof(redaction)];
    if (redaction.entry_count > 0)
      [redactions appendBytes:zones
                       length:(NSUInteger)redaction.entry_count * sizeof(float[2])];
  }
  return redactions;
}

/// Binds the record at `*offset` in `redactions` and the zones after it,
/// with a fresh cells buffer where it averages cells, and steps past them.
/// Answers the record, and how many cells it averages; zero for none.
static const ScreenwideRedaction *bind_redaction(id<MTLComputeCommandEncoder> encoder,
                                                 id<MTLDevice> device, NSData *redactions,
                                                 NSUInteger *offset, NSUInteger *cells) {
  const uint8_t *bytes = redactions.bytes;
  const ScreenwideRedaction *redaction = (const ScreenwideRedaction *)(bytes + *offset);
  *offset += sizeof(*redaction);
  NSUInteger zones_length = (NSUInteger)redaction->entry_count * sizeof(float[2]);
  [encoder setBytes:redaction length:sizeof(*redaction) atIndex:1];
  // A box's zones outgrow the 4 KiB that bytes can carry, so they are a
  // buffer; the kernel never reads one it was not given zones for.
  if (zones_length > 0)
    [encoder setBuffer:[device newBufferWithBytes:bytes + *offset
                                           length:zones_length
                                          options:MTLResourceStorageModeShared]
                offset:0
               atIndex:2];
  else {
    const float none[2] = {-1.0f, -1.0f};
    [encoder setBytes:none length:sizeof(none) atIndex:2];
  }
  *offset += zones_length;
  BOOL averaged = redaction->mode == SCREENWIDE_REDACT_BLUR ||
                  redaction->mode == SCREENWIDE_REDACT_MOSAIC;
  *cells = averaged ? (NSUInteger)redaction->grid[0] * redaction->grid[1] : 0;
  if (*cells > 0)
    [encoder setBuffer:[device newBufferWithLength:*cells * sizeof(uint32_t)
                                           options:MTLResourceStorageModePrivate]
                offset:0
               atIndex:3];
  else {
    const uint32_t none = 0;
    [encoder setBytes:&none length:sizeof(none) atIndex:3];
  }
  return redaction;
}

/// Dispatches `pipeline` over a `width` by `height` grid.
static void dispatch_over(id<MTLComputeCommandEncoder> encoder,
                          id<MTLComputePipelineState> pipeline, NSUInteger width,
                          NSUInteger height) {
  if (width == 0 || height == 0) return;
  NSUInteger group_width = pipeline.threadExecutionWidth;
  NSUInteger group_height = MAX((NSUInteger)1, pipeline.maxTotalThreadsPerThreadgroup /
                                                   MAX(group_width, (NSUInteger)1));
  [encoder setComputePipelineState:pipeline];
  [encoder dispatchThreads:MTLSizeMake(width, height, 1)
      threadsPerThreadgroup:MTLSizeMake(MIN(group_width, width), MIN(group_height, height), 1)];
}

/// Dispatches the cells pass, one threadgroup a cell.
static void average_cells(id<MTLComputeCommandEncoder> encoder,
                          id<MTLComputePipelineState> pipeline, NSUInteger cells) {
  if (cells == 0) return;
  [encoder setComputePipelineState:pipeline];
  [encoder dispatchThreadgroups:MTLSizeMake(cells, 1, 1)
          threadsPerThreadgroup:MTLSizeMake(256, 1, 1)];
}

void screenwide_encode_redactions(id<MTLCommandBuffer> command,
                                  ScreenwideRedactPipelines *pipelines,
                                  id<MTLBuffer> pixels, NSData *redactions) {
  if (redactions.length < sizeof(ScreenwideRedaction) || command == nil ||
      pipelines == nil || pixels == nil)
    return;
  // Dispatches in one encoder run in order, each seeing what the last wrote.
  id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
  [encoder setBuffer:pixels offset:0 atIndex:0];
  NSUInteger offset = 0;
  while (offset + sizeof(ScreenwideRedaction) <= redactions.length) {
    NSUInteger cells = 0;
    const ScreenwideRedaction *redaction =
        bind_redaction(encoder, pixels.device, redactions, &offset, &cells);
    average_cells(encoder, pipelines.cells, cells);
    dispatch_over(encoder, pipelines.paint, redaction->x1 - redaction->x0,
                  redaction->y1 - redaction->y0);
  }
  [encoder endEncoding];
}

void screenwide_encode_video_redactions(id<MTLCommandBuffer> command,
                                        ScreenwideRedactPipelines *pipelines,
                                        id<MTLTexture> luma, id<MTLTexture> chroma,
                                        NSData *redactions) {
  if (redactions.length < sizeof(ScreenwideRedaction) || command == nil ||
      pipelines == nil || luma == nil || chroma == nil)
    return;
  id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
  NSUInteger offset = 0;
  while (offset + sizeof(ScreenwideRedaction) <= redactions.length) {
    NSUInteger cells = 0;
    const ScreenwideRedaction *redaction =
        bind_redaction(encoder, luma.device, redactions, &offset, &cells);
    [encoder setTexture:luma atIndex:0];
    [encoder setTexture:chroma atIndex:1];
    average_cells(encoder, pipelines.videoCells, cells);
    NSUInteger wide = redaction->x1 - redaction->x0, tall = redaction->y1 - redaction->y0;
    dispatch_over(encoder, pipelines.videoLuma, wide, tall);
    // One thread a colour sample the box touches, from its first.
    [encoder setTexture:chroma atIndex:0];
    dispatch_over(encoder, pipelines.videoChroma,
                  (redaction->x1 + 1) / 2 - redaction->x0 / 2,
                  (redaction->y1 + 1) / 2 - redaction->y0 / 2);
  }
  [encoder endEncoding];
}
