// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "video_export.h"

static NSArray<AVAssetTrack *> *video_tracks(AVURLAsset *asset, NSError **error) {
  dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
  __block NSArray<AVAssetTrack *> *tracks = nil;
  __block NSError *load_error = nil;
  [asset loadTracksWithMediaType:AVMediaTypeVideo
               completionHandler:^(NSArray<AVAssetTrack *> *loaded, NSError *failure) {
                 tracks = loaded;
                 load_error = failure;
                 dispatch_semaphore_signal(semaphore);
               }];
  dispatch_semaphore_wait(semaphore, DISPATCH_TIME_FOREVER);
  if (error != NULL) *error = load_error;
  return tracks;
}

/// The GPU samples and draws into these buffers in place, so they are asked
/// for backed by IOSurfaces it can open.
static AVAssetReaderTrackOutput *reader_output(AVAssetReader *reader, AVAssetTrack *track,
                                               OSType format, NSError **error) {
  NSDictionary *settings = @{
    (NSString *)kCVPixelBufferPixelFormatTypeKey : @(format),
    (NSString *)kCVPixelBufferMetalCompatibilityKey : @YES,
    (NSString *)kCVPixelBufferIOSurfacePropertiesKey : @{},
  };
  AVAssetReaderTrackOutput *output = [[AVAssetReaderTrackOutput alloc] initWithTrack:track
                                                                      outputSettings:settings];
  output.alwaysCopiesSampleData = NO;
  if (![reader canAddOutput:output]) {
    if (error != NULL) {
      *error = [NSError
          errorWithDomain:@"ScreenwideVideoExport"
                     code:1
                 userInfo:@{NSLocalizedDescriptionKey : @"AVFoundation rejected a video reader"}];
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
       errorCapacity:(size_t)error_capacity {
  NSError *error = nil;
  AVURLAsset *screen_asset =
      [AVURLAsset URLAssetWithURL:[NSURL fileURLWithPath:@(screen_path)] options:nil];
  AVURLAsset *camera_asset =
      camera_path == NULL
          ? nil
          : [AVURLAsset URLAssetWithURL:[NSURL fileURLWithPath:@(camera_path)] options:nil];
  AVAssetTrack *screen_track = video_tracks(screen_asset, &error).firstObject;
  if (screen_track == nil && error != nil)
    return fail(error_text, error_capacity, error.localizedDescription);
  AVAssetTrack *camera_track =
      camera_asset == nil ? nil : video_tracks(camera_asset, &error).firstObject;
  if (screen_track == nil)
    return fail(error_text, error_capacity, @"The video export could not find the recording track");
  if (camera_asset != nil && camera_track == nil)
    return fail(error_text, error_capacity,
                error.localizedDescription ?: @"The video export could not find the camera track");

  screen_reader = [[AVAssetReader alloc] initWithAsset:screen_asset error:&error];
  screen_output = reader_output(screen_reader, screen_track,
                                kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange, &error);
  if (screen_output == nil) return fail(error_text, error_capacity, error.localizedDescription);
  camera_reader =
      camera_asset == nil ? nil : [[AVAssetReader alloc] initWithAsset:camera_asset error:&error];
  camera_output = camera_reader == nil
                      ? nil
                      : reader_output(camera_reader, camera_track, kCVPixelFormatType_32BGRA,
                                      &error);
  if (camera_reader != nil && camera_output == nil)
    return fail(error_text, error_capacity, error.localizedDescription);

  output_url = [NSURL fileURLWithPath:@(output_path)];
  [[NSFileManager defaultManager] removeItemAtURL:output_url error:nil];
  writer = [[AVAssetWriter alloc] initWithURL:output_url fileType:AVFileTypeMPEG4 error:&error];
  if (writer == nil) return fail(error_text, error_capacity, error.localizedDescription);
  writer.shouldOptimizeForNetworkUse = YES;
  source_frame_rate = screen_track.nominalFrameRate;
  if (!isfinite(source_frame_rate) || source_frame_rate < 60.0) source_frame_rate = 60.0;
  source_duration_us = (uint64_t)llround(
      CMTimeGetSeconds(CMTimeRangeGetEnd(screen_track.timeRange)) * 1000000.0);
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
    return fail(error_text, error_capacity, @"AVFoundation rejected the video writer");
  [writer addInput:writer_input];
  return 1;
}
@end
