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

/// The reader and writer one export runs between.
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
           errorText:(char *)error_text
       errorCapacity:(size_t)error_capacity;
@end
