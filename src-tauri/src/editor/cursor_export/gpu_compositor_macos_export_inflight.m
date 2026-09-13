// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_export_private.h"

@implementation ScreenwideInflightFrame
- (void)dealloc {
  if (_destination != NULL)
    CVPixelBufferRelease(_destination);
  if (_sourceLuma != NULL)
    CFRelease(_sourceLuma);
  if (_sourceChroma != NULL)
    CFRelease(_sourceChroma);
  if (_destinationLuma != NULL)
    CFRelease(_destinationLuma);
  if (_destinationChroma != NULL)
    CFRelease(_destinationChroma);
  if (_cameraTexture != NULL)
    CFRelease(_cameraTexture);
  if (_screenSample != NULL)
    CFRelease(_screenSample);
  if (_cameraSample != NULL)
    CFRelease(_cameraSample);
}
@end

/// Waits for the oldest in-flight frame and appends it. The ring pops
/// oldest-first, so source order is structural; by the time this waits the
/// command buffer is almost always already done and the encoder has had three
/// frames worth of head start.
ScreenwideDrainResult screenwide_export_drain_inflight_frame(
    ScreenwideInflightFrame *frame,
    AVAssetWriterInputPixelBufferAdaptor *adaptor, AVAssetWriterInput *input,
    AVAssetWriter *writer, void *context, ScreenwideShouldCancel should_cancel,
    ScreenwideProgress progress, BOOL *primed, float source_frame_rate,
    NSError **error) {
  [frame.command waitUntilCompleted];
  if (frame.command.status == MTLCommandBufferStatusError) {
    *error =
        frame.command.error
            ?: [NSError errorWithDomain:@"ScreenwideGPUCompositor"
                                   code:3
                               userInfo:@{
                                 NSLocalizedDescriptionKey :
                                     @"The GPU encoder rejected a video frame"
                               }];
    return ScreenwideDrainFailed;
  }
  // The residual encoder wait. It now overlaps with the newer frames whose
  // decode and GPU work is already in flight, which is the point of the ring.
  while (!input.isReadyForMoreMediaData) {
    if (should_cancel != NULL && should_cancel(context))
      return ScreenwideDrainCancelled;
    [NSThread sleepForTimeInterval:0.001];
  }
  if (!*primed) {
    *primed = YES;
    // Hardware rate control ramps up over its first seconds (the first
    // keyframe of an export measures at half the steady-state size).
    // Feeding the first frame repeatedly at negative timestamps warms
    // the encoder on samples the edit list trims away, so the visible
    // first frame starts at steady-state quality.
    int32_t warm_fps = (int32_t)MAX(llround(source_frame_rate), 1);
    for (int32_t warm = 45; warm >= 1; warm--) {
      while (!input.isReadyForMoreMediaData)
        [NSThread sleepForTimeInterval:0.001];
      if (![adaptor appendPixelBuffer:frame.destination
                 withPresentationTime:CMTimeMake(-warm, warm_fps)])
        break;
    }
    while (!input.isReadyForMoreMediaData)
      [NSThread sleepForTimeInterval:0.001];
  }
  if (![adaptor appendPixelBuffer:frame.destination
             withPresentationTime:frame.presentation]) {
    *error =
        writer.error
            ?: [NSError errorWithDomain:@"ScreenwideGPUCompositor"
                                   code:3
                               userInfo:@{
                                 NSLocalizedDescriptionKey :
                                     @"The GPU encoder rejected a video frame"
                               }];
    return ScreenwideDrainFailed;
  }
  if (progress != NULL)
    progress(context,
             (uint64_t)llround(CMTimeGetSeconds(frame.presentation) * 1000.0));
  return ScreenwideDrainAppended;
}
