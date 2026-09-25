// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_export_private.h"
#import "gpu_compositor_macos_generators_layered.h"
#import "gpu_compositor_macos_redact.h"

static NSArray<AVAssetTrack *> *video_tracks(AVURLAsset *asset,
                                             NSError **error) {
  dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
  __block NSArray<AVAssetTrack *> *tracks = nil;
  __block NSError *load_error = nil;
  [asset loadTracksWithMediaType:AVMediaTypeVideo
               completionHandler:^(NSArray<AVAssetTrack *> *loaded,
                                   NSError *failure) {
                 tracks = loaded;
                 load_error = failure;
                 dispatch_semaphore_signal(semaphore);
               }];
  dispatch_semaphore_wait(semaphore, DISPATCH_TIME_FOREVER);
  if (error != NULL)
    *error = load_error;
  return tracks;
}

static AVAssetReaderTrackOutput *
reader_output(AVAssetReader *reader, AVAssetTrack *track, OSType format,
              NSNumber *width, NSNumber *height, NSError **error) {
  NSMutableDictionary *settings = [@{
    (NSString *)kCVPixelBufferPixelFormatTypeKey : @(format),
    (NSString *)kCVPixelBufferMetalCompatibilityKey : @YES,
    (NSString *)kCVPixelBufferIOSurfacePropertiesKey : @{},
  } mutableCopy];
  if (width != nil)
    settings[(NSString *)kCVPixelBufferWidthKey] = width;
  if (height != nil)
    settings[(NSString *)kCVPixelBufferHeightKey] = height;
  AVAssetReaderTrackOutput *output =
      [[AVAssetReaderTrackOutput alloc] initWithTrack:track
                                       outputSettings:settings];
  output.alwaysCopiesSampleData = NO;
  if (![reader canAddOutput:output]) {
    if (error != NULL) {
      *error = [NSError errorWithDomain:@"ScreenwideGPUCompositor"
                                   code:1
                               userInfo:@{
                                 NSLocalizedDescriptionKey :
                                     @"AVFoundation rejected a GPU video reader"
                               }];
    }
    return nil;
  }
  [reader addOutput:output];
  return output;
}

@implementation ScreenwideVideoExport (Setup)
- (int)prepareScreen:(const char *)screen_path
              camera:(const char *)camera_path
              output:(const char *)output_path
             bitrate:(uint64_t)bitrate
           errorText:(char *)error_text
       errorCapacity:(size_t)error_capacity
               error:(NSError **)export_error {
  NSError *error = nil;
  AVURLAsset *screen_asset =
      [AVURLAsset URLAssetWithURL:[NSURL fileURLWithPath:@(screen_path)]
                          options:nil];
  AVURLAsset *camera_asset =
      camera_path == NULL
          ? nil
          : [AVURLAsset URLAssetWithURL:[NSURL fileURLWithPath:@(camera_path)]
                                options:nil];
  AVAssetTrack *screen_track = video_tracks(screen_asset, &error).firstObject;
  if (screen_track == nil && error != nil)
    return fail(error_text, error_capacity, error.localizedDescription);
  AVAssetTrack *camera_track =
      camera_asset == nil ? nil
                          : video_tracks(camera_asset, &error).firstObject;
  if (screen_track == nil)
    return fail(error_text, error_capacity,
                @"The GPU compositor could not find the recording track");
  if (camera_asset != nil && camera_track == nil)
    return fail(error_text, error_capacity,
                error.localizedDescription
                    ?: @"The GPU compositor could not find the camera track");

  screen_reader = [[AVAssetReader alloc] initWithAsset:screen_asset
                                                 error:&error];
  screen_output = reader_output(screen_reader, screen_track,
                                kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
                                nil, nil, &error);
  if (screen_output == nil)
    return fail(error_text, error_capacity, error.localizedDescription);
  camera_reader = camera_asset == nil
                      ? nil
                      : [[AVAssetReader alloc] initWithAsset:camera_asset
                                                       error:&error];
  camera_output =
      camera_reader == nil
          ? nil
          : reader_output(camera_reader, camera_track,
                          kCVPixelFormatType_32BGRA, nil, nil, &error);
  if (camera_reader != nil && camera_output == nil)
    return fail(error_text, error_capacity, error.localizedDescription);

  output_url = [NSURL fileURLWithPath:@(output_path)];
  [[NSFileManager defaultManager] removeItemAtURL:output_url error:nil];
  writer = [[AVAssetWriter alloc] initWithURL:output_url
                                     fileType:AVFileTypeMPEG4
                                        error:&error];
  if (writer == nil)
    return fail(error_text, error_capacity, error.localizedDescription);
  writer.shouldOptimizeForNetworkUse = YES;
  source_frame_rate = screen_track.nominalFrameRate;
  if (!isfinite(source_frame_rate) || source_frame_rate < 60.0)
    source_frame_rate = 60.0;
  source_duration_us = (uint64_t)llround(CMTimeGetSeconds(CMTimeRangeGetEnd(screen_track.timeRange)) * 1000000.0);
  NSNumber *expected_frame_rate = @((NSInteger)llround(source_frame_rate));
  NSDictionary *video_settings = @{
    AVVideoCodecKey : AVVideoCodecTypeH264,
    AVVideoWidthKey : @(output_width),
    AVVideoHeightKey : @(output_height),
    AVVideoCompressionPropertiesKey : @{
      AVVideoAverageBitRateKey : @(bitrate),
      AVVideoExpectedSourceFrameRateKey : expected_frame_rate,
      AVVideoAverageNonDroppableFrameRateKey : expected_frame_rate,
      AVVideoMaxKeyFrameIntervalKey : @(expected_frame_rate.integerValue * 4),
      AVVideoMaxKeyFrameIntervalDurationKey : @4,
      AVVideoAllowFrameReorderingKey : @NO,
      AVVideoH264EntropyModeKey : AVVideoH264EntropyModeCABAC,
      AVVideoProfileLevelKey : AVVideoProfileLevelH264HighAutoLevel,
    },
  };
  writer_input = [[AVAssetWriterInput alloc] initWithMediaType:AVMediaTypeVideo
                                                outputSettings:video_settings];
  NSDictionary *pixel_attributes = @{
    (NSString *)kCVPixelBufferPixelFormatTypeKey :
        @(kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange),
    (NSString *)kCVPixelBufferWidthKey : @(output_width),
    (NSString *)kCVPixelBufferHeightKey : @(output_height),
    (NSString *)kCVPixelBufferMetalCompatibilityKey : @YES,
    (NSString *)kCVPixelBufferIOSurfacePropertiesKey : @{},
  };
  adaptor = [[AVAssetWriterInputPixelBufferAdaptor alloc]
         initWithAssetWriterInput:writer_input
      sourcePixelBufferAttributes:pixel_attributes];
  if (![writer canAddInput:writer_input])
    return fail(error_text, error_capacity,
                @"AVFoundation rejected the GPU video writer");
  [writer addInput:writer_input];

  device = MTLCreateSystemDefaultDevice();
  id<MTLLibrary> library =
      [device newLibraryWithSource:screenwide_gpu_canvas_shader_source()
                           options:nil
                             error:&error];
  luma_pipeline = [device newComputePipelineStateWithFunction:
                              [library newFunctionWithName:@"overlay_luma"]
                                                        error:&error];
  chroma_pipeline = [device newComputePipelineStateWithFunction:
                                [library newFunctionWithName:@"overlay_chroma"]
                                                          error:&error];
  keyboard_luma_pipeline = screenwide_keyboard_pipeline(
      device, library, @"overlay_keyboard_luma", &error);
  keyboard_chroma_pipeline = screenwide_keyboard_pipeline(
      device, library, @"overlay_keyboard_chroma", &error);
  annotation_luma_pipeline = screenwide_keyboard_pipeline(device, library, @"overlay_annotation_luma", &error);
  annotation_chroma_pipeline = screenwide_keyboard_pipeline(device, library, @"overlay_annotation_chroma", &error);
  redact_pipelines = screenwide_redact_pipelines(library);
  camera_luma_pipeline =
      [device newComputePipelineStateWithFunction:
                  [library newFunctionWithName:@"overlay_camera_luma"]
                                            error:&error];
  camera_chroma_pipeline =
      [device newComputePipelineStateWithFunction:
                  [library newFunctionWithName:@"overlay_camera_chroma"]
                                            error:&error];
  canvas_luma_pipeline =
      [device newComputePipelineStateWithFunction:
                  [library newFunctionWithName:@"compose_canvas_luma"]
                                            error:&error];
  canvas_chroma_pipeline =
      [device newComputePipelineStateWithFunction:
                  [library newFunctionWithName:@"compose_canvas_chroma"]
                                            error:&error];
  screen_luma_pipeline =
      [device newComputePipelineStateWithFunction:
                  [library newFunctionWithName:@"overlay_screen_luma"]
                                            error:&error];
  screen_chroma_pipeline =
      [device newComputePipelineStateWithFunction:
                  [library newFunctionWithName:@"overlay_screen_chroma"]
                                            error:&error];
  queue = [device newCommandQueue];
  cursor_artwork =
      screenwide_export_cursor_artwork_texture(device, artworks, artwork_count);
  keyboard_cache = [NSMutableDictionary dictionary];
  texture_cache = NULL;
  CVMetalTextureCacheCreate(kCFAllocatorDefault, NULL, device, NULL,
                            &texture_cache);
  if (device == nil || library == nil || luma_pipeline == nil ||
      chroma_pipeline == nil || keyboard_luma_pipeline == nil ||
      keyboard_chroma_pipeline == nil || camera_luma_pipeline == nil ||
      camera_chroma_pipeline == nil || queue == nil ||
      canvas_luma_pipeline == nil || canvas_chroma_pipeline == nil ||
      screen_luma_pipeline == nil || screen_chroma_pipeline == nil ||
      annotation_luma_pipeline == nil || annotation_chroma_pipeline == nil ||
      redact_pipelines == nil ||
      texture_cache == NULL)
    return fail(error_text, error_capacity,
                error.localizedDescription
                    ?: @"The Metal cursor shader could not be created");

  *export_error = error;
  return 1;
}
@end
