// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "video_export.h"

@implementation ScreenwideInflightFrame
- (void)dealloc {
  if (_destination != NULL) CVPixelBufferRelease(_destination);
  if (_screenSample != NULL) CFRelease(_screenSample);
  if (_cameraSample != NULL) CFRelease(_cameraSample);
}
@end

static NSError *export_error(NSString *message) {
  return [NSError errorWithDomain:@"ScreenwideVideoExport"
                             code:3
                         userInfo:@{NSLocalizedDescriptionKey : message}];
}

/// Waits for the oldest in-flight frame and appends it. The ring pops
/// oldest-first, so source order is structural; by the time this waits the
/// GPU is almost always already done and the encoder has had three frames
/// worth of head start.
ScreenwideDrainResult screenwide_export_drain_inflight_frame(
    ScreenwideInflightFrame *frame, AVAssetWriterInputPixelBufferAdaptor *adaptor,
    AVAssetWriterInput *input, AVAssetWriter *writer, const ScreenwideExportCallbacks *callbacks,
    BOOL *primed, float source_frame_rate, NSError **error) {
  if (!callbacks->wait(callbacks->context, frame.token)) {
    *error = export_error(@"The GPU did not finish a video frame");
    return ScreenwideDrainFailed;
  }
  // The residual encoder wait. It overlaps with the newer frames whose
  // decode and GPU work is already in flight, which is the point of the ring.
  while (!input.isReadyForMoreMediaData) {
    if (callbacks->should_cancel(callbacks->context)) return ScreenwideDrainCancelled;
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
      while (!input.isReadyForMoreMediaData) [NSThread sleepForTimeInterval:0.001];
      if (![adaptor appendPixelBuffer:frame.destination
                 withPresentationTime:CMTimeMake(-warm, warm_fps)])
        break;
    }
    while (!input.isReadyForMoreMediaData) [NSThread sleepForTimeInterval:0.001];
  }
  if (![adaptor appendPixelBuffer:frame.destination withPresentationTime:frame.presentation]) {
    *error = writer.error ?: export_error(@"The encoder rejected a video frame");
    return ScreenwideDrainFailed;
  }
  callbacks->progress(callbacks->context,
                      (uint64_t)llround(CMTimeGetSeconds(frame.presentation) * 1000.0));
  return ScreenwideDrainAppended;
}
