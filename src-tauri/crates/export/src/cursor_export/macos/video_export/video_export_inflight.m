// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "video_export.h"

#include <errno.h>
#include <unistd.h>

@implementation ScreenwideInflightFrame
- (void)dealloc {
  if (_destination != NULL) CVPixelBufferRelease(_destination);
  if (_screenSample != NULL) CFRelease(_screenSample);
  if (_cameraSample != NULL) CFRelease(_cameraSample);
}

- (instancetype)repeatedAt:(CMTime)presentation {
  ScreenwideInflightFrame *repeat = [ScreenwideInflightFrame new];
  repeat.token = _token;
  repeat.presentation = presentation;
  repeat.destination = CVPixelBufferRetain(_destination);
  return repeat;
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

/// Writes all of `bytes` to `fd`, however a pipe splits it.
static BOOL write_all(int fd, const uint8_t *bytes, size_t length) {
  while (length > 0) {
    ssize_t written = write(fd, bytes, length);
    if (written < 0) {
      if (errno == EINTR) continue;
      return NO;
    }
    bytes += written;
    length -= (size_t)written;
  }
  return YES;
}

/// Copies `buffer`'s two planes into `staging` with their rows packed:
/// IOSurface rows are padded past the picture, and raw video has none.
static BOOL pack_planes(CVPixelBufferRef buffer, NSMutableData *staging) {
  if (CVPixelBufferLockBaseAddress(buffer, kCVPixelBufferLock_ReadOnly) != kCVReturnSuccess)
    return NO;
  uint8_t *out = staging.mutableBytes;
  size_t room = staging.length;
  BOOL packed = YES;
  for (size_t plane = 0; plane < 2 && packed; plane++) {
    const uint8_t *base = CVPixelBufferGetBaseAddressOfPlane(buffer, plane);
    size_t stride = CVPixelBufferGetBytesPerRowOfPlane(buffer, plane);
    // Chroma interleaves two bytes, Cb and Cr, for each of its samples.
    size_t row = CVPixelBufferGetWidthOfPlane(buffer, plane) * (plane == 0 ? 1 : 2);
    size_t rows = CVPixelBufferGetHeightOfPlane(buffer, plane);
    packed = base != NULL && row <= stride && row * rows <= room;
    for (size_t y = 0; y < rows && packed; y++) {
      memcpy(out, base + y * stride, row);
      out += row;
    }
    room -= packed ? row * rows : 0;
  }
  CVPixelBufferUnlockBaseAddress(buffer, kCVPixelBufferLock_ReadOnly);
  return packed && room == 0;
}

ScreenwideDrainResult screenwide_export_stream_inflight_frame(
    ScreenwideInflightFrame *frame, int fd, NSMutableData *staging,
    const ScreenwideExportCallbacks *callbacks, NSError **error) {
  if (!callbacks->wait(callbacks->context, frame.token)) {
    *error = export_error(@"The GPU did not finish a video frame");
    return ScreenwideDrainFailed;
  }
  if (!pack_planes(frame.destination, staging)) {
    *error = export_error(@"A video frame could not be read back for the encoder");
    return ScreenwideDrainFailed;
  }
  // Blocks while the encoder is behind, which paces the loop as the
  // writer's readiness does.
  if (!write_all(fd, staging.bytes, staging.length)) {
    *error = export_error(@"The video encoder stopped taking frames");
    return ScreenwideDrainFailed;
  }
  if (callbacks->should_cancel(callbacks->context)) return ScreenwideDrainCancelled;
  callbacks->progress(callbacks->context,
                      (uint64_t)llround(CMTimeGetSeconds(frame.presentation) * 1000.0));
  return ScreenwideDrainAppended;
}
