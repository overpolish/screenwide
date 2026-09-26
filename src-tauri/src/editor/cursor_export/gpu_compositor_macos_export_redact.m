// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_export_private.h"
#import "gpu_compositor_macos_redact.h"

/// A private copy of `texture` the redaction passes can write.
static id<MTLTexture> writable_copy(id<MTLCommandBuffer> command, id<MTLTexture> texture) {
  MTLTextureDescriptor *descriptor =
      [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:texture.pixelFormat
                                                         width:texture.width
                                                        height:texture.height
                                                     mipmapped:NO];
  descriptor.storageMode = MTLStorageModePrivate;
  descriptor.usage = MTLTextureUsageShaderRead | MTLTextureUsageShaderWrite;
  id<MTLTexture> copy = [texture.device newTextureWithDescriptor:descriptor];
  if (copy == nil) return nil;
  id<MTLBlitCommandEncoder> blit = [command blitCommandEncoder];
  [blit copyFromTexture:texture toTexture:copy];
  [blit endEncoding];
  return copy;
}

BOOL screenwide_export_redact_frame(ScreenwideVideoExport *session,
                                    id<MTLCommandBuffer> command,
                                    id<MTLTexture> __strong *luma,
                                    id<MTLTexture> __strong *chroma, uint64_t source_ms) {
  if (session->annotation_count == 0 || session->redact_pipelines == nil) return YES;
  NSMutableData *items =
      [NSMutableData dataWithLength:session->annotation_count * sizeof(ScreenwideAnnotation)];
  ScreenwideAnnotation *showing = items.mutableBytes;
  ScreenwideAnnotations annotations = {.items = showing};
  if (session->annotation_data != NULL) annotations.data = *session->annotation_data;
  for (uint32_t index = 0; index < session->annotation_count; index++) {
    const ScreenwideTimedAnnotation *clip = &session->annotations[index];
    if (clip->annotation.kind != SCREENWIDE_ANNOTATION_REDACT) continue;
    if (clip->start_ms > source_ms || source_ms >= clip->end_ms) continue;
    ScreenwideAnnotation *annotation = &showing[annotations.count++];
    *annotation = clip->annotation;
    // A pinned clip's records each arrive over the stretch they show in, and
    // all read the surface from their clip's own start.
    float elapsed_ms = (float)(source_ms - clip->clip_start_ms);
    screenwide_annotation_reveal_window(
        (float)(source_ms - clip->reveal_start_ms),
        (float)(clip->reveal_end_ms - clip->reveal_start_ms),
        session->source_frame_rate > 0 ? 1000.0f / session->source_frame_rate : 0,
        annotation->animated, annotation->kind, &annotation->reveal);
    // The surface the ring round the box settles on at this frame, from the
    // timeline worked out ahead for the whole clip.
    uint64_t first = (uint64_t)annotation->p1[0], count = (uint64_t)annotation->p1[1];
    if ((annotation->flags & SCREENWIDE_ANNOTATION_FLAG_SURFACES) != 0 &&
        annotations.data.points != NULL && first + count <= annotations.data.point_count) {
      float surface[3];
      if (screenwide_redaction_surface_at(annotations.data.points + first, (uint32_t)count,
                                          elapsed_ms, surface))
        memcpy(annotation->color, surface, sizeof(surface));
    }
  }
  if (annotations.count == 0) return YES;
  NSData *redactions =
      screenwide_redactions(&annotations, (uint32_t)(*luma).width, (uint32_t)(*luma).height);
  if (redactions.length == 0) return YES;
  // The decoded frame belongs to the reader, and the canvas samples only
  // these copies of it, so no pass after this one can reach what they hide.
  id<MTLTexture> redacted_luma = writable_copy(command, *luma);
  id<MTLTexture> redacted_chroma = writable_copy(command, *chroma);
  if (redacted_luma == nil || redacted_chroma == nil) return NO;
  screenwide_encode_video_redactions(command, session->redact_pipelines, redacted_luma,
                                     redacted_chroma, redactions);
  *luma = redacted_luma;
  *chroma = redacted_chroma;
  return YES;
}
