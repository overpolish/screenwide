// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once
#import <AVFoundation/AVFoundation.h>
#include <math.h>
#include <stdbool.h>

typedef bool (*ScreenwideShouldCancel)(void *context);
typedef void (*ScreenwideProgress)(void *context, uint64_t position_ms);

/// One frame handed to Rust to compose: its decoded NV12 screen, its BGRA
/// camera frame or NULL, the NV12 buffer it is drawn into, and its time in
/// the source. The twin of Rust's `ExportFrame`.
typedef struct {
  CVPixelBufferRef screen;
  CVPixelBufferRef camera;
  CVPixelBufferRef destination;
  uint64_t source_us;
  float frame_ms;
} ScreenwideExportFrame;

/// Submits the frame's GPU work and answers a token for `ScreenwideWaitFrame`.
typedef bool (*ScreenwideComposeFrame)(void *context, const ScreenwideExportFrame *frame,
                                       uint64_t *token);
/// Blocks until the GPU has finished the frame `token` names.
typedef bool (*ScreenwideWaitFrame)(void *context, uint64_t token);
/// Starts the encoder frames are streamed to at `frame_rate`, answering the
/// descriptor each frame's planes are written to, or -1 to encode them with
/// AVFoundation instead.
typedef int (*ScreenwideOpenStream)(void *context, uint32_t frame_rate);

typedef struct {
  uint64_t output_start_us;
  uint64_t source_end_us;
  uint64_t source_start_us;
  double playback_rate;
} ScreenwideTimelineRange;

static inline int fail(char *error, size_t capacity, NSString *message) {
  if (error != NULL && capacity > 0) {
    snprintf(error, capacity, "%s", (message ?: @"The video export failed").UTF8String);
  }
  return 0;
}

/// One frame whose GPU work is submitted but not yet handed to the writer.
/// It holds the output buffer the GPU draws into and the sample buffers it
/// reads from until that work completes.
@interface ScreenwideInflightFrame : NSObject
@property(nonatomic) uint64_t token;
@property(nonatomic) CMTime presentation;
@property(nonatomic) CVPixelBufferRef destination;
@property(nonatomic) CMSampleBufferRef screenSample;
@property(nonatomic) CMSampleBufferRef cameraSample;
/// The same picture shown again at `presentation`, once this frame's GPU
/// work is done.
- (instancetype)repeatedAt:(CMTime)presentation;
@end

typedef enum {
  ScreenwideDrainAppended,
  ScreenwideDrainCancelled,
  ScreenwideDrainFailed,
} ScreenwideDrainResult;

/// What the loop calls back into, borrowed for the export's length.
typedef struct {
  void *context;
  ScreenwideShouldCancel should_cancel;
  ScreenwideProgress progress;
  ScreenwideWaitFrame wait;
} ScreenwideExportCallbacks;

__attribute__((visibility("hidden"))) ScreenwideDrainResult screenwide_export_drain_inflight_frame(
    ScreenwideInflightFrame *frame, AVAssetWriterInputPixelBufferAdaptor *adaptor,
    AVAssetWriterInput *input, AVAssetWriter *writer, const ScreenwideExportCallbacks *callbacks,
    BOOL *primed, float source_frame_rate, NSError **error);

/// Waits for the oldest in-flight frame and writes its planes to `fd`, each
/// plane's rows packed, through `staging`, which holds one whole frame.
__attribute__((visibility("hidden"))) ScreenwideDrainResult screenwide_export_stream_inflight_frame(
    ScreenwideInflightFrame *frame, int fd, NSMutableData *staging,
    const ScreenwideExportCallbacks *callbacks, NSError **error);

/// The reader and writer one export runs between.
@interface ScreenwideVideoExport : NSObject {
@public
  AVAssetReader *screen_reader;
  AVAssetReader *camera_reader;
  AVAssetReaderTrackOutput *screen_output;
  AVAssetReaderTrackOutput *camera_output;
  AVAssetTrack *screen_track;
  AVAssetTrack *camera_track;
  /// Readers a reordered timeline left behind going back in the source,
  /// cancelled but held until the export ends: frames still in flight hold
  /// their buffers.
  NSMutableArray<AVAssetReader *> *retired_readers;
  AVAssetWriter *writer;
  AVAssetWriterInput *writer_input;
  AVAssetWriterInputPixelBufferAdaptor *adaptor;
  NSURL *output_url;
  /// Where frames are written when an encoder outside AVFoundation takes
  /// them, or -1; then the pool they are drawn into and the frame each is
  /// packed in on its way there.
  int stream_fd;
  CVPixelBufferPoolRef stream_pool;
  NSMutableData *stream_staging;
  float source_frame_rate;
  uint64_t source_duration_us;
  uint32_t output_width;
  uint32_t output_height;
}
@end

@interface ScreenwideVideoExport (Setup)
- (int)prepareScreen:(const char *)screen_path
              camera:(const char *)camera_path
              output:(const char *)output_path
             bitrate:(uint64_t)bitrate
              stream:(ScreenwideOpenStream)open_stream
             context:(void *)context
           errorText:(char *)error_text
       errorCapacity:(size_t)error_capacity;
/// Starts reading both tracks again from `time`, for a timeline that plays
/// an earlier stretch of the source after a later one: a reader only moves
/// forward.
- (BOOL)restartReadersAt:(CMTime)time error:(NSError **)error;
@end
