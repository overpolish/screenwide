// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_export_private.h"

/// Uploads one RGBA slice per cursor style. Every slice shares the largest
/// artwork's dimensions; each style only ever samples its own recorded size.
id<MTLTexture> screenwide_export_cursor_artwork_texture(
    id<MTLDevice> device, const ScreenwideCursorArtwork *artworks,
    uint32_t count) {
  if (artworks == NULL || count == 0)
    return nil;
  uint32_t width = 0;
  uint32_t height = 0;
  for (uint32_t index = 0; index < count; ++index) {
    if (artworks[index].pixels == NULL)
      continue;
    width = MAX(width, artworks[index].width);
    height = MAX(height, artworks[index].height);
  }
  if (width == 0 || height == 0)
    return nil;
  MTLTextureDescriptor *description = [[MTLTextureDescriptor alloc] init];
  description.textureType = MTLTextureType2DArray;
  description.pixelFormat = MTLPixelFormatRGBA8Unorm;
  description.width = width;
  description.height = height;
  description.arrayLength = count;
  description.usage = MTLTextureUsageShaderRead;
  id<MTLTexture> texture = [device newTextureWithDescriptor:description];
  if (texture == nil)
    return nil;
  for (uint32_t index = 0; index < count; ++index) {
    const ScreenwideCursorArtwork *artwork = &artworks[index];
    if (artwork->pixels == NULL || artwork->width == 0 || artwork->height == 0)
      continue;
    [texture
        replaceRegion:MTLRegionMake2D(0, 0, artwork->width, artwork->height)
          mipmapLevel:0
                slice:index
            withBytes:artwork->pixels
          bytesPerRow:(NSUInteger)artwork->width * 4
        bytesPerImage:(NSUInteger)artwork->width * artwork->height * 4];
  }
  return texture;
}

/// The cursor for one output frame. Frame `n` of the 60 Hz cursor grid becomes
/// current the moment its own timestamp is reached, which is exactly how the
/// retired positions sidecar was indexed against the cursor movie.
const ScreenwideGpuCursor *
screenwide_export_cursor_at(const ScreenwideGpuCursor *cursors, uint32_t count,
                            CMTime pts) {
  if (cursors == NULL || count == 0)
    return NULL;
  double seconds = CMTimeGetSeconds(pts);
  if (!isfinite(seconds) || seconds < 0.0)
    seconds = 0.0;
  // Frame timestamps are exact sixtieths, so a presentation time landing on
  // one must select it rather than its predecessor.
  int64_t index = (int64_t)floor(seconds * 60.0 + 1e-6);
  if (index > (int64_t)count - 1)
    index = (int64_t)count - 1;
  return &cursors[index];
}

/// Draws the cursor into the composed frame's planes. The shader owns the
/// pixels; this only sizes the dispatch to the cursor's bounds
/// (`raster::bounds`, cursor_effects/raster.rs:294-307).
void screenwide_export_encode_cursor_overlay(
    id<MTLCommandBuffer> command, id<MTLComputePipelineState> luma_pipeline,
    id<MTLComputePipelineState> chroma_pipeline, id<MTLTexture> destination_y,
    id<MTLTexture> destination_uv, id<MTLTexture> artwork_texture,
    const ScreenwideGpuCursor *cursor, const ScreenwideCursorArtwork *artworks,
    uint32_t artwork_count, const ScreenwideCanvas *canvas,
    uint32_t output_width, uint32_t output_height) {
  if (cursor == NULL || cursor->visible == 0 || artwork_texture == nil ||
      cursor->style >= artwork_count)
    return;
  const ScreenwideCursorArtwork *artwork = &artworks[cursor->style];
  if (artwork->pixels == NULL || artwork->width == 0 || artwork->height == 0)
    return;
  double travel = hypot(cursor->blur_delta_x, cursor->blur_delta_y);
  double distance = MIN(travel, 80.0);
  double blur = distance > 1.25 ? distance : 0.0;
  double radius = hypot(cursor->width, cursor->height) * cursor->scale;
  double left = floor(cursor->x - radius - blur);
  double top = floor(cursor->y - radius - blur);
  double right = ceil(cursor->x + radius + blur);
  double bottom = ceil(cursor->y + radius + blur);
  left = MAX(left, 0.0);
  top = MAX(top, 0.0);
  right = MIN(right, (double)output_width);
  bottom = MIN(bottom, (double)output_height);
  // A chroma thread owns the four output pixels of one chroma sample, so an
  // even origin keeps two threads from writing the same chroma pixel.
  int32_t x = (int32_t)left & ~1;
  int32_t y = (int32_t)top & ~1;
  if (right <= (double)x || bottom <= (double)y)
    return;
  uint32_t box_width = (uint32_t)(right - (double)x);
  uint32_t box_height = (uint32_t)(bottom - (double)y);
  ScreenwideOverlayUniforms uniforms = {
      x,
      y,
      box_width,
      box_height,
      output_width,
      output_height,
      canvas->crop_x,
      canvas->crop_y,
      canvas->crop_width,
      canvas->crop_height,
      canvas->radius,
      // The frame carries the same setting the canvas does; taking it from the
      // cursor keeps one owner of the effect for the drawn cursor.
      cursor->clip_at_video_edge,
      *cursor,
      {
          artwork->width,
          artwork->height,
          artwork->design_width,
          artwork->design_height,
          artwork->origin_x,
          artwork->origin_y,
          artwork->use_design,
          artwork->clip_local_box,
          artwork->supersample,
      },
  };
  MTLSize group = MTLSizeMake(16, 16, 1);
  id<MTLComputeCommandEncoder> compute = [command computeCommandEncoder];
  [compute setComputePipelineState:luma_pipeline];
  [compute setTexture:artwork_texture atIndex:0];
  [compute setTexture:destination_y atIndex:1];
  [compute setBytes:&uniforms length:sizeof(uniforms) atIndex:0];
  [compute dispatchThreads:MTLSizeMake(box_width, box_height, 1)
      threadsPerThreadgroup:group];
  [compute endEncoding];
  compute = [command computeCommandEncoder];
  [compute setComputePipelineState:chroma_pipeline];
  [compute setTexture:artwork_texture atIndex:0];
  [compute setTexture:destination_uv atIndex:1];
  [compute setBytes:&uniforms length:sizeof(uniforms) atIndex:0];
  [compute dispatchThreads:MTLSizeMake((box_width + 1) / 2,
                                       (box_height + 1) / 2, 1)
      threadsPerThreadgroup:group];
  [compute endEncoding];
}
