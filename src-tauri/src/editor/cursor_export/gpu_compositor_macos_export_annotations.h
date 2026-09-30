// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// Evaluate source-time clips once per exported frame, their arrivals timed at
/// `output_ms`, the frame's place on the edited timeline. Bind the same
/// prepared geometry used by screenshot export and native video preview.
/// `source_luma` and `source_chroma` are the screen frame after its
/// redactions, which a magnifier enlarges. `redo` is set for the pass run
/// again over the screen layer redrawn above a camera sent behind it.
///
/// `cursor` is the frame's cursor on the screen layer's pass and NULL on the
/// camera's. The screen layer's cursor is shaded and hidden by what acts on
/// the picture, so this pass draws it whenever any annotation shows - every
/// spotlight blurs it, on either layer - and says so; where none shows,
/// nothing is drawn and the caller draws the cursor on its own.
static BOOL screenwide_export_annotations(ScreenwideVideoExport *session,
    id<MTLCommandBuffer> command, id<MTLTexture> luma, id<MTLTexture> chroma,
    id<MTLTexture> source_luma, id<MTLTexture> source_chroma, uint64_t source_ms,
    uint64_t output_ms, uint32_t above, uint32_t redo, const ScreenwideGpuCursor *cursor) {
  uint32_t source_width = (uint32_t)source_luma.width;
  uint32_t source_height = (uint32_t)source_luma.height;
  // A frame shows at most every clip; the list is sized to that, on the heap.
  NSMutableData *items = [NSMutableData
      dataWithLength:MAX(session->annotation_count, 1u) * sizeof(ScreenwideAnnotation)];
  ScreenwideAnnotation *showing = items.mutableBytes;
  ScreenwideAnnotations annotations = {.items = showing};
  // Every clip's record indexes the one set of side buffers the export was
  // given, so the frame points at them and the offsets stay as they are.
  if (session->annotation_data != NULL) annotations.data = *session->annotation_data;
  uint32_t on_layer = 0;
  for (uint32_t i = 0; i < session->annotation_count; i++) {
    const ScreenwideTimedAnnotation *clip = &session->annotations[i];
    if (clip->start_ms <= source_ms && source_ms < clip->end_ms) {
      if (clip->annotation.above_camera == above) on_layer += 1;
      ScreenwideAnnotation *annotation = &showing[annotations.count++];
      *annotation = clip->annotation;
      // seek lands on exactly the frame a play-through drew. The blur's lead
      // is one source frame of travel at the rate this export encodes.
      screenwide_annotation_reveal_window(
          screenwide_timed_reveal_elapsed_ms(clip, output_ms),
          (float)(clip->reveal_end_ms - clip->reveal_start_ms),
          session->source_frame_rate > 0 ? 1000.0f / session->source_frame_rate : 0,
          annotation->animated, annotation->kind, clip->path_ms, clip->joins,
          clip->blur_share, &annotation->reveal);
    }
  }
  if (on_layer == 0 && (cursor == NULL || annotations.count == 0)) return NO;
  ScreenwideOverlayUniforms pointer = screenwide_export_cursor_uniforms(
      session->cursor_artwork, cursor, session->artworks, session->artwork_count,
      session->canvas, session->output_width, session->output_height);
  // Highlights and spotlights change what is under them, so the pass reads
  // both planes at once and draws the whole layer over them.
  id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
  [encoder setComputePipelineState:session->annotation_pipeline];
  [encoder setTexture:luma atIndex:0];
  [encoder setTexture:chroma atIndex:1];
  [encoder setTexture:source_luma atIndex:2];
  [encoder setTexture:source_chroma atIndex:3];
  [encoder setTexture:session->cursor_artwork ?: session->cursor_placeholder atIndex:4];
  [encoder setBytes:session->canvas length:sizeof(*session->canvas) atIndex:0];
  screenwide_bind_annotations(encoder, &annotations, session->canvas, source_width,
                              source_height, 1.0f);
  [encoder setBytes:&above length:sizeof(above) atIndex:14];
  [encoder setBytes:&redo length:sizeof(redo) atIndex:20];
  [encoder setBytes:&pointer length:sizeof(pointer) atIndex:22];
  [encoder dispatchThreads:MTLSizeMake(chroma.width, chroma.height, 1)
      threadsPerThreadgroup:MTLSizeMake(16, 16, 1)];
  [encoder endEncoding];
  return cursor != NULL;
}
