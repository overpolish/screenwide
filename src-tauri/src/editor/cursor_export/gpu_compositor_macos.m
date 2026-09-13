// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_export_private.h"

#import "gpu_compositor_macos_shader_source.h"

static bool timeline_presentation(CMTime source,
                                  const ScreenwideTimelineRange *ranges,
                                  uint32_t count, CMTime *output) {
  if (ranges == NULL || count == 0) {
    *output = source;
    return true;
  }
  double source_us = CMTimeGetSeconds(source) * 1000000.0;
  if (!isfinite(source_us) || source_us < 0.0)
    return false;
  uint64_t rounded_source_us = (uint64_t)llround(source_us);
  for (uint32_t index = 0; index < count; ++index) {
    const ScreenwideTimelineRange *range = &ranges[index];
    if (rounded_source_us < range->source_start_us ||
        rounded_source_us >= range->source_end_us)
      continue;
    uint64_t output_us =
        range->output_start_us +
        (uint64_t)llround((rounded_source_us - range->source_start_us) /
                          range->playback_rate);
    *output = CMTimeMake((int64_t)output_us, 1000000);
    return true;
  }
  return false;
}

/// How many composited frames may sit on the GPU while the loop keeps
/// decoding. A four second sample of the old fully serial loop (decode ->
/// composite -> block on the GPU -> poll the encoder -> append) spent 63% of
/// its time sleeping on `readyForMoreMediaData` and 34% inside
/// `waitUntilCompleted`, which put a 3494x2260 export at 0.4x realtime: every
/// stage idled while another worked. Keeping three frames in flight lets the
/// encoder chew on frame K while the GPU composites K+1 and the reader decodes
/// K+2, so the two waits overlap with real work instead of each other. Three is
/// enough to cover both stalls (the deepest measured stall was one frame of
/// encoder backpressure) without holding many 4K frame buffers - the ring costs
/// this many pool buffers plus the one being encoded, and the reader's own
/// sample buffers stay retained just as long.
#define SCREENWIDE_GPU_INFLIGHT_FRAMES 3

@implementation ScreenwideVideoExport
@end

int screenwide_gpu_composite_cursor(
    const char *screen_path, const ScreenwideGpuCursor *cursors,
    uint32_t cursor_count, const ScreenwideCursorArtwork *artworks,
    uint32_t artwork_count, const ScreenwideKeyboardOverlay *keyboards,
    uint32_t keyboard_count, const ScreenwideTimelineRange *timeline_ranges,
    uint32_t timeline_range_count, const char *camera_path,
    const ScreenwideCameraOverlay *camera_overlay,
    const ScreenwideCanvas *canvas, const char *output_path,
    uint32_t source_width, uint32_t source_height, uint32_t output_width,
    uint32_t output_height, uint64_t bitrate, void *context,
    ScreenwideShouldCancel should_cancel, ScreenwideProgress progress,
    char *error_text, size_t error_capacity) {
  (void)source_width;
  (void)source_height;
  @autoreleasepool {
    ScreenwideVideoExport *session = [ScreenwideVideoExport new];
    session->canvas = canvas;
    session->camera_overlay = camera_overlay;
    session->artworks = artworks;
    session->artwork_count = artwork_count;
    session->output_width = output_width;
    session->output_height = output_height;

    NSError *error = nil;
    if (![session prepareScreen:screen_path
                         camera:camera_path
                         output:output_path
                        bitrate:bitrate
                      errorText:error_text
                  errorCapacity:error_capacity
                          error:&error])
      return 0;
    if (![session->screen_reader startReading] ||
        (session->camera_reader != nil &&
         ![session->camera_reader startReading]) ||
        ![session->writer startWriting]) {
      CFRelease(session->texture_cache);
      return fail(error_text, error_capacity,
                  session->screen_reader.error.localizedDescription ?:
                    (session->camera_reader != nil ? session->camera_reader.error.localizedDescription : nil) ?:
                    session->writer.error.localizedDescription ?:
                    @"The GPU export could not be started");
    }
    [session->writer startSessionAtSourceTime:kCMTimeZero];
    BOOL primed = NO;
    // Frames whose Metal work is committed but not yet appended, oldest first.
    NSMutableArray<ScreenwideInflightFrame *> *ring =
        [NSMutableArray arrayWithCapacity:SCREENWIDE_GPU_INFLIGHT_FRAMES + 1];
    CMSampleBufferRef camera_sample = NULL;
    CMSampleBufferRef next_camera_sample =
        session->camera_output == nil
            ? NULL
            : [session->camera_output copyNextSampleBuffer];
    bool cancelled = false;
    CMSampleBufferRef screen_sample = NULL;
    while ((screen_sample = [session->screen_output copyNextSampleBuffer]) !=
           NULL) {
      @autoreleasepool {
        if (should_cancel != NULL && should_cancel(context)) {
          cancelled = true;
          CFRelease(screen_sample);
          break;
        }
        CMTime pts = CMSampleBufferGetPresentationTimeStamp(screen_sample);
        CMTime output_pts = kCMTimeInvalid;
        if (!timeline_presentation(pts, timeline_ranges, timeline_range_count,
                                   &output_pts)) {
          CFRelease(screen_sample);
          continue;
        }
        const ScreenwideGpuCursor *cursor =
            screenwide_export_cursor_at(cursors, cursor_count, pts);
        const ScreenwideKeyboardOverlay *keyboard =
            screenwide_keyboard_at(keyboards, keyboard_count, pts);
        while (next_camera_sample != NULL &&
               CMTimeCompare(
                   CMSampleBufferGetPresentationTimeStamp(next_camera_sample),
                   pts) <= 0) {
          if (camera_sample != NULL)
            CFRelease(camera_sample);
          camera_sample = next_camera_sample;
          next_camera_sample = [session->camera_output copyNextSampleBuffer];
        }
        CVPixelBufferRef destination = NULL;
        if (CVPixelBufferPoolCreatePixelBuffer(
                kCFAllocatorDefault, session->adaptor.pixelBufferPool,
                &destination) != kCVReturnSuccess ||
            destination == NULL) {
          CFRelease(screen_sample);
          error =
              [NSError errorWithDomain:@"ScreenwideGPUCompositor"
                                  code:2
                              userInfo:@{
                                NSLocalizedDescriptionKey :
                                    @"The GPU encoder ran out of video buffers"
                              }];
          break;
        }
        ScreenwideInflightFrame *frame = [session encodeScreen:screen_sample
                                                        camera:camera_sample
                                                   destination:destination
                                                        cursor:cursor
                                                      keyboard:keyboard
                                                    sourceTime:pts
                                                    outputTime:output_pts];
        [ring addObject:frame];
        if (ring.count > SCREENWIDE_GPU_INFLIGHT_FRAMES) {
          ScreenwideInflightFrame *oldest = ring.firstObject;
          [ring removeObjectAtIndex:0];
          ScreenwideDrainResult drained =
              screenwide_export_drain_inflight_frame(
                  oldest, session->adaptor, session->writer_input,
                  session->writer, context, should_cancel, progress, &primed,
                  session->source_frame_rate, &error);
          if (drained == ScreenwideDrainCancelled)
            cancelled = true;
          if (drained != ScreenwideDrainAppended)
            break;
        }
      }
    }
    // Flush whatever is still on the GPU, oldest first, so the tail of the
    // export keeps its source order.
    while (!cancelled && error == nil && ring.count > 0) {
      @autoreleasepool {
        ScreenwideInflightFrame *oldest = ring.firstObject;
        [ring removeObjectAtIndex:0];
        ScreenwideDrainResult drained = screenwide_export_drain_inflight_frame(
            oldest, session->adaptor, session->writer_input, session->writer,
            context, should_cancel, progress, &primed,
            session->source_frame_rate, &error);
        if (drained == ScreenwideDrainCancelled)
          cancelled = true;
        if (drained != ScreenwideDrainAppended)
          break;
      }
    }
    // A cancel or a writer failure abandons the rest of the ring. The
    // GPU still owns those textures, so wait for each command buffer before the
    // frames release their buffers.
    for (ScreenwideInflightFrame *abandoned in ring)
      [abandoned.command waitUntilCompleted];
    [ring removeAllObjects];
    if (camera_sample != NULL)
      CFRelease(camera_sample);
    if (next_camera_sample != NULL)
      CFRelease(next_camera_sample);
    CFRelease(session->texture_cache);
    if (cancelled) {
      [session->screen_reader cancelReading];
      if (session->camera_reader != nil)
        [session->camera_reader cancelReading];
      [session->writer cancelWriting];
      [[NSFileManager defaultManager] removeItemAtURL:session->output_url
                                                error:nil];
      return -1;
    }
    if (error != nil ||
        session->screen_reader.status == AVAssetReaderStatusFailed ||
        (session->camera_reader != nil &&
         session->camera_reader.status == AVAssetReaderStatusFailed)) {
      [session->writer cancelWriting];
      [[NSFileManager defaultManager] removeItemAtURL:session->output_url
                                                error:nil];
      return fail(error_text, error_capacity,
                  error.localizedDescription ?:
                    session->screen_reader.error.localizedDescription ?:
                    (session->camera_reader != nil ? session->camera_reader.error.localizedDescription : nil) ?:
                    @"The GPU compositor could not read the recording");
    }
    [session->writer_input markAsFinished];
    dispatch_semaphore_t finish_semaphore = dispatch_semaphore_create(0);
    [session->writer finishWritingWithCompletionHandler:^{
      dispatch_semaphore_signal(finish_semaphore);
    }];
    dispatch_semaphore_wait(finish_semaphore, DISPATCH_TIME_FOREVER);
    if (session->writer.status != AVAssetWriterStatusCompleted) {
      [[NSFileManager defaultManager] removeItemAtURL:session->output_url
                                                error:nil];
      return fail(error_text, error_capacity,
                  session->writer.error.localizedDescription
                      ?: @"The GPU encoder could not finish the recording");
    }
    return 1;
  }
}
