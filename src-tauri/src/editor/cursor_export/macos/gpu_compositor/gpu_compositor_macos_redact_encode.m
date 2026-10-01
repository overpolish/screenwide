// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The redaction kernels, and the passes that apply the records
// `gpu_compositor_macos_redact.m` packs.

#import "gpu_compositor_macos_redact.h"

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
  pipelines.rows = redact_pipeline(library, @"redact_rows_rgba");
  pipelines.paint = redact_pipeline(library, @"redact_source_rgba");
  pipelines.videoCells = redact_pipeline(library, @"redact_cells_video");
  pipelines.videoRows = redact_pipeline(library, @"redact_rows_video");
  pipelines.videoLuma = redact_pipeline(library, @"redact_video_luma");
  pipelines.videoChroma = redact_pipeline(library, @"redact_video_chroma");
  return pipelines.cells != nil && pipelines.rows != nil && pipelines.paint != nil &&
                 pipelines.videoCells != nil && pipelines.videoRows != nil &&
                 pipelines.videoLuma != nil && pipelines.videoChroma != nil
             ? pipelines
             : nil;
}

/// Binds the record at `*offset` in `redactions` and the zones after it,
/// with a fresh cells buffer where it averages cells and a fresh rows buffer
/// where it blurs, and steps past them. Answers the record, how many cells
/// it averages, and whether it blurs.
static const ScreenwideRedaction *bind_redaction(id<MTLComputeCommandEncoder> encoder,
                                                 id<MTLDevice> device, NSData *redactions,
                                                 NSUInteger *offset, NSUInteger *cells,
                                                 BOOL *blurs) {
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
  *cells = redaction->mode == SCREENWIDE_REDACT_MOSAIC
      ? (NSUInteger)redaction->grid[0] * redaction->grid[1]
      : 0;
  if (*cells > 0)
    [encoder setBuffer:[device newBufferWithLength:*cells * sizeof(uint32_t)
                                           options:MTLResourceStorageModePrivate]
                offset:0
               atIndex:3];
  else {
    const uint32_t none = 0;
    [encoder setBytes:&none length:sizeof(none) atIndex:3];
  }
  *blurs = redaction->mode == SCREENWIDE_REDACT_BLUR ||
           redaction->mode == SCREENWIDE_REDACT_SPOTLIGHT;
  // The rows pass writes one half-float colour a pixel of the box.
  NSUInteger pixels =
      (NSUInteger)(redaction->x1 - redaction->x0) * (redaction->y1 - redaction->y0);
  if (*blurs && pixels > 0)
    [encoder setBuffer:[device newBufferWithLength:pixels * 4 * sizeof(uint16_t)
                                           options:MTLResourceStorageModePrivate]
                offset:0
               atIndex:4];
  else {
    const uint16_t none[4] = {0, 0, 0, 0};
    [encoder setBytes:none length:sizeof(none) atIndex:4];
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
    BOOL blurs = NO;
    const ScreenwideRedaction *redaction =
        bind_redaction(encoder, pixels.device, redactions, &offset, &cells, &blurs);
    NSUInteger wide = redaction->x1 - redaction->x0, tall = redaction->y1 - redaction->y0;
    average_cells(encoder, pipelines.cells, cells);
    if (blurs) dispatch_over(encoder, pipelines.rows, wide, tall);
    dispatch_over(encoder, pipelines.paint, wide, tall);
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
    BOOL blurs = NO;
    const ScreenwideRedaction *redaction =
        bind_redaction(encoder, luma.device, redactions, &offset, &cells, &blurs);
    [encoder setTexture:luma atIndex:0];
    [encoder setTexture:chroma atIndex:1];
    NSUInteger wide = redaction->x1 - redaction->x0, tall = redaction->y1 - redaction->y0;
    average_cells(encoder, pipelines.videoCells, cells);
    if (blurs) dispatch_over(encoder, pipelines.videoRows, wide, tall);
    dispatch_over(encoder, pipelines.videoLuma, wide, tall);
    // One thread a colour sample the box touches, from its first.
    [encoder setTexture:chroma atIndex:0];
    dispatch_over(encoder, pipelines.videoChroma,
                  (redaction->x1 + 1) / 2 - redaction->x0 / 2,
                  (redaction->y1 + 1) / 2 - redaction->y0 / 2);
  }
  [encoder endEncoding];
}
