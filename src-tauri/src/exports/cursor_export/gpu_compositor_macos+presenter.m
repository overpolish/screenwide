// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <CoreVideo/CoreVideo.h>
#import <Metal/Metal.h>
#import <QuartzCore/CAMetalLayer.h>
#include <math.h>
#include <stdlib.h>

#import "gpu_compositor_macos.h"
#import "gpu_compositor_macos_cursor_resources.h"
#import "gpu_compositor_macos_keyboard.h"

extern __attribute__((visibility("hidden"))) NSString *const shader_source;

/// Native still/workspace extension point for future screenshot annotation passes.
@interface ScreenwideStillPresenter : NSObject
@property(nonatomic, strong) id<MTLDevice> device;
@property(nonatomic, strong) id<MTLCommandQueue> queue;
@property(nonatomic, strong) id<MTLComputePipelineState> pipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> unpackPipeline;
@property(nonatomic, strong) id<MTLBuffer> source;
@property(nonatomic, strong) id<MTLBuffer> camera;
@property(nonatomic) CVMetalTextureCacheRef textureCache;
@property(nonatomic) uint64_t sourceToken;
@property(nonatomic) uint32_t sourceWidth;
@property(nonatomic) uint32_t sourceHeight;
@property(nonatomic) uint64_t cameraToken;
@property(nonatomic) uint32_t cameraWidth;
@property(nonatomic) uint32_t cameraHeight;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, id<MTLBuffer>> *workspaceSources;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, id<MTLBuffer>> *workspaceCameraSources;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, NSValue *> *workspaceSourceSizes;
@property(nonatomic, strong) NSMutableArray<NSValue *> *workspaceLayers; @property(nonatomic, strong) NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *keyboardArtworks;
@property(nonatomic, strong) NSArray<NSValue *> *workspaceResizeLayers;
@property(nonatomic) BOOL workspaceResizeApplied;
@property(nonatomic, strong) id<MTLComputePipelineState> workspaceClearPipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> workspaceLayerPipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> workspaceMagnifierPipeline;
@property(nonatomic, strong) ScreenwideCursorResources *cursorResources;
@end

@implementation ScreenwideStillPresenter
- (void)dealloc {
  if (_textureCache != NULL) CFRelease(_textureCache);
}
@end

void *screenwide_gpu_still_presenter_create(void) {
  @autoreleasepool {
    ScreenwideStillPresenter *presenter = [ScreenwideStillPresenter new];
    presenter.device = MTLCreateSystemDefaultDevice();
    NSError *error = nil;
    id<MTLLibrary> library =
        [presenter.device newLibraryWithSource:shader_source options:nil error:&error];
    presenter.pipeline = [presenter.device newComputePipelineStateWithFunction:
        [library newFunctionWithName:@"present_canvas_rgba"] error:&error];
    presenter.unpackPipeline = [presenter.device newComputePipelineStateWithFunction:
        [library newFunctionWithName:@"unpack_preview_bgra"] error:&error];
    presenter.workspaceClearPipeline = [presenter.device newComputePipelineStateWithFunction:
        [library newFunctionWithName:@"workspace_clear"] error:&error];
    presenter.workspaceLayerPipeline = [presenter.device newComputePipelineStateWithFunction:
        [library newFunctionWithName:@"workspace_layer"] error:&error];
    presenter.workspaceMagnifierPipeline = [presenter.device newComputePipelineStateWithFunction:
        [library newFunctionWithName:@"workspace_magnifier"] error:&error];
    presenter.queue = [presenter.device newCommandQueue];
    presenter.workspaceSources = [NSMutableDictionary dictionary];
    presenter.workspaceCameraSources = [NSMutableDictionary dictionary];
    presenter.workspaceSourceSizes = [NSMutableDictionary dictionary];
    presenter.workspaceLayers = [NSMutableArray array];
    presenter.keyboardArtworks = [NSMutableDictionary dictionary];
    const uint8_t transparentCursor[4] = {0, 0, 0, 0};
    const ScreenwideCursorArtwork emptyCursor = {transparentCursor, 1, 1, 1, 1,
                                                  0, 0, 0, 0, 0};
    if (!screenwide_gpu_still_presenter_set_cursor_artworks((__bridge void *)presenter, &emptyCursor, 1)) return NULL;
    CVMetalTextureCacheRef texture_cache = NULL;
    CVMetalTextureCacheCreate(kCFAllocatorDefault, NULL, presenter.device, NULL,
                              &texture_cache);
    presenter.textureCache = texture_cache;
    if (presenter.pipeline == nil || presenter.unpackPipeline == nil ||
        presenter.workspaceClearPipeline == nil || presenter.workspaceLayerPipeline == nil ||
        presenter.workspaceMagnifierPipeline == nil ||
        presenter.queue == nil || presenter.textureCache == NULL) return NULL;
    return (__bridge_retained void *)presenter;
  }
}
int screenwide_gpu_still_presenter_set_cursor_artworks(
    void *handle, const ScreenwideCursorArtwork *artworks,
    uint32_t artwork_count) {
  if (handle == NULL || artworks == NULL || artwork_count == 0) return 0;
  @autoreleasepool {
    ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
    if (presenter.cursorResources.count == artwork_count) return 1;
    presenter.cursorResources = screenwide_cursor_resources(presenter.device, artworks, artwork_count);
    return presenter.cursorResources != nil;
  }
}

static id<MTLTexture> preview_texture(ScreenwideStillPresenter *presenter,
                                      CVPixelBufferRef pixels,
                                      CVMetalTextureRef *reference) {
  size_t width = CVPixelBufferGetWidth(pixels);
  size_t height = CVPixelBufferGetHeight(pixels);
  CVReturn result = CVMetalTextureCacheCreateTextureFromImage(
      kCFAllocatorDefault, presenter.textureCache, pixels, NULL,
      MTLPixelFormatBGRA8Unorm, width, height, 0, reference);
  return result == kCVReturnSuccess && *reference != NULL
      ? CVMetalTextureGetTexture(*reference) : nil;
}

static id<MTLBuffer> unpack_pixels(ScreenwideStillPresenter *presenter,
                                   CVPixelBufferRef pixels,
                                   id<MTLCommandBuffer> command,
                                   CVMetalTextureRef *reference) {
  id<MTLTexture> source = preview_texture(presenter, pixels, reference);
  if (source == nil) return nil;
  NSUInteger length = source.width * source.height * 4;
  id<MTLBuffer> output = [presenter.device newBufferWithLength:length
      options:MTLResourceStorageModePrivate];
  id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
  [encoder setComputePipelineState:presenter.unpackPipeline];
  [encoder setTexture:source atIndex:0];
  [encoder setBuffer:output offset:0 atIndex:0];
  MTLSize grid = MTLSizeMake(source.width, source.height, 1);
  NSUInteger width = MIN(presenter.unpackPipeline.threadExecutionWidth, grid.width);
  NSUInteger height = MIN(MAX((NSUInteger)1,
      presenter.unpackPipeline.maxTotalThreadsPerThreadgroup / MAX(width, (NSUInteger)1)),
      grid.height);
  [encoder dispatchThreads:grid threadsPerThreadgroup:MTLSizeMake(width, height, 1)];
  [encoder endEncoding];
  return output;
}

int screenwide_gpu_still_presenter_present_pixels(
    void *handle, void *metal_layer, uint64_t source_token,
    void *source_pixels_handle, const ScreenwideCanvas *canvas, double seconds,
    const uint8_t *cursor_rgba, const uint8_t *camera_rgba,
    void *camera_pixels_handle,
    const ScreenwideStillOverlay *overlay,
    ScreenwidePresentBlock present) {
  if (handle == NULL || metal_layer == NULL || source_pixels_handle == NULL ||
      canvas == NULL || present == NULL) return 0;
  @autoreleasepool {
    ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
    CAMetalLayer *layer = (__bridge CAMetalLayer *)metal_layer;
    CVPixelBufferRef source_pixels = (CVPixelBufferRef)source_pixels_handle;
    uint32_t source_width = (uint32_t)CVPixelBufferGetWidth(source_pixels);
    uint32_t source_height = (uint32_t)CVPixelBufferGetHeight(source_pixels);
    id<CAMetalDrawable> drawable = [layer nextDrawable];
    if (drawable == nil) return 0;
    id<MTLCommandBuffer> command = [presenter.queue commandBuffer];
    CVMetalTextureRef source_reference = NULL;
    if (presenter.source == nil || presenter.sourceToken != source_token ||
        presenter.sourceWidth != source_width || presenter.sourceHeight != source_height) {
      presenter.source = unpack_pixels(presenter, source_pixels, command, &source_reference);
      if (presenter.source == nil) return 0;
      presenter.sourceToken = source_token;
      presenter.sourceWidth = source_width;
      presenter.sourceHeight = source_height;
    }
    ScreenwideStillOverlay empty_overlay = {0};
    if (overlay == NULL) overlay = &empty_overlay;
    id<MTLBuffer> uniforms = [presenter.device newBufferWithBytes:canvas
      length:sizeof(*canvas) options:MTLResourceStorageModeShared];
    id<MTLBuffer> cursor = cursor_rgba == NULL
      ? [presenter.device newBufferWithLength:4 options:MTLResourceStorageModeShared]
      : [presenter.device newBufferWithBytes:cursor_rgba
          length:(NSUInteger)overlay->cursor_source_width * overlay->cursor_source_height * 4
          options:MTLResourceStorageModeShared];
    CVMetalTextureRef camera_reference = NULL;
    if (camera_pixels_handle != NULL) {
      CVPixelBufferRef camera_pixels = (CVPixelBufferRef)camera_pixels_handle;
      uint32_t camera_width = (uint32_t)CVPixelBufferGetWidth(camera_pixels);
      uint32_t camera_height = (uint32_t)CVPixelBufferGetHeight(camera_pixels);
      if (presenter.camera == nil || presenter.cameraToken != source_token ||
          presenter.cameraWidth != camera_width || presenter.cameraHeight != camera_height) {
        presenter.camera = unpack_pixels(presenter, camera_pixels, command,
                                         &camera_reference);
        presenter.cameraToken = source_token;
        presenter.cameraWidth = camera_width;
        presenter.cameraHeight = camera_height;
      }
    } else {
      presenter.camera = nil;
    }
    id<MTLBuffer> camera = presenter.camera != nil
      ? presenter.camera
      : camera_rgba != NULL
        ? [presenter.device newBufferWithBytes:camera_rgba
            length:(NSUInteger)overlay->camera_source_width *
                   overlay->camera_source_height * 4
            options:MTLResourceStorageModeShared]
        : [presenter.device newBufferWithLength:4 options:MTLResourceStorageModeShared];
    if (camera == nil) return 0;
    id<MTLBuffer> overlay_uniforms = [presenter.device newBufferWithBytes:overlay
      length:sizeof(*overlay) options:MTLResourceStorageModeShared];
    uint32_t dimensions[2] = {source_width, source_height};
    float time = (float)seconds;
    id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
    [encoder setComputePipelineState:presenter.pipeline];
    [encoder setBuffer:presenter.source offset:0 atIndex:0];
    [encoder setTexture:drawable.texture atIndex:0];
    [encoder setBuffer:uniforms offset:0 atIndex:1];
    [encoder setBytes:dimensions length:sizeof(dimensions) atIndex:2];
    [encoder setBytes:&time length:sizeof(time) atIndex:3];
    [encoder setBuffer:cursor offset:0 atIndex:4];
    [encoder setBuffer:camera offset:0 atIndex:5];
    [encoder setBuffer:overlay_uniforms offset:0 atIndex:6];
    MTLSize grid = MTLSizeMake(drawable.texture.width, drawable.texture.height, 1);
    NSUInteger width = MIN(presenter.pipeline.threadExecutionWidth, grid.width);
    NSUInteger height = MIN(MAX((NSUInteger)1,
      presenter.pipeline.maxTotalThreadsPerThreadgroup / MAX(width, (NSUInteger)1)), grid.height);
    [encoder dispatchThreads:grid threadsPerThreadgroup:MTLSizeMake(width, height, 1)];
    [encoder endEncoding];
    [command addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
      if (source_reference != NULL) CFRelease(source_reference);
      if (camera_reference != NULL) CFRelease(camera_reference);
    }];
    present((__bridge void *)command, (__bridge void *)drawable);
    return 1;
  }
}

static id<MTLBuffer> workspace_source_buffer(
    ScreenwideStillPresenter *presenter, const ScreenwideWorkspaceLayer *layer) {
  if (layer->source_width == 0 || layer->source_height == 0) return nil;
  NSNumber *key = @(layer->source_token);
  NSValue *known_size = presenter.workspaceSourceSizes[key];
  uint32_t dimensions[2] = {layer->source_width, layer->source_height};
  if (known_size != nil) {
    uint32_t cached[2] = {0, 0};
    [known_size getValue:cached size:sizeof(cached)];
    if (cached[0] == dimensions[0] && cached[1] == dimensions[1])
      return presenter.workspaceSources[key];
  }
  if (layer->source_rgba == NULL && layer->source_pixels == NULL) return nil;
  NSUInteger length = (NSUInteger)layer->source_width * layer->source_height * 4;
  if (layer->source_kind != 0 && layer->source_pixels != NULL) return nil;
  if (layer->source_rgba == NULL) return nil;
  id<MTLBuffer> buffer = [presenter.device newBufferWithBytes:layer->source_rgba
      length:length options:MTLResourceStorageModeShared];
  if (buffer == nil) return nil;
  presenter.workspaceSources[key] = buffer;
  presenter.workspaceSourceSizes[key] = [NSValue valueWithBytes:dimensions
                                                         objCType:@encode(uint32_t[2])];
  return buffer;
}

static void workspace_dispatch(
    id<MTLComputeCommandEncoder> encoder, id<MTLComputePipelineState> pipeline,
    MTLSize grid) {
  NSUInteger width = MIN(pipeline.threadExecutionWidth, grid.width);
  NSUInteger height = MIN(MAX((NSUInteger)1,
      pipeline.maxTotalThreadsPerThreadgroup / MAX(width, (NSUInteger)1)),
      grid.height);
  [encoder dispatchThreads:grid threadsPerThreadgroup:MTLSizeMake(
      MAX(width, (NSUInteger)1), MAX(height, (NSUInteger)1), 1)];
}

static int presenter_present_workspace_layers(
    ScreenwideStillPresenter *presenter, CAMetalLayer *layer,
    const ScreenwideWorkspaceLayer *layers, uint32_t layer_count,
    ScreenwidePresentBlock present) {
  if (layer_count == 0 || layers == NULL || present == NULL) return 0;
  id<CAMetalDrawable> drawable = [layer nextDrawable];
  if (drawable == nil) return 0;
  id<MTLCommandBuffer> command = [presenter.queue commandBuffer];
  if (command == nil) return 0;
  MTLSize grid = MTLSizeMake(drawable.texture.width, drawable.texture.height, 1);
  id<MTLComputeCommandEncoder> clear = [command computeCommandEncoder];
  [clear setComputePipelineState:presenter.workspaceClearPipeline];
  [clear setTexture:drawable.texture atIndex:0];
  workspace_dispatch(clear, presenter.workspaceClearPipeline, grid);
  [clear endEncoding];
  NSMutableArray<NSValue *> *pixelReferences = [NSMutableArray array];
  for (uint32_t index = 0; index < layer_count; ++index) {
    const ScreenwideWorkspaceLayer *item = &layers[index];
    id<MTLBuffer> source = workspace_source_buffer(presenter, item);
    CVMetalTextureRef source_reference = NULL;
    if (source == nil && item->source_kind != 0 && item->source_pixels != NULL) {
      source = unpack_pixels(presenter, (CVPixelBufferRef)item->source_pixels,
                             command, &source_reference);
      if (source != nil) {
        presenter.workspaceSources[@(item->source_token)] = source;
        uint32_t dimensions[2] = {item->source_width, item->source_height};
        presenter.workspaceSourceSizes[@(item->source_token)] =
            [NSValue valueWithBytes:dimensions objCType:@encode(uint32_t[2])];
        if (source_reference != NULL)
          [pixelReferences addObject:[NSValue valueWithPointer:source_reference]];
      }
    }
    if (source == nil) return 0;
    if (item->placement.width == 0 || item->placement.height == 0) continue;
    id<MTLBuffer> uniforms = [presenter.device newBufferWithBytes:&item->canvas
        length:sizeof(item->canvas) options:MTLResourceStorageModeShared];
    if (uniforms == nil) return 0;
    NSUInteger camera_length = (NSUInteger)item->overlay.camera_source_width *
        item->overlay.camera_source_height * 4;
    NSNumber *token = @(item->source_token);
    id<MTLBuffer> camera = presenter.workspaceCameraSources[token];
    if (camera == nil)
      camera = item->camera_rgba != NULL && camera_length > 0
          ? [presenter.device newBufferWithBytes:item->camera_rgba length:camera_length
            options:MTLResourceStorageModeShared]
          : [presenter.device newBufferWithLength:4 options:MTLResourceStorageModeShared];
    CVMetalTextureRef camera_reference = NULL;
    if (presenter.workspaceCameraSources[token] == nil &&
        item->camera_rgba == NULL && item->camera_pixels != NULL) {
      camera = unpack_pixels(presenter, (CVPixelBufferRef)item->camera_pixels,
                             command, &camera_reference);
      if (camera == nil) return 0;
      if (camera_reference != NULL)
        [pixelReferences addObject:[NSValue valueWithPointer:camera_reference]];
    }
    id<MTLBuffer> overlay = [presenter.device newBufferWithBytes:&item->overlay length:sizeof(item->overlay) options:MTLResourceStorageModeShared];
    if (camera == nil || overlay == nil) return 0;
    presenter.workspaceCameraSources[token] = camera;
    ScreenwideOverlayUniforms cursorUniforms = screenwide_workspace_cursor_uniforms(presenter.cursorResources, item);
    uint32_t dimensions[2] = {item->source_width, item->source_height};
    uint32_t first = index == 0 ? 1 : 0;
    id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
    [encoder setComputePipelineState:presenter.workspaceLayerPipeline];
    [encoder setBuffer:source offset:0 atIndex:0];
    [encoder setTexture:drawable.texture atIndex:0];
    [encoder setBuffer:uniforms offset:0 atIndex:1];
    [encoder setBytes:dimensions length:sizeof(dimensions) atIndex:2];
    [encoder setBytes:&item->placement length:sizeof(item->placement) atIndex:3];
    [encoder setBytes:&first length:sizeof(first) atIndex:4];
    uint32_t logical[2] = {item->canvas_width, item->canvas_height};
    [encoder setBytes:logical length:sizeof(logical) atIndex:5];
    [encoder setBytes:&cursorUniforms length:sizeof(cursorUniforms) atIndex:6];
    [encoder setBuffer:camera offset:0 atIndex:7];
    [encoder setBuffer:overlay offset:0 atIndex:8];
    float seconds = (float)item->seconds;
    [encoder setBytes:&seconds length:sizeof(seconds) atIndex:9];
    screenwide_bind_keyboard(encoder, presenter.device, presenter.keyboardArtworks, item->keyboard, item->canvas_height);
    [encoder setTexture:presenter.cursorResources.texture atIndex:1];
    workspace_dispatch(encoder, presenter.workspaceLayerPipeline, grid);
    [encoder endEncoding];
  }
  [command addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
    for (NSValue *value in pixelReferences) {
      CVMetalTextureRef reference = [value pointerValue];
      if (reference != NULL) CFRelease(reference);
    }
  }];
  present((__bridge void *)command, (__bridge void *)drawable);
  return 1;
}

int screenwide_gpu_still_presenter_set_workspace(
    void *handle, const ScreenwideWorkspaceLayer *layers,
    uint32_t layer_count) {
  if (handle == NULL || layers == NULL || layer_count == 0) return 0;
  @autoreleasepool {
    ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
    // The retained resize state is authoritative only once a Frame/auto-fit
    // update has actually rewritten the layers: then a late asynchronous
    // preview frame would clobber it with geometry from an earlier pointer
    // sample. Before that (a plain Move that merely opened the session so
    // Option can auto-fit later) the decoder's recomposed frames are the only
    // source of the moved pixels and must be accepted, otherwise the clip only
    // moves on mouse-up.
    if (presenter.workspaceResizeLayers.count > 0 &&
        presenter.workspaceResizeApplied) {
      return 1;
    }
    NSMutableArray<NSValue *> *retained = [NSMutableArray arrayWithCapacity:layer_count];
    NSMutableSet<NSNumber *> *activeTokens = [NSMutableSet setWithCapacity:layer_count];
    id<MTLCommandBuffer> uploadCommand = nil;
    NSMutableArray<NSValue *> *pixelReferences = [NSMutableArray array];
    for (uint32_t index = 0; index < layer_count; ++index) {
      id<MTLBuffer> source = workspace_source_buffer(presenter, &layers[index]);
      if (source == nil && layers[index].source_kind != 0 &&
          layers[index].source_pixels != NULL) {
        if (uploadCommand == nil) uploadCommand = [presenter.queue commandBuffer];
        CVMetalTextureRef reference = NULL;
        source = unpack_pixels(
            presenter, (CVPixelBufferRef)layers[index].source_pixels,
            uploadCommand, &reference);
        if (source != nil) {
          presenter.workspaceSources[@(layers[index].source_token)] = source;
          uint32_t dimensions[2] = {layers[index].source_width,
                                    layers[index].source_height};
          presenter.workspaceSourceSizes[@(layers[index].source_token)] =
              [NSValue valueWithBytes:dimensions
                              objCType:@encode(uint32_t[2])];
          if (reference != NULL)
            [pixelReferences addObject:[NSValue valueWithPointer:reference]];
        }
      }
      if (source == nil) return 0;
      NSNumber *token = @(layers[index].source_token);
      NSUInteger cameraLength = (NSUInteger)layers[index].overlay.camera_source_width *
          layers[index].overlay.camera_source_height * 4;
      if (layers[index].camera_rgba != NULL && cameraLength > 0)
        presenter.workspaceCameraSources[token] =
            [presenter.device newBufferWithBytes:layers[index].camera_rgba
                                          length:cameraLength
                                         options:MTLResourceStorageModeShared];
      else if (layers[index].camera_pixels != NULL) {
        if (uploadCommand == nil) uploadCommand = [presenter.queue commandBuffer];
        CVMetalTextureRef reference = NULL;
        id<MTLBuffer> camera = unpack_pixels(
            presenter, (CVPixelBufferRef)layers[index].camera_pixels,
            uploadCommand, &reference);
        if (camera == nil) return 0;
        presenter.workspaceCameraSources[token] = camera;
        if (reference != NULL)
          [pixelReferences addObject:[NSValue valueWithPointer:reference]];
      } else
        [presenter.workspaceCameraSources removeObjectForKey:token];
      [retained addObject:[NSValue valueWithBytes:&layers[index]
                                          objCType:@encode(ScreenwideWorkspaceLayer)]];
      [activeTokens addObject:token];
    }
    if (uploadCommand != nil) {
      [uploadCommand addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
        for (NSValue *value in pixelReferences) {
          CVMetalTextureRef reference = [value pointerValue];
          if (reference != NULL) CFRelease(reference);
        }
      }];
      [uploadCommand commit];
    }
    for (NSNumber *key in [presenter.workspaceSources.allKeys copy])
      if (![activeTokens containsObject:key]) {
        [presenter.workspaceSources removeObjectForKey:key];
        [presenter.workspaceSourceSizes removeObjectForKey:key];
        [presenter.workspaceCameraSources removeObjectForKey:key];
      }
    presenter.workspaceLayers = retained;
    return 1;
  }
}

int screenwide_gpu_still_presenter_workspace_source_size(
    void *handle, uint32_t pane_index, uint32_t *width, uint32_t *height) {
  if (handle == NULL || width == NULL || height == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  for (NSValue *value in presenter.workspaceLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index != pane_index) continue;
    if (presenter.workspaceSources[@(layer.source_token)] == nil) return 0;
    *width = layer.source_width;
    *height = layer.source_height;
    return layer.source_width > 0 && layer.source_height > 0;
  }
  return 0;
}

int screenwide_gpu_still_presenter_workspace_canvas_size(
    void *handle, uint32_t pane_index, uint32_t *width, uint32_t *height) {
  if (handle == NULL || width == NULL || height == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  for (NSValue *value in presenter.workspaceLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index != pane_index) continue;
    *width = layer.canvas_width;
    *height = layer.canvas_height;
    return *width > 0 && *height > 0;
  }
  return 0;
}

int screenwide_gpu_still_presenter_workspace_camera_source_size(
    void *handle, uint32_t pane_index, uint32_t *width, uint32_t *height) {
  if (handle == NULL || width == NULL || height == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  for (NSValue *value in presenter.workspaceLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index != pane_index) continue;
    if (presenter.workspaceCameraSources[@(layer.source_token)] == nil) return 0;
    *width = layer.overlay.camera_source_width;
    *height = layer.overlay.camera_source_height;
    return *width > 0 && *height > 0;
  }
  return 0;
}

int screenwide_gpu_still_presenter_update_workspace_canvas(
    void *handle, uint32_t pane_index, uint32_t canvas_width,
    uint32_t canvas_height, const ScreenwideCanvas *canvas) {
  if (handle == NULL || canvas == NULL || canvas_width == 0 || canvas_height == 0)
    return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  NSMutableArray<NSValue *> *updated = [presenter.workspaceLayers mutableCopy];
  for (NSUInteger index = 0; index < updated.count; ++index) {
    ScreenwideWorkspaceLayer layer;
    [updated[index] getValue:&layer size:sizeof(layer)];
    if (layer.pane_index != pane_index) continue;
    if (presenter.workspaceSources[@(layer.source_token)] == nil) return 0;
    layer.canvas_width = canvas_width;
    layer.canvas_height = canvas_height;
    layer.canvas = *canvas;
    updated[index] = [NSValue valueWithBytes:&layer
                                     objCType:@encode(ScreenwideWorkspaceLayer)];
    presenter.workspaceLayers = updated;
    return 1;
  }
  return 0;
}

int screenwide_gpu_still_presenter_update_workspace_camera_overlay(
    void *handle, uint32_t pane_index, const ScreenwideStillOverlay *overlay) {
  if (handle == NULL || overlay == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  NSMutableArray<NSValue *> *updated = [presenter.workspaceLayers mutableCopy];
  for (NSUInteger index = 0; index < updated.count; ++index) {
    ScreenwideWorkspaceLayer layer;
    [updated[index] getValue:&layer size:sizeof(layer)];
    if (layer.pane_index != pane_index) continue;
    if (presenter.workspaceCameraSources[@(layer.source_token)] == nil) return 0;
    layer.overlay.camera_crop_x = overlay->camera_crop_x;
    layer.overlay.camera_crop_y = overlay->camera_crop_y;
    layer.overlay.camera_crop_width = overlay->camera_crop_width;
    layer.overlay.camera_crop_height = overlay->camera_crop_height;
    layer.overlay.camera_frame_x = overlay->camera_frame_x;
    layer.overlay.camera_frame_y = overlay->camera_frame_y;
    layer.overlay.camera_frame_width = overlay->camera_frame_width;
    layer.overlay.camera_frame_height = overlay->camera_frame_height;
    layer.overlay.camera_radius = overlay->camera_radius;
    layer.overlay.camera_source_width = overlay->camera_source_width;
    layer.overlay.camera_source_height = overlay->camera_source_height;
    layer.overlay.camera_drop_shadow = overlay->camera_drop_shadow;
    layer.overlay.camera_on_top = overlay->camera_on_top;
    updated[index] = [NSValue valueWithBytes:&layer
                                     objCType:@encode(ScreenwideWorkspaceLayer)];
    presenter.workspaceLayers = updated;
    return 1;
  }
  return 0;
}

/// Moves everything attached to a layer's clip by (dx, dy) canvas pixels when
/// a Frame gesture moves the canvas origin: the crop and image placement, the
/// visible source region the shader gates clip coverage on, and both cursor
/// representations. The workspace kernel draws `cursor`, not
/// `overlay.cursor_*`, so the recorded pointer must travel with the clip too.
static void shift_layer_content(ScreenwideWorkspaceLayer *layer, double dx,
                                double dy) {
  layer->canvas.crop_x = (int32_t)llround(layer->canvas.crop_x + dx);
  layer->canvas.crop_y = (int32_t)llround(layer->canvas.crop_y + dy);
  layer->canvas.image_x += (float)dx;
  layer->canvas.image_y += (float)dy;
  layer->canvas.source_crop_x += (int32_t)llround(dx);
  layer->canvas.source_crop_y += (int32_t)llround(dy);
  if (layer->overlay.cursor_width > 0) {
    layer->overlay.cursor_x += (int32_t)llround(dx);
    layer->overlay.cursor_y += (int32_t)llround(dy);
  }
  layer->cursor.x += (float)dx;
  layer->cursor.y += (float)dy;
}

int screenwide_gpu_still_presenter_begin_workspace_resize(void *handle) {
  if (handle == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  presenter.workspaceResizeLayers = [presenter.workspaceLayers copy];
  presenter.workspaceResizeApplied = NO;
  return presenter.workspaceResizeLayers.count > 0;
}

static int update_workspace_resize(
    void *handle, uint32_t selected_layer, double move_x_ratio,
    double move_y_ratio, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  if (handle == NULL || !isfinite(origin_x_ratio) ||
      !isfinite(origin_y_ratio) || !isfinite(width_ratio) ||
      !isfinite(height_ratio) || width_ratio <= 0.0 ||
      height_ratio <= 0.0) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  if (presenter.workspaceResizeLayers.count == 0) return 0;
  NSMutableArray<NSValue *> *resized =
      [NSMutableArray arrayWithCapacity:presenter.workspaceResizeLayers.count];
  NSUInteger index = 0;
  for (NSValue *value in presenter.workspaceResizeLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    double old_width = MAX(layer.canvas_width, 1u);
    double old_height = MAX(layer.canvas_height, 1u);
    uint32_t next_width = (uint32_t)MAX(llround(old_width * width_ratio), 1);
    uint32_t next_height = (uint32_t)MAX(llround(old_height * height_ratio), 1);
    double origin_x = old_width * origin_x_ratio;
    double origin_y = old_height * origin_y_ratio;
    double move_x = index == selected_layer ? old_width * move_x_ratio : 0.0;
    double move_y = index == selected_layer ? old_height * move_y_ratio : 0.0;
    double old_shortest = MAX(MIN(old_width, old_height), 1.0);
    double next_shortest = MIN(next_width, next_height);
    layer.canvas_width = next_width;
    layer.canvas_height = next_height;
    shift_layer_content(&layer, move_x - origin_x, move_y - origin_y);
    layer.canvas.background_radius = (uint32_t)MAX(
        llround(layer.canvas.background_radius * next_shortest / old_shortest), 0);
    [resized addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
    index += 1;
  }
  presenter.workspaceResizeApplied = YES;
  presenter.workspaceLayers = resized;
  return 1;
}

int screenwide_gpu_still_presenter_update_workspace_resize(
    void *handle, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  return update_workspace_resize(
      handle, UINT32_MAX, 0.0, 0.0, origin_x_ratio, origin_y_ratio,
      width_ratio, height_ratio);
}

int screenwide_gpu_still_presenter_update_workspace_auto_fit_move(
    void *handle, uint32_t selected_layer, double move_x_ratio,
    double move_y_ratio, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  return update_workspace_resize(
      handle, selected_layer, move_x_ratio, move_y_ratio,
      origin_x_ratio, origin_y_ratio, width_ratio, height_ratio);
}

int screenwide_gpu_still_presenter_update_recording_auto_fit_move(
    void *handle, uint32_t selected_pane, double move_x_ratio,
    double move_y_ratio, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  if (handle == NULL || !isfinite(origin_x_ratio) ||
      !isfinite(origin_y_ratio) || !isfinite(width_ratio) ||
      !isfinite(height_ratio) || width_ratio <= 0.0 || height_ratio <= 0.0)
    return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  if (presenter.workspaceResizeLayers.count == 0) return 0;
  BOOL bakedCamera = selected_pane == 1 && presenter.workspaceResizeLayers.count == 1;
  NSMutableArray<NSValue *> *resized =
      [NSMutableArray arrayWithCapacity:presenter.workspaceResizeLayers.count];
  BOOL found = NO;
  for (NSValue *value in presenter.workspaceResizeLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == selected_pane || bakedCamera) {
      found = YES;
      double oldWidth = MAX(layer.canvas_width, 1u);
      double oldHeight = MAX(layer.canvas_height, 1u);
      double originX = oldWidth * origin_x_ratio;
      double originY = oldHeight * origin_y_ratio;
      double moveX = oldWidth * move_x_ratio;
      double moveY = oldHeight * move_y_ratio;
      layer.canvas_width = (uint32_t)MAX(llround(oldWidth * width_ratio), 1);
      layer.canvas_height = (uint32_t)MAX(llround(oldHeight * height_ratio), 1);
      if (bakedCamera) {
        // The selected camera moves independently, while the screen content
        // (and the cursor attached to it) only follows the canvas origin.
        shift_layer_content(&layer, -originX, -originY);
        layer.overlay.camera_frame_x += (int32_t)llround(moveX - originX);
        layer.overlay.camera_frame_y += (int32_t)llround(moveY - originY);
      } else {
        shift_layer_content(&layer, moveX - originX, moveY - originY);
      }
      layer.placement.width =
          (uint32_t)MAX(llround(layer.placement.width * width_ratio), 1);
      layer.placement.height =
          (uint32_t)MAX(llround(layer.placement.height * height_ratio), 1);
    }
    [resized addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
  }
  if (!found) return 0;
  presenter.workspaceResizeApplied = YES;
  presenter.workspaceLayers = resized;
  return 1;
}

int screenwide_gpu_still_presenter_update_workspace_selected_resize(
    void *handle, uint32_t selected_layer, double origin_x_ratio,
    double origin_y_ratio, double width_ratio, double height_ratio) {
  if (handle == NULL || !isfinite(origin_x_ratio) ||
      !isfinite(origin_y_ratio) || !isfinite(width_ratio) ||
      !isfinite(height_ratio) || width_ratio <= 0.0 || height_ratio <= 0.0)
    return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  NSArray<NSValue *> *base = presenter.workspaceResizeLayers.count > 0
      ? presenter.workspaceResizeLayers
      : presenter.workspaceLayers;
  NSMutableArray<NSValue *> *resized =
      [NSMutableArray arrayWithCapacity:base.count];
  BOOL found = NO;
  for (NSValue *value in base) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == selected_layer) {
      found = YES;
      double old_width = MAX(layer.canvas_width, 1u);
      double old_height = MAX(layer.canvas_height, 1u);
      uint32_t next_width = (uint32_t)MAX(llround(old_width * width_ratio), 1);
      uint32_t next_height = (uint32_t)MAX(llround(old_height * height_ratio), 1);
      double old_shortest = MAX(MIN(old_width, old_height), 1.0);
      double next_shortest = MIN(next_width, next_height);
      double origin_x = old_width * origin_x_ratio;
      double origin_y = old_height * origin_y_ratio;
      shift_layer_content(&layer, -origin_x, -origin_y);
      // A baked camera is another layer in this canvas. Keep its absolute
      // canvas position stable when a Frame gesture moves the canvas origin,
      // matching the shared semantic rebase used at gesture commit.
      if (layer.overlay.camera_frame_width > 0) {
        layer.overlay.camera_frame_x -= (int32_t)llround(origin_x);
        layer.overlay.camera_frame_y -= (int32_t)llround(origin_y);
      }
      layer.canvas_width = next_width;
      layer.canvas_height = next_height;
      layer.canvas.background_radius = (uint32_t)MAX(
          llround(layer.canvas.background_radius * next_shortest / old_shortest), 0);
      layer.placement.width = (uint32_t)MAX(llround(layer.placement.width * width_ratio), 1);
      layer.placement.height = (uint32_t)MAX(llround(layer.placement.height * height_ratio), 1);
    }
    [resized addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
  }
  if (!found) return 0;
  presenter.workspaceResizeApplied = YES;
  presenter.workspaceLayers = resized;
  return 1;
}

int screenwide_gpu_still_presenter_update_workspace_selected_radius(
    void *handle, uint32_t selected_layer, double radius_percent, int frame) {
  if (handle == NULL || !isfinite(radius_percent)) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  NSMutableArray<NSValue *> *updated =
      [NSMutableArray arrayWithCapacity:presenter.workspaceLayers.count];
  BOOL found = NO;
  double percent = fmin(50.0, fmax(0.0, radius_percent));
  for (NSValue *value in presenter.workspaceLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == selected_layer) {
      found = YES;
      // Same pixel derivation as the export path (platform_macos.rs): the
      // frame radius is a share of the canvas's shorter side, the clip radius
      // a share of the crop's shorter side.
      if (frame != 0) {
        double shortest = MIN(layer.canvas_width, layer.canvas_height);
        layer.canvas.background_radius =
            (uint32_t)MAX(llround(shortest * percent / 100.0), 0);
      } else {
        double shortest = MIN(layer.canvas.crop_width, layer.canvas.crop_height);
        layer.canvas.radius =
            (uint32_t)MAX(llround(shortest * percent / 100.0), 0);
      }
    }
    [updated addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
  }
  if (!found) return 0;
  presenter.workspaceLayers = updated;
  return 1;
}

void screenwide_gpu_still_presenter_end_workspace_resize(
    void *handle, int commit) {
  if (handle == NULL) return;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  if (commit == 0 && presenter.workspaceResizeLayers.count > 0)
    presenter.workspaceLayers = [presenter.workspaceResizeLayers mutableCopy];
  presenter.workspaceResizeLayers = nil;
  presenter.workspaceResizeApplied = NO;
}

int screenwide_gpu_still_presenter_present_workspace(
    void *handle, void *metal_layer, const ScreenwideWorkspaceLayer *layers,
    uint32_t layer_count, ScreenwidePresentBlock present) {
  if (handle == NULL || metal_layer == NULL || layers == NULL ||
      layer_count == 0 || present == NULL) return 0;
  @autoreleasepool {
    if (!screenwide_gpu_still_presenter_set_workspace(
            handle, layers, layer_count)) return 0;
    ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
    return presenter_present_workspace_layers(
        presenter, (__bridge CAMetalLayer *)metal_layer, layers, layer_count,
        present);
  }
}

int screenwide_gpu_still_presenter_redraw_workspace(
    void *handle, void *metal_layer,
    const ScreenwideWorkspacePlacement *placements, uint32_t placement_count,
    const ScreenwideWorkspaceMagnifier *magnifier,
    ScreenwidePresentBlock present) {
  if (handle == NULL || metal_layer == NULL || placements == NULL ||
      placement_count == 0 || present == NULL) return 0;
  @autoreleasepool {
    ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
    if (presenter.workspaceLayers.count != placement_count) return 0;
    ScreenwideWorkspaceLayer *layers = calloc(placement_count, sizeof(*layers));
    if (layers == NULL) return 0;
    for (uint32_t index = 0; index < placement_count; ++index) {
      [presenter.workspaceLayers[index] getValue:&layers[index]
                                            size:sizeof(layers[index])];
      layers[index].source_rgba = NULL;
      layers[index].placement = placements[index];
      NSNumber *key = @(layers[index].source_token);
      if (presenter.workspaceSources[key] == nil) {
        free(layers);
        return 0;
      }
    }
    // The cached source buffers are bound below; source_rgba is intentionally
    // null here so a redraw cannot accidentally copy from stale CPU memory.
    int result = 0;
    id<CAMetalDrawable> drawable = [(__bridge CAMetalLayer *)metal_layer nextDrawable];
    if (drawable != nil) {
      id<MTLCommandBuffer> command = [presenter.queue commandBuffer];
      MTLSize grid = MTLSizeMake(drawable.texture.width, drawable.texture.height, 1);
      id<MTLComputeCommandEncoder> clear = [command computeCommandEncoder];
      [clear setComputePipelineState:presenter.workspaceClearPipeline];
      [clear setTexture:drawable.texture atIndex:0];
      workspace_dispatch(clear, presenter.workspaceClearPipeline, grid);
      [clear endEncoding];
      for (uint32_t index = 0; index < placement_count; ++index) {
        id<MTLBuffer> source = presenter.workspaceSources[@(layers[index].source_token)];
        id<MTLBuffer> uniforms = [presenter.device newBufferWithBytes:&layers[index].canvas
            length:sizeof(layers[index].canvas) options:MTLResourceStorageModeShared];
        id<MTLBuffer> camera = presenter.workspaceCameraSources[@(layers[index].source_token)]
            ?: [presenter.device newBufferWithLength:4 options:MTLResourceStorageModeShared];
        id<MTLBuffer> overlay = [presenter.device newBufferWithBytes:&layers[index].overlay length:sizeof(layers[index].overlay) options:MTLResourceStorageModeShared];
        uint32_t dimensions[2] = {layers[index].source_width, layers[index].source_height};
        ScreenwideOverlayUniforms cursor = screenwide_workspace_cursor_uniforms(presenter.cursorResources, &layers[index]);
        uint32_t first = index == 0 ? 1 : 0;
        id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
        [encoder setComputePipelineState:presenter.workspaceLayerPipeline];
        [encoder setBuffer:source offset:0 atIndex:0];
        [encoder setTexture:drawable.texture atIndex:0];
        [encoder setBuffer:uniforms offset:0 atIndex:1];
        [encoder setBytes:dimensions length:sizeof(dimensions) atIndex:2];
        [encoder setBytes:&layers[index].placement length:sizeof(layers[index].placement) atIndex:3];
        [encoder setBytes:&first length:sizeof(first) atIndex:4];
        uint32_t logical[2] = {layers[index].canvas_width,
                               layers[index].canvas_height};
        [encoder setBytes:logical length:sizeof(logical) atIndex:5];
        [encoder setBytes:&cursor length:sizeof(cursor) atIndex:6];
        [encoder setBuffer:camera offset:0 atIndex:7];
        [encoder setBuffer:overlay offset:0 atIndex:8];
        float seconds = (float)layers[index].seconds;
        [encoder setBytes:&seconds length:sizeof(seconds) atIndex:9];
        screenwide_bind_keyboard(encoder, presenter.device, presenter.keyboardArtworks, layers[index].keyboard, layers[index].canvas_height);
        [encoder setTexture:presenter.cursorResources.texture atIndex:1];
        workspace_dispatch(encoder, presenter.workspaceLayerPipeline, grid);
        [encoder endEncoding];
      }
      if (magnifier != NULL && magnifier->active != 0) {
        for (uint32_t index = 0; index < placement_count; ++index) {
          BOOL selectedLayer = magnifier->sample_camera != 0
              ? layers[index].pane_index == magnifier->pane_index
              : layers[index].layer_id == magnifier->layer_id;
          if (!selectedLayer) continue;
          NSNumber *token = @(layers[index].source_token);
          id<MTLBuffer> source = magnifier->sample_camera != 0
              ? presenter.workspaceCameraSources[token]
              : presenter.workspaceSources[token];
          uint32_t dimensions[2] = {
            magnifier->sample_camera != 0
                ? layers[index].overlay.camera_source_width
                : layers[index].source_width,
            magnifier->sample_camera != 0
                ? layers[index].overlay.camera_source_height
                : layers[index].source_height,
          };
          if (source == nil || dimensions[0] == 0 || dimensions[1] == 0) break;
          id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
          [encoder setComputePipelineState:presenter.workspaceMagnifierPipeline];
          [encoder setBuffer:source offset:0 atIndex:0];
          [encoder setTexture:drawable.texture atIndex:0];
          [encoder setBytes:dimensions length:sizeof(dimensions) atIndex:1];
          [encoder setBytes:magnifier length:sizeof(*magnifier) atIndex:2];
          workspace_dispatch(
              encoder, presenter.workspaceMagnifierPipeline,
              MTLSizeMake(MAX(magnifier->box_width, 1),
                          MAX(magnifier->box_height, 1), 1));
          [encoder endEncoding];
          break;
        }
      }
      present((__bridge void *)command, (__bridge void *)drawable);
      result = 1;
    }
    free(layers);
    return result;
  }
}

int screenwide_gpu_still_presenter_present(
    void *handle, void *metal_layer, uint64_t source_token,
    const uint8_t *source_rgba, uint32_t source_width, uint32_t source_height,
    const ScreenwideCanvas *canvas, double seconds, const uint8_t *cursor_rgba,
    const uint8_t *camera_rgba, const ScreenwideStillOverlay *overlay,
    ScreenwidePresentBlock present) {
  if (handle == NULL || metal_layer == NULL || source_rgba == NULL ||
      canvas == NULL || source_width == 0 || source_height == 0 ||
      present == NULL) return 0;
  @autoreleasepool {
    ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
    CAMetalLayer *layer = (__bridge CAMetalLayer *)metal_layer;
    if (presenter.source == nil || presenter.sourceToken != source_token ||
        presenter.sourceWidth != source_width || presenter.sourceHeight != source_height) {
      presenter.source = [presenter.device newBufferWithBytes:source_rgba
          length:(NSUInteger)source_width * source_height * 4
          options:MTLResourceStorageModeShared];
      presenter.sourceToken = source_token;
      presenter.sourceWidth = source_width;
      presenter.sourceHeight = source_height;
    }
    id<CAMetalDrawable> drawable = [layer nextDrawable];
    if (drawable == nil) return 0;
    ScreenwideStillOverlay empty_overlay = {0};
    if (overlay == NULL) overlay = &empty_overlay;
    id<MTLBuffer> uniforms = [presenter.device newBufferWithBytes:canvas
      length:sizeof(*canvas) options:MTLResourceStorageModeShared];
    id<MTLBuffer> cursor = cursor_rgba == NULL
      ? [presenter.device newBufferWithLength:4 options:MTLResourceStorageModeShared]
      : [presenter.device newBufferWithBytes:cursor_rgba
          length:(NSUInteger)overlay->cursor_source_width * overlay->cursor_source_height * 4
          options:MTLResourceStorageModeShared];
    id<MTLBuffer> camera = camera_rgba == NULL
      ? [presenter.device newBufferWithLength:4 options:MTLResourceStorageModeShared]
      : [presenter.device newBufferWithBytes:camera_rgba
          length:(NSUInteger)overlay->camera_source_width * overlay->camera_source_height * 4
          options:MTLResourceStorageModeShared];
    id<MTLBuffer> overlay_uniforms = [presenter.device newBufferWithBytes:overlay
      length:sizeof(*overlay) options:MTLResourceStorageModeShared];
    uint32_t dimensions[2] = {source_width, source_height};
    float time = (float)seconds;
    id<MTLCommandBuffer> command = [presenter.queue commandBuffer];
    id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
    [encoder setComputePipelineState:presenter.pipeline];
    [encoder setBuffer:presenter.source offset:0 atIndex:0];
    [encoder setTexture:drawable.texture atIndex:0];
    [encoder setBuffer:uniforms offset:0 atIndex:1];
    [encoder setBytes:dimensions length:sizeof(dimensions) atIndex:2];
    [encoder setBytes:&time length:sizeof(time) atIndex:3];
    [encoder setBuffer:cursor offset:0 atIndex:4];
    [encoder setBuffer:camera offset:0 atIndex:5];
    [encoder setBuffer:overlay_uniforms offset:0 atIndex:6];
    MTLSize grid = MTLSizeMake(drawable.texture.width, drawable.texture.height, 1);
    NSUInteger width = MIN(presenter.pipeline.threadExecutionWidth, grid.width);
    NSUInteger height = MIN(MAX((NSUInteger)1,
      presenter.pipeline.maxTotalThreadsPerThreadgroup / MAX(width, (NSUInteger)1)), grid.height);
    [encoder dispatchThreads:grid threadsPerThreadgroup:MTLSizeMake(width, height, 1)];
    [encoder endEncoding];
    present((__bridge void *)command, (__bridge void *)drawable);
    return 1;
  }
}

void screenwide_gpu_still_presenter_destroy(void *handle) {
  if (handle != NULL) CFBridgingRelease(handle);
}
