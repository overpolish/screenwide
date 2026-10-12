// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "video_export.h"
#import "video_export_cadence.h"

/// How many composited frames may sit on the GPU while the loop keeps
/// decoding. A four second sample of a fully serial loop (decode ->
/// composite -> block on the GPU -> poll the encoder -> append) spent 63% of
/// its time sleeping on `readyForMoreMediaData` and 34% waiting on the GPU,
/// which put a 3494x2260 export at 0.4x realtime: every stage idled while
/// another worked. Keeping three frames in flight lets the encoder chew on
/// frame K while the GPU composites K+1 and the reader decodes K+2, so the two
/// waits overlap with real work instead of each other. Three is enough to
/// cover both stalls (the deepest measured stall was one frame of encoder
/// backpressure) without holding many 4K frame buffers - the ring costs this
/// many pool buffers plus the one being encoded, and the reader's own sample
/// buffers stay retained just as long.
#define SCREENWIDE_INFLIGHT_FRAMES 3

@implementation ScreenwideVideoExport
@end

static NSError *loop_error(NSInteger code, NSString *message) {
  return [NSError errorWithDomain:@"ScreenwideVideoExport"
                             code:code
                         userInfo:@{NSLocalizedDescriptionKey : message}];
}

/// Advances `current` past every sample at or before `pts`, keeping the next
/// one read ahead in `next`.
static void advance(AVAssetReaderTrackOutput *output, CMSampleBufferRef *current,
                    CMSampleBufferRef *next, CMTime pts) {
  while (*next != NULL &&
         CMTimeCompare(CMSampleBufferGetPresentationTimeStamp(*next), pts) <= 0) {
    if (*current != NULL) CFRelease(*current);
    *current = *next;
    *next = [output copyNextSampleBuffer];
  }
}

int screenwide_video_export(const char *screen_path, const char *camera_path,
                            const char *output_path, const ScreenwideTimelineRange *timeline_ranges,
                            uint32_t timeline_range_count, uint32_t output_width,
                            uint32_t output_height, uint64_t bitrate, void *context,
                            ScreenwideShouldCancel should_cancel, ScreenwideProgress progress,
                            ScreenwideComposeFrame compose, ScreenwideWaitFrame wait,
                            char *error_text, size_t error_capacity) {
  @autoreleasepool {
    ScreenwideVideoExport *session = [ScreenwideVideoExport new];
    session->output_width = output_width;
    session->output_height = output_height;
    if (![session prepareScreen:screen_path
                         camera:camera_path
                         output:output_path
                        bitrate:bitrate
                      errorText:error_text
                  errorCapacity:error_capacity])
      return 0;
    if (![session->screen_reader startReading] ||
        (session->camera_reader != nil && ![session->camera_reader startReading]) ||
        ![session->writer startWriting]) {
      return fail(error_text, error_capacity,
                  session->screen_reader.error.localizedDescription
                      ?: (session->camera_reader != nil
                              ? session->camera_reader.error.localizedDescription
                              : nil)
                      ?: session->writer.error.localizedDescription
                      ?: @"The video export could not be started");
    }
    [session->writer startSessionAtSourceTime:kCMTimeZero];
    ScreenwideExportCallbacks callbacks = {context, should_cancel, progress, wait};
    NSError *error = nil;
    BOOL primed = NO;
    // Frames whose GPU work is submitted but not yet appended, oldest first.
    NSMutableArray<ScreenwideInflightFrame *> *ring =
        [NSMutableArray arrayWithCapacity:SCREENWIDE_INFLIGHT_FRAMES + 1];
    CMSampleBufferRef camera_sample = NULL;
    CMSampleBufferRef next_camera_sample =
        session->camera_output == nil ? NULL : [session->camera_output copyNextSampleBuffer];
    bool cancelled = false;
    CMSampleBufferRef screen_sample = [session->screen_output copyNextSampleBuffer];
    CMSampleBufferRef next_screen_sample = [session->screen_output copyNextSampleBuffer];
    uint64_t duration_us =
        export_duration_us(session->source_duration_us, timeline_ranges, timeline_range_count);
    int32_t fps = (int32_t)llround(session->source_frame_rate);
    for (int64_t index = 0; screen_sample != NULL; index++) {
      CMTime output_pts = CMTimeMake(index, fps);
      if (CMTimeGetSeconds(output_pts) * 1000000.0 >= duration_us) break;
      @autoreleasepool {
        if (should_cancel(context)) {
          cancelled = true;
          break;
        }
        CMTime pts = export_source_time(output_pts, timeline_ranges, timeline_range_count);
        if (!CMTIME_IS_VALID(pts)) continue;
        advance(session->screen_output, &screen_sample, &next_screen_sample, pts);
        advance(session->camera_output, &camera_sample, &next_camera_sample, pts);
        CVPixelBufferRef destination = NULL;
        if (CVPixelBufferPoolCreatePixelBuffer(kCFAllocatorDefault,
                                               session->adaptor.pixelBufferPool,
                                               &destination) != kCVReturnSuccess ||
            destination == NULL) {
          error = loop_error(2, @"The encoder ran out of video buffers");
          break;
        }
        ScreenwideExportFrame request = {
            .screen = CMSampleBufferGetImageBuffer(screen_sample),
            .camera = camera_sample == NULL ? NULL : CMSampleBufferGetImageBuffer(camera_sample),
            .destination = destination,
            .source_us = (uint64_t)llround(MAX(CMTimeGetSeconds(pts), 0.0) * 1000000.0),
            .frame_ms = 1000.0f / session->source_frame_rate,
        };
        uint64_t token = 0;
        if (!compose(context, &request, &token)) {
          CVPixelBufferRelease(destination);
          error = loop_error(3, @"A video frame could not be composed");
          break;
        }
        ScreenwideInflightFrame *frame = [ScreenwideInflightFrame new];
        frame.token = token;
        frame.presentation = output_pts;
        frame.destination = destination;
        frame.screenSample = (CMSampleBufferRef)CFRetain(screen_sample);
        frame.cameraSample =
            camera_sample == NULL ? NULL : (CMSampleBufferRef)CFRetain(camera_sample);
        [ring addObject:frame];
        if (ring.count > SCREENWIDE_INFLIGHT_FRAMES) {
          ScreenwideInflightFrame *oldest = ring.firstObject;
          [ring removeObjectAtIndex:0];
          ScreenwideDrainResult drained = screenwide_export_drain_inflight_frame(
              oldest, session->adaptor, session->writer_input, session->writer, &callbacks,
              &primed, session->source_frame_rate, &error);
          if (drained == ScreenwideDrainCancelled) cancelled = true;
          if (drained != ScreenwideDrainAppended) break;
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
            oldest, session->adaptor, session->writer_input, session->writer, &callbacks, &primed,
            session->source_frame_rate, &error);
        if (drained == ScreenwideDrainCancelled) cancelled = true;
        if (drained != ScreenwideDrainAppended) break;
      }
    }
    // A cancel or a failure abandons the rest of the ring. The GPU may still
    // be drawing into those buffers, so the newest is waited for, and every
    // one before it with it, before the frames release them.
    if (ring.count > 0) wait(context, ring.lastObject.token);
    [ring removeAllObjects];
    if (screen_sample != NULL) CFRelease(screen_sample);
    if (next_screen_sample != NULL) CFRelease(next_screen_sample);
    if (camera_sample != NULL) CFRelease(camera_sample);
    if (next_camera_sample != NULL) CFRelease(next_camera_sample);
    if (cancelled) {
      [session->screen_reader cancelReading];
      if (session->camera_reader != nil) [session->camera_reader cancelReading];
      [session->writer cancelWriting];
      [[NSFileManager defaultManager] removeItemAtURL:session->output_url error:nil];
      return -1;
    }
    if (error != nil || session->screen_reader.status == AVAssetReaderStatusFailed ||
        (session->camera_reader != nil &&
         session->camera_reader.status == AVAssetReaderStatusFailed)) {
      [session->writer cancelWriting];
      [[NSFileManager defaultManager] removeItemAtURL:session->output_url error:nil];
      return fail(error_text, error_capacity,
                  error.localizedDescription
                      ?: session->screen_reader.error.localizedDescription
                      ?: (session->camera_reader != nil
                              ? session->camera_reader.error.localizedDescription
                              : nil)
                      ?: @"The video export could not read the recording");
    }
    [session->writer_input markAsFinished];
    dispatch_semaphore_t finish_semaphore = dispatch_semaphore_create(0);
    [session->writer finishWritingWithCompletionHandler:^{
      dispatch_semaphore_signal(finish_semaphore);
    }];
    dispatch_semaphore_wait(finish_semaphore, DISPATCH_TIME_FOREVER);
    if (session->writer.status != AVAssetWriterStatusCompleted) {
      [[NSFileManager defaultManager] removeItemAtURL:session->output_url error:nil];
      return fail(error_text, error_capacity,
                  session->writer.error.localizedDescription
                      ?: @"The encoder could not finish the recording");
    }
    return 1;
  }
}
