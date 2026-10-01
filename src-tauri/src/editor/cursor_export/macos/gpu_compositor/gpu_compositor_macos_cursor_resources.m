// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_cursor_resources.h"

@implementation ScreenwideCursorResources
@end

ScreenwideCursorResources *screenwide_cursor_resources(
    id<MTLDevice> device, const ScreenwideCursorArtwork *artworks,
    uint32_t artwork_count) {
  if (device == nil || (artworks == NULL && artwork_count != 0)) return nil;
  uint32_t width = 1, height = 1;
  for (uint32_t index = 0; index < artwork_count; ++index) {
    width = MAX(width, artworks[index].width);
    height = MAX(height, artworks[index].height);
  }
  if (width == 0 || height == 0) return nil;
  MTLTextureDescriptor *description = [MTLTextureDescriptor new];
  description.textureType = MTLTextureType2DArray;
  description.pixelFormat = MTLPixelFormatRGBA8Unorm;
  description.width = width;
  description.height = height;
  description.arrayLength = MAX(artwork_count, 1u);
  description.usage = MTLTextureUsageShaderRead;
  id<MTLTexture> texture = [device newTextureWithDescriptor:description];
  if (texture == nil) return nil;
  NSMutableData *uniforms = [NSMutableData
      dataWithLength:sizeof(ScreenwideCursorArtworkUniforms) *
                     MAX(artwork_count, 1u)];
  ScreenwideCursorArtworkUniforms *mapped = uniforms.mutableBytes;
  for (uint32_t index = 0; index < artwork_count; ++index) {
    const ScreenwideCursorArtwork *artwork = &artworks[index];
    if (artwork->pixels == NULL || artwork->width == 0 || artwork->height == 0)
      return nil;
    [texture replaceRegion:MTLRegionMake2D(0, 0, artwork->width, artwork->height)
               mipmapLevel:0 slice:index withBytes:artwork->pixels
               bytesPerRow:(NSUInteger)artwork->width * 4
             bytesPerImage:(NSUInteger)artwork->width * artwork->height * 4];
    mapped[index] = (ScreenwideCursorArtworkUniforms){
        artwork->width, artwork->height, artwork->design_width,
        artwork->design_height, artwork->origin_x, artwork->origin_y,
        artwork->use_design, artwork->clip_local_box, artwork->supersample};
  }
  ScreenwideCursorResources *resources = [ScreenwideCursorResources new];
  resources.texture = texture;
  resources.uniforms = uniforms;
  resources.count = artwork_count;
  return resources;
}

ScreenwideOverlayUniforms screenwide_workspace_cursor_uniforms(
    ScreenwideCursorResources *resources,
    const ScreenwideWorkspaceLayer *layer) {
  ScreenwideOverlayUniforms cursor = {0};
  cursor.output_width = layer->canvas_width;
  cursor.output_height = layer->canvas_height;
  cursor.crop_x = layer->canvas.crop_x;
  cursor.crop_y = layer->canvas.crop_y;
  cursor.crop_width = layer->canvas.crop_width;
  cursor.crop_height = layer->canvas.crop_height;
  cursor.crop_radius = layer->canvas.radius;
  cursor.clip_at_video_edge = layer->cursor.clip_at_video_edge;
  cursor.cursor = layer->cursor;
  if (layer->cursor.visible != 0 && layer->cursor.style < resources.count)
    cursor.artwork = ((const ScreenwideCursorArtworkUniforms *)
        resources.uniforms.bytes)[layer->cursor.style];
  return cursor;
}

ScreenwideOverlayUniforms screenwide_canvas_cursor_uniforms(
    ScreenwideCursorResources *resources, const ScreenwideGpuCursor *cursor,
    const ScreenwideCanvas *canvas, uint32_t output_width,
    uint32_t output_height) {
  ScreenwideOverlayUniforms uniforms = {0};
  uniforms.output_width = output_width;
  uniforms.output_height = output_height;
  uniforms.crop_x = canvas->crop_x;
  uniforms.crop_y = canvas->crop_y;
  uniforms.crop_width = canvas->crop_width;
  uniforms.crop_height = canvas->crop_height;
  uniforms.crop_radius = canvas->radius;
  uniforms.clip_at_video_edge = cursor->clip_at_video_edge;
  uniforms.cursor = *cursor;
  if (cursor->visible != 0 && cursor->style < resources.count)
    uniforms.artwork = ((const ScreenwideCursorArtworkUniforms *)
        resources.uniforms.bytes)[cursor->style];
  return uniforms;
}
