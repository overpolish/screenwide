// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_export_private.h"
#import "gpu_compositor_macos_generators_layered.h"

int screenwide_gpu_composite_still(
    const uint8_t *source_rgba, uint32_t source_width, uint32_t source_height,
    const ScreenwideCanvas *canvas, uint32_t output_width,
    uint32_t output_height, double seconds,
    const ScreenwideGpuCursor *gpu_cursor,
    const ScreenwideCursorArtwork *cursor_artworks,
    uint32_t cursor_artwork_count, const uint8_t *camera_rgba,
    const ScreenwideStillOverlay *overlay,
    const ScreenwideKeyboardOverlay *keyboard,
    const ScreenwideAnnotations *annotations, uint8_t *output_rgba,
    char *error_text, size_t error_capacity) {
  @autoreleasepool {
    if (source_rgba == NULL || output_rgba == NULL || canvas == NULL ||
        gpu_cursor == NULL || source_width == 0 || source_height == 0 ||
        output_width == 0 || output_height == 0) {
      return fail(error_text, error_capacity,
                  @"The GPU still compositor received invalid pixels");
    }
    static id<MTLDevice> device;
    static id<MTLComputePipelineState> pipeline;
    static id<MTLCommandQueue> queue;
    static NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *>
        *keyboard_cache;
    static NSString *initialization_error;
    static dispatch_once_t once;
    dispatch_once(&once, ^{
      NSError *error = nil;
      device = MTLCreateSystemDefaultDevice();
      id<MTLLibrary> library =
          [device newLibraryWithSource:screenwide_gpu_canvas_shader_source()
                               options:nil
                                 error:&error];
      id<MTLFunction> function =
          [library newFunctionWithName:@"compose_canvas_rgba"];
      pipeline = [device newComputePipelineStateWithFunction:function
                                                       error:&error];
      queue = [device newCommandQueue];
      keyboard_cache = [NSMutableDictionary dictionary];
      initialization_error = error.localizedDescription;
    });
    if (device == nil || pipeline == nil || queue == nil) {
      return fail(error_text, error_capacity,
                  initialization_error
                      ?: @"The Metal still compositor could not be created");
    }
    NSUInteger source_length = (NSUInteger)source_width * source_height * 4;
    NSUInteger output_length = (NSUInteger)output_width * output_height * 4;
    id<MTLBuffer> source =
        [device newBufferWithBytes:source_rgba
                            length:source_length
                           options:MTLResourceStorageModeShared];
    id<MTLBuffer> output =
        [device newBufferWithLength:output_length
                            options:MTLResourceStorageModeShared];
    ScreenwideStillOverlay empty_overlay = {0};
    if (overlay == NULL)
      overlay = &empty_overlay;
    ScreenwideKeyboardOverlay empty_keyboard = {0};
    if (keyboard == NULL)
      keyboard = &empty_keyboard;
    // Metal rejects a nil buffer even where the kernel skips every read, so
    // an empty list still binds the zeroed array with a count of zero.
    ScreenwideAnnotations empty_annotations = {0};
    if (annotations == NULL)
      annotations = &empty_annotations;
    ScreenwideCursorResources *cursor_resources = screenwide_cursor_resources(
        device, cursor_artworks, cursor_artwork_count);
    if (cursor_resources == nil ||
        (gpu_cursor->visible != 0 &&
         gpu_cursor->style >= cursor_resources.count)) {
      return fail(error_text, error_capacity,
                  @"The GPU still compositor could not load cursor artwork");
    }
    ScreenwideOverlayUniforms cursor_uniforms_value =
        screenwide_canvas_cursor_uniforms(cursor_resources, gpu_cursor, canvas,
                                          output_width, output_height);
    id<MTLBuffer> cursor_uniforms =
        [device newBufferWithBytes:&cursor_uniforms_value
                            length:sizeof(cursor_uniforms_value)
                           options:MTLResourceStorageModeShared];
    id<MTLBuffer> camera =
        camera_rgba == NULL
            ? [device newBufferWithLength:4
                                  options:MTLResourceStorageModeShared]
            : [device
                  newBufferWithBytes:camera_rgba
                              length:(NSUInteger)overlay->camera_source_width *
                                     overlay->camera_source_height * 4
                             options:MTLResourceStorageModeShared];
    id<MTLBuffer> overlay_uniforms =
        [device newBufferWithBytes:overlay
                            length:sizeof(*overlay)
                           options:MTLResourceStorageModeShared];
    uint32_t source_dimensions[2] = {source_width, source_height};
    float time = (float)seconds;
    id<MTLCommandBuffer> commands = [queue commandBuffer];
    id<MTLComputeCommandEncoder> encoder = [commands computeCommandEncoder];
    [encoder setComputePipelineState:pipeline];
    [encoder setBuffer:source offset:0 atIndex:0];
    [encoder setBuffer:output offset:0 atIndex:1];
    screenwide_gpu_bind_canvas(encoder, device, canvas, 2, 1);
    [encoder setBytes:source_dimensions
               length:sizeof(source_dimensions)
              atIndex:3];
    [encoder setBytes:&time length:sizeof(time) atIndex:4];
    [encoder setBuffer:cursor_uniforms offset:0 atIndex:5];
    [encoder setBuffer:camera offset:0 atIndex:6];
    [encoder setBuffer:overlay_uniforms offset:0 atIndex:7];
    screenwide_bind_keyboard(encoder, device, keyboard_cache, *keyboard,
                             output_height);
    screenwide_bind_annotations(encoder, annotations);
    [encoder setTexture:cursor_resources.texture atIndex:0];
    MTLSize grid = MTLSizeMake(output_width, output_height, 1);
    NSUInteger width = MIN(pipeline.threadExecutionWidth, output_width);
    NSUInteger height =
        MIN(MAX((NSUInteger)1, pipeline.maxTotalThreadsPerThreadgroup /
                                   MAX(width, (NSUInteger)1)),
            output_height);
    [encoder dispatchThreads:grid
        threadsPerThreadgroup:MTLSizeMake(width, height, 1)];
    [encoder endEncoding];
    [commands commit];
    [commands waitUntilCompleted];
    if (commands.status == MTLCommandBufferStatusError) {
      return fail(error_text, error_capacity,
                  commands.error.localizedDescription
                      ?: @"The Metal still compositor failed");
    }
    memcpy(output_rgba, output.contents, output_length);
    return 1;
  }
}

int screenwide_gpu_alpha_composite(const uint8_t *base_rgba,
                                   const uint8_t *overlay_rgba, uint32_t width,
                                   uint32_t height, uint8_t *output_rgba,
                                   char *error_text, size_t error_capacity) {
  @autoreleasepool {
    if (base_rgba == NULL || overlay_rgba == NULL || output_rgba == NULL ||
        width == 0 || height == 0) {
      return fail(error_text, error_capacity,
                  @"The GPU layer compositor received invalid pixels");
    }
    static id<MTLDevice> device;
    static id<MTLComputePipelineState> pipeline;
    static id<MTLCommandQueue> queue;
    static NSString *initialization_error;
    static dispatch_once_t once;
    dispatch_once(&once, ^{
      NSError *error = nil;
      device = MTLCreateSystemDefaultDevice();
      id<MTLLibrary> library =
          [device newLibraryWithSource:screenwide_gpu_canvas_shader_source()
                               options:nil
                                 error:&error];
      id<MTLFunction> function =
          [library newFunctionWithName:@"alpha_composite_rgba"];
      pipeline = [device newComputePipelineStateWithFunction:function
                                                       error:&error];
      queue = [device newCommandQueue];
      initialization_error = error.localizedDescription;
    });
    if (device == nil || pipeline == nil || queue == nil) {
      return fail(error_text, error_capacity,
                  initialization_error
                      ?: @"The Metal layer compositor could not be created");
    }
    NSUInteger pixel_count = (NSUInteger)width * height;
    NSUInteger byte_length = pixel_count * 4;
    id<MTLBuffer> base =
        [device newBufferWithBytes:base_rgba
                            length:byte_length
                           options:MTLResourceStorageModeShared];
    id<MTLBuffer> overlay =
        [device newBufferWithBytes:overlay_rgba
                            length:byte_length
                           options:MTLResourceStorageModeShared];
    id<MTLBuffer> output =
        [device newBufferWithLength:byte_length
                            options:MTLResourceStorageModeShared];
    id<MTLCommandBuffer> commands = [queue commandBuffer];
    id<MTLComputeCommandEncoder> encoder = [commands computeCommandEncoder];
    [encoder setComputePipelineState:pipeline];
    [encoder setBuffer:base offset:0 atIndex:0];
    [encoder setBuffer:overlay offset:0 atIndex:1];
    [encoder setBuffer:output offset:0 atIndex:2];
    NSUInteger group_width = MIN(pipeline.threadExecutionWidth, pixel_count);
    [encoder dispatchThreads:MTLSizeMake(pixel_count, 1, 1)
        threadsPerThreadgroup:MTLSizeMake(MAX(group_width, (NSUInteger)1), 1,
                                          1)];
    [encoder endEncoding];
    [commands commit];
    [commands waitUntilCompleted];
    if (commands.status == MTLCommandBufferStatusError) {
      return fail(error_text, error_capacity,
                  commands.error.localizedDescription
                      ?: @"The Metal layer compositor failed");
    }
    memcpy(output_rgba, output.contents, byte_length);
    return 1;
  }
}
