// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// Evaluate source-time clips once per exported frame. Bind the same prepared
/// geometry used by screenshot export and native video preview.
static void screenwide_export_annotations(ScreenwideVideoExport *session,
    id<MTLCommandBuffer> command, id<MTLTexture> luma, id<MTLTexture> chroma,
    uint32_t source_width, uint32_t source_height, uint64_t source_ms, uint32_t above) {
  ScreenwideAnnotations marks = {0};
  uint32_t active_count = 0;
  for (uint32_t i = 0; i < session->annotation_count; i++) {
    const ScreenwideTimedAnnotation *clip = &session->annotations[i];
    if (clip->start_ms <= source_ms && source_ms < clip->end_ms) {
      if (active_count++ == SCREENWIDE_MAX_ANNOTATIONS) break;
      if (clip->annotation.above_camera == above)
        marks.items[marks.count++] = clip->annotation;
    }
  }
  if (marks.count == 0) return;
  for (int plane = 0; plane < 2; plane++) {
    id<MTLTexture> texture = plane == 0 ? luma : chroma;
    id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
    [encoder setComputePipelineState:plane == 0 ? session->annotation_luma_pipeline : session->annotation_chroma_pipeline];
    [encoder setTexture:texture atIndex:0];
    [encoder setBytes:session->canvas length:sizeof(*session->canvas) atIndex:0];
    screenwide_bind_annotations(encoder, &marks, session->canvas, source_width, source_height);
    [encoder setBytes:&above length:sizeof(above) atIndex:14];
    [encoder dispatchThreads:MTLSizeMake(texture.width, texture.height, 1)
        threadsPerThreadgroup:MTLSizeMake(16, 16, 1)];
    [encoder endEncoding];
  }
}
