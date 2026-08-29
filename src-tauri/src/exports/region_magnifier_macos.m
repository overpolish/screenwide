// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "region_osc_renderer_macos.h"

id<MTLComputePipelineState> screenwide_region_magnifier_make_pipeline(
    id<MTLDevice> device, id<MTLLibrary> library, NSError **error) {
  return [device newComputePipelineStateWithFunction:
                     [library newFunctionWithName:@"region_magnifier"]
                                           error:error];
}

NSPoint screenwide_region_magnifier_anchor(NSPoint point, NSRect frame,
                                           uint32_t edges) {
  CGFloat x = (edges & 1) != 0 ? NSMinX(frame)
              : (edges & 2) != 0 ? NSMaxX(frame)
                                 : point.x;
  CGFloat y = (edges & 4) != 0 ? NSMinY(frame)
              : (edges & 8) != 0 ? NSMaxY(frame)
                                 : point.y;
  return NSMakePoint(fmin(NSMaxX(frame), fmax(NSMinX(frame), x)),
                     fmin(NSMaxY(frame), fmax(NSMinY(frame), y)));
}

static float unit(float value) {
  return fminf(1.0f, fmaxf(0.0f, value));
}

ScreenwideRegionMagnifier screenwide_region_magnifier_make(
    NSPoint point, CGFloat scale, uint32_t edges, uint32_t light_mode,
    uint32_t pane_index, uint32_t layer_id, uint32_t sample_camera,
    float sample_u, float sample_v, float source_min_u, float source_min_v,
    float source_max_u, float source_max_v) {
  int32_t size = (int32_t)MAX(llround(96.0 * scale), 1);
  int32_t center_x = (int32_t)llround(point.x * scale);
  int32_t center_y = (int32_t)llround(point.y * scale);
  return (ScreenwideRegionMagnifier){
      .active = 1,
      .pane_index = pane_index,
      .layer_id = layer_id,
      .sample_camera = sample_camera,
      .edges = edges,
      .light_mode = light_mode,
      .sample_u = unit(sample_u),
      .sample_v = unit(sample_v),
      .source_min_u = unit(source_min_u),
      .source_min_v = unit(source_min_v),
      .source_max_u = unit(source_max_u),
      .source_max_v = unit(source_max_v),
      .box_x = center_x - size / 2,
      .box_y = center_y - size / 2,
      .box_width = (uint32_t)size,
      .box_height = (uint32_t)size,
  };
}

void screenwide_region_magnifier_encode(
    id<MTLComputeCommandEncoder> encoder,
    id<MTLComputePipelineState> pipeline, id<MTLBuffer> source,
    id<MTLTexture> output, const uint32_t source_dimensions[2],
    ScreenwideRegionMagnifier magnifier) {
  [encoder setComputePipelineState:pipeline];
  [encoder setBuffer:source offset:0 atIndex:0];
  [encoder setTexture:output atIndex:0];
  [encoder setBytes:source_dimensions
             length:sizeof(uint32_t) * 2
            atIndex:1];
  [encoder setBytes:&magnifier length:sizeof(magnifier) atIndex:2];
  NSUInteger width = pipeline.threadExecutionWidth;
  NSUInteger height = MAX(pipeline.maxTotalThreadsPerThreadgroup / width, 1);
  [encoder dispatchThreads:MTLSizeMake(MAX(magnifier.box_width, 1),
                                       MAX(magnifier.box_height, 1), 1)
      threadsPerThreadgroup:MTLSizeMake(width, height, 1)];
}
