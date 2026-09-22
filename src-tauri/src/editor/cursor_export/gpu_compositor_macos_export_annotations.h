// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// Evaluate source-time clips once per exported frame. Bind the same prepared
/// geometry used by screenshot export and native video preview.
static void screenwide_export_annotations(ScreenwideVideoExport *session,
    id<MTLCommandBuffer> command, id<MTLTexture> luma, id<MTLTexture> chroma,
    uint32_t source_width, uint32_t source_height, uint64_t source_ms, uint32_t above) {
  // A frame shows at most every clip; the list is sized to that, on the heap.
  NSMutableData *items = [NSMutableData
      dataWithLength:MAX(session->annotation_count, 1u) * sizeof(ScreenwideAnnotation)];
  ScreenwideAnnotation *showing = items.mutableBytes;
  ScreenwideAnnotations annotations = {.items = showing};
  // Every clip's record indexes the one set of side buffers the export was
  // given, so the frame points at them and the offsets stay as they are.
  if (session->annotation_data != NULL) annotations.data = *session->annotation_data;
  for (uint32_t i = 0; i < session->annotation_count; i++) {
    const ScreenwideTimedAnnotation *clip = &session->annotations[i];
    if (clip->start_ms <= source_ms && source_ms < clip->end_ms) {
      if (clip->annotation.above_camera != above) continue;
      ScreenwideAnnotation *annotation = &showing[annotations.count++];
      *annotation = clip->annotation;
      // seek lands on exactly the frame a play-through drew. The blur's lead
      // is one source frame of travel at the rate this export encodes.
      screenwide_annotation_reveal_window(
          (float)(source_ms - clip->start_ms),
          (float)(clip->end_ms - clip->start_ms),
          session->source_frame_rate > 0 ? 1000.0f / session->source_frame_rate : 0,
          annotation->animated, annotation->kind, &annotation->reveal);
    }
  }
  if (annotations.count == 0) return;
  for (int plane = 0; plane < 2; plane++) {
    id<MTLTexture> texture = plane == 0 ? luma : chroma;
    id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
    [encoder setComputePipelineState:plane == 0 ? session->annotation_luma_pipeline : session->annotation_chroma_pipeline];
    [encoder setTexture:texture atIndex:0];
    [encoder setBytes:session->canvas length:sizeof(*session->canvas) atIndex:0];
    screenwide_bind_annotations(encoder, &annotations, session->canvas, source_width, source_height);
    [encoder setBytes:&above length:sizeof(above) atIndex:14];
    [encoder dispatchThreads:MTLSizeMake(texture.width, texture.height, 1)
        threadsPerThreadgroup:MTLSizeMake(16, 16, 1)];
    [encoder endEncoding];
  }
}
