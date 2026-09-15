// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_export_private.h"
#import "gpu_compositor_macos_export_annotations.h"

static id<MTLTexture> texture(CVMetalTextureCacheRef cache,
                              CVPixelBufferRef pixels, MTLPixelFormat format,
                              size_t width, size_t height, size_t plane,
                              CVMetalTextureRef *reference) {
  CVReturn result = CVMetalTextureCacheCreateTextureFromImage(
      kCFAllocatorDefault, cache, pixels, NULL, format, width, height, plane,
      reference);
  if (result != kCVReturnSuccess || *reference == NULL)
    return nil;
  return CVMetalTextureGetTexture(*reference);
}

@implementation ScreenwideVideoExport (Encoding)
- (ScreenwideInflightFrame *)encodeScreen:(CMSampleBufferRef)screen_sample
                                   camera:(CMSampleBufferRef)camera_sample
                              destination:(CVPixelBufferRef)destination
                                   cursor:(const ScreenwideGpuCursor *)cursor
                                 keyboard:
                                     (const ScreenwideKeyboardOverlay *)keyboard
                               sourceTime:(CMTime)pts
                               outputTime:(CMTime)output_pts {
  CVPixelBufferRef source = CMSampleBufferGetImageBuffer(screen_sample);
  size_t source_y_width = CVPixelBufferGetWidthOfPlane(source, 0);
  size_t source_y_height = CVPixelBufferGetHeightOfPlane(source, 0);
  size_t source_uv_width = CVPixelBufferGetWidthOfPlane(source, 1);
  size_t source_uv_height = CVPixelBufferGetHeightOfPlane(source, 1);
  size_t y_width = output_width;
  size_t y_height = output_height;
  size_t uv_width = (output_width + 1) / 2;
  size_t uv_height = (output_height + 1) / 2;
  CVMetalTextureRef source_y_ref = NULL, source_uv_ref = NULL;
  CVMetalTextureRef destination_y_ref = NULL, destination_uv_ref = NULL;
  id<MTLTexture> source_y =
      texture(texture_cache, source, MTLPixelFormatR8Unorm, source_y_width,
              source_y_height, 0, &source_y_ref);
  id<MTLTexture> source_uv =
      texture(texture_cache, source, MTLPixelFormatRG8Unorm, source_uv_width,
              source_uv_height, 1, &source_uv_ref);
  id<MTLTexture> destination_y =
      texture(texture_cache, destination, MTLPixelFormatR8Unorm, y_width,
              y_height, 0, &destination_y_ref);
  id<MTLTexture> destination_uv =
      texture(texture_cache, destination, MTLPixelFormatRG8Unorm, uv_width,
              uv_height, 1, &destination_uv_ref);
  id<MTLCommandBuffer> command = [queue commandBuffer];
  float seconds = (float)CMTimeGetSeconds(pts);
  MTLSize canvas_group = MTLSizeMake(16, 16, 1);
  id<MTLComputeCommandEncoder> canvas_compute = [command computeCommandEncoder];
  [canvas_compute setComputePipelineState:canvas_luma_pipeline];
  [canvas_compute setTexture:source_y atIndex:0];
  [canvas_compute setTexture:source_uv atIndex:1];
  [canvas_compute setTexture:destination_y atIndex:2];
  screenwide_gpu_bind_canvas(canvas_compute, device, canvas, 0, 3);
  [canvas_compute setBytes:&seconds length:sizeof(seconds) atIndex:1];
  [canvas_compute dispatchThreads:MTLSizeMake(y_width, y_height, 1)
            threadsPerThreadgroup:canvas_group];
  [canvas_compute endEncoding];
  canvas_compute = [command computeCommandEncoder];
  [canvas_compute setComputePipelineState:canvas_chroma_pipeline];
  [canvas_compute setTexture:source_y atIndex:0];
  [canvas_compute setTexture:source_uv atIndex:1];
  [canvas_compute setTexture:destination_uv atIndex:2];
  screenwide_gpu_bind_canvas(canvas_compute, device, canvas, 0, 3);
  [canvas_compute setBytes:&seconds length:sizeof(seconds) atIndex:1];
  [canvas_compute dispatchThreads:MTLSizeMake(uv_width, uv_height, 1)
            threadsPerThreadgroup:canvas_group];
  [canvas_compute endEncoding];
  screenwide_export_encode_cursor_overlay(
      command, luma_pipeline, chroma_pipeline, destination_y, destination_uv,
      cursor_artwork, cursor, artworks, artwork_count, canvas, output_width,
      output_height);
  uint64_t annotation_ms = (uint64_t)llround(CMTimeGetSeconds(pts) * 1000.0);
  screenwide_export_annotations(self, command, destination_y, destination_uv,
      (uint32_t)source_y_width, (uint32_t)source_y_height, annotation_ms, 0);
  CVMetalTextureRef camera_ref = NULL;
  if (camera_sample != NULL && camera_overlay != NULL) {
    CVPixelBufferRef camera_pixels =
        CMSampleBufferGetImageBuffer(camera_sample);
    size_t camera_width = CVPixelBufferGetWidth(camera_pixels);
    size_t camera_height = CVPixelBufferGetHeight(camera_pixels);
    id<MTLTexture> camera_texture =
        texture(texture_cache, camera_pixels, MTLPixelFormatBGRA8Unorm,
                camera_width, camera_height, 0, &camera_ref);
    ScreenwideCameraUniforms camera_uniforms = {
        camera_overlay->crop_x,      camera_overlay->crop_y,
        camera_overlay->crop_width,  camera_overlay->crop_height,
        camera_overlay->frame_x,     camera_overlay->frame_y,
        camera_overlay->frame_width, camera_overlay->frame_height,
        camera_overlay->radius,      (uint32_t)camera_width,
        (uint32_t)camera_height,     camera_overlay->drop_shadow,
    };
    MTLSize camera_group = MTLSizeMake(16, 16, 1);
    id<MTLComputeCommandEncoder> camera_compute =
        [command computeCommandEncoder];
    [camera_compute setComputePipelineState:camera_luma_pipeline];
    [camera_compute setTexture:camera_texture atIndex:0];
    [camera_compute setTexture:destination_y atIndex:1];
    [camera_compute setBytes:&camera_uniforms
                      length:sizeof(camera_uniforms)
                     atIndex:0];
    [camera_compute dispatchThreads:MTLSizeMake(y_width, y_height, 1)
              threadsPerThreadgroup:camera_group];
    [camera_compute endEncoding];
    camera_compute = [command computeCommandEncoder];
    [camera_compute setComputePipelineState:camera_chroma_pipeline];
    [camera_compute setTexture:camera_texture atIndex:0];
    [camera_compute setTexture:destination_uv atIndex:1];
    [camera_compute setBytes:&camera_uniforms
                      length:sizeof(camera_uniforms)
                     atIndex:0];
    [camera_compute dispatchThreads:MTLSizeMake(uv_width, uv_height, 1)
              threadsPerThreadgroup:camera_group];
    [camera_compute endEncoding];
  }
  if (camera_sample != NULL && camera_overlay != NULL &&
      camera_overlay->camera_on_top == 0) {
    MTLSize screen_group = MTLSizeMake(16, 16, 1);
    id<MTLComputeCommandEncoder> screen_compute =
        [command computeCommandEncoder];
    [screen_compute setComputePipelineState:screen_luma_pipeline];
    [screen_compute setTexture:source_y atIndex:0];
    [screen_compute setTexture:source_uv atIndex:1];
    [screen_compute setTexture:destination_y atIndex:2];
    [screen_compute setBytes:canvas length:sizeof(*canvas) atIndex:0];
    [screen_compute dispatchThreads:MTLSizeMake(y_width, y_height, 1)
              threadsPerThreadgroup:screen_group];
    [screen_compute endEncoding];
    screen_compute = [command computeCommandEncoder];
    [screen_compute setComputePipelineState:screen_chroma_pipeline];
    [screen_compute setTexture:source_y atIndex:0];
    [screen_compute setTexture:source_uv atIndex:1];
    [screen_compute setTexture:destination_uv atIndex:2];
    [screen_compute setBytes:canvas length:sizeof(*canvas) atIndex:0];
    [screen_compute dispatchThreads:MTLSizeMake(uv_width, uv_height, 1)
              threadsPerThreadgroup:screen_group];
    [screen_compute endEncoding];

    // Cursor belongs to the screen layer. Reapply it after the screen
    // when the camera has been sent behind that layer.
    screenwide_export_encode_cursor_overlay(
        command, luma_pipeline, chroma_pipeline, destination_y, destination_uv,
        cursor_artwork, cursor, artworks, artwork_count, canvas, output_width,
        output_height);
  }
  if (camera_sample != NULL && camera_overlay != NULL && camera_overlay->camera_on_top == 0)
    screenwide_export_annotations(self, command, destination_y, destination_uv,
        (uint32_t)source_y_width, (uint32_t)source_y_height, annotation_ms, 0);
  screenwide_export_annotations(self, command, destination_y, destination_uv,
      (uint32_t)source_y_width, (uint32_t)source_y_height, annotation_ms, 1);
  screenwide_encode_keyboard_overlay(command, device, keyboard_luma_pipeline,
                                     keyboard_chroma_pipeline, destination_y,
                                     destination_uv, keyboard_cache, keyboard,
                                     output_width, output_height);
  // Commit without waiting: the GPU works on this frame while the loop
  // decodes and composites the next ones. The frame owns every resource
  // the command buffer reads or writes until it completes.
  [command commit];
  ScreenwideInflightFrame *frame = [ScreenwideInflightFrame new];
  frame.command = command;
  frame.presentation = output_pts;
  frame.destination = destination;
  frame.sourceLuma = source_y_ref;
  frame.sourceChroma = source_uv_ref;
  frame.destinationLuma = destination_y_ref;
  frame.destinationChroma = destination_uv_ref;
  frame.cameraTexture = camera_ref;
  frame.screenSample = (CMSampleBufferRef)CFRetain(screen_sample);
  frame.cameraSample =
      camera_sample == NULL ? NULL : (CMSampleBufferRef)CFRetain(camera_sample);
  return frame;
}
@end
