// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once
#import "gpu_compositor_macos.h"
#import "gpu_compositor_macos_annotations.h"
#import "gpu_compositor_macos_background_image.h"
#import "gpu_compositor_macos_cursor_resources.h"
#import "gpu_compositor_macos_keyboard.h"
#import <AVFoundation/AVFoundation.h>
#import <Metal/Metal.h>
#import <VideoToolbox/VideoToolbox.h>
#include <math.h>

typedef bool (*ScreenwideShouldCancel)(void *context);
typedef void (*ScreenwideProgress)(void *context, uint64_t position_ms);

/// The artwork description the shader needs, without the caller's bitmap
/// pointer. Slice `style` of the artwork texture array holds the pixels.
typedef struct {
  uint32_t crop_x;
  uint32_t crop_y;
  uint32_t crop_width;
  uint32_t crop_height;
  int32_t frame_x;
  int32_t frame_y;
  uint32_t frame_width;
  uint32_t frame_height;
  uint32_t radius;
  uint32_t drop_shadow;
  uint32_t camera_on_top;
} ScreenwideCameraOverlay;

typedef struct {
  uint32_t crop_x;
  uint32_t crop_y;
  uint32_t crop_width;
  uint32_t crop_height;
  int32_t frame_x;
  int32_t frame_y;
  uint32_t frame_width;
  uint32_t frame_height;
  uint32_t radius;
  uint32_t source_width;
  uint32_t source_height;
  uint32_t drop_shadow;
} ScreenwideCameraUniforms;

typedef struct {
  uint64_t output_start_us;
  uint64_t source_end_us;
  uint64_t source_start_us;
  double playback_rate;
} ScreenwideTimelineRange;

static inline int fail(char *error, size_t capacity, NSString *message) {
  if (error != NULL && capacity > 0) {
    snprintf(error, capacity, "%s",
             (message ?: @"The GPU compositor failed").UTF8String);
  }
  return 0;
}

/// One frame whose Metal work is committed but not yet handed to the writer.
/// Everything the command buffer touches lives here until it completes: the
/// output pixel buffer, the Metal textures wrapping it and the source, and the
/// sample buffers those source textures read from.
@interface ScreenwideInflightFrame : NSObject
@property(nonatomic, strong) id<MTLCommandBuffer> command;
@property(nonatomic) CMTime presentation;
@property(nonatomic) CVPixelBufferRef destination;
@property(nonatomic) CVMetalTextureRef sourceLuma;
@property(nonatomic) CVMetalTextureRef sourceChroma;
@property(nonatomic) CVMetalTextureRef destinationLuma;
@property(nonatomic) CVMetalTextureRef destinationChroma;
@property(nonatomic) CVMetalTextureRef cameraTexture;
@property(nonatomic) CMSampleBufferRef screenSample;
@property(nonatomic) CMSampleBufferRef cameraSample;
@end

typedef enum {
  ScreenwideDrainAppended,
  ScreenwideDrainCancelled,
  ScreenwideDrainFailed,
} ScreenwideDrainResult;

__attribute__((visibility("hidden"))) ScreenwideDrainResult
screenwide_export_drain_inflight_frame(
    ScreenwideInflightFrame *frame,
    AVAssetWriterInputPixelBufferAdaptor *adaptor, AVAssetWriterInput *input,
    AVAssetWriter *writer, void *context, ScreenwideShouldCancel should_cancel,
    ScreenwideProgress progress, BOOL *primed, float source_frame_rate,
    NSError **error);

__attribute__((visibility("hidden"))) id<MTLTexture>
screenwide_export_cursor_artwork_texture(
    id<MTLDevice> device, const ScreenwideCursorArtwork *artworks,
    uint32_t count);
__attribute__((visibility("hidden"))) const ScreenwideGpuCursor *
screenwide_export_cursor_at(const ScreenwideGpuCursor *cursors, uint32_t count,
                            CMTime pts);
__attribute__((visibility("hidden"))) void
screenwide_export_encode_cursor_overlay(
    id<MTLCommandBuffer> command, id<MTLComputePipelineState> luma_pipeline,
    id<MTLComputePipelineState> chroma_pipeline, id<MTLTexture> destination_y,
    id<MTLTexture> destination_uv, id<MTLTexture> artwork_texture,
    const ScreenwideGpuCursor *cursor, const ScreenwideCursorArtwork *artworks,
    uint32_t artwork_count, const ScreenwideCanvas *canvas,
    uint32_t output_width, uint32_t output_height);

/// Per-export resources shared by setup and encoding. Input pointers remain
/// borrowed from the synchronous C entry point until its frame ring is drained.
@interface ScreenwideVideoExport : NSObject {
@public
  AVAssetReader *screen_reader;
  AVAssetReader *camera_reader;
  AVAssetReaderTrackOutput *screen_output;
  AVAssetReaderTrackOutput *camera_output;
  AVAssetWriter *writer;
  AVAssetWriterInput *writer_input;
  AVAssetWriterInputPixelBufferAdaptor *adaptor;
  NSURL *output_url;
  float source_frame_rate;
  id<MTLDevice> device;
  id<MTLCommandQueue> queue;
  id<MTLTexture> cursor_artwork;
  NSMutableDictionary *keyboard_cache;
  CVMetalTextureCacheRef texture_cache;
  const ScreenwideCanvas *canvas;
  const ScreenwideCameraOverlay *camera_overlay;
  const ScreenwideCursorArtwork *artworks;
  uint32_t artwork_count;
  uint32_t output_width;
  uint32_t output_height;
  id<MTLComputePipelineState> luma_pipeline;
  id<MTLComputePipelineState> chroma_pipeline;
  id<MTLComputePipelineState> keyboard_luma_pipeline;
  id<MTLComputePipelineState> keyboard_chroma_pipeline;
  id<MTLComputePipelineState> camera_luma_pipeline;
  id<MTLComputePipelineState> camera_chroma_pipeline;
  id<MTLComputePipelineState> canvas_luma_pipeline;
  id<MTLComputePipelineState> canvas_chroma_pipeline;
  id<MTLComputePipelineState> screen_luma_pipeline;
  id<MTLComputePipelineState> screen_chroma_pipeline;
}
@end

@interface ScreenwideVideoExport (Setup)
- (int)prepareScreen:(const char *)screen_path
              camera:(const char *)camera_path
              output:(const char *)output_path
             bitrate:(uint64_t)bitrate
           errorText:(char *)error_text
       errorCapacity:(size_t)error_capacity
               error:(NSError **)export_error;
@end

@interface ScreenwideVideoExport (Encoding)
- (ScreenwideInflightFrame *)encodeScreen:(CMSampleBufferRef)screen_sample
                                   camera:(CMSampleBufferRef)camera_sample
                              destination:(CVPixelBufferRef)destination
                                   cursor:(const ScreenwideGpuCursor *)cursor
                                 keyboard:
                                     (const ScreenwideKeyboardOverlay *)keyboard
                               sourceTime:(CMTime)pts
                               outputTime:(CMTime)output_pts;
@end
