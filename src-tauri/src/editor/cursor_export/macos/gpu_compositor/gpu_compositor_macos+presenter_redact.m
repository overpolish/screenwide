// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_presenter_private.h"
#import "gpu_compositor_macos_redact.h"

// A redaction is applied to the retained source rather than drawn over the
// canvas: the canvas, the crop ghost and the magnifier all sample the source,
// and each of them must find the covered pixels already gone. The original
// stays untouched beside the copy so a redaction that moves, or is removed,
// uncovers exactly the pixels that were there.
//
// Staging a scene only notes that a copy is due. The copy is made at the
// start of the command buffer that draws from it, where the order between
// its passes is Metal's to keep; made in a command buffer of its own, a draw
// committed after it could still sample the copy before it was written, and
// the preview would stay on the redaction it showed before. Each change also
// writes a fresh copy, so no frame still in flight sees its source rewritten.

@implementation ScreenwideRedactedSource
@end

void screenwide_presenter_redact_source(ScreenwideStillPresenter *presenter,
                                        id<MTLBuffer> source,
                                        const ScreenwideWorkspaceLayer *layer) {
  NSNumber *token = @(layer->source_token);
  NSData *redactions = screenwide_redactions(&layer->annotations, layer->source_width,
                                             layer->source_height);
  if (redactions.length == 0 || source == nil) {
    [presenter.workspaceRedactedSources removeObjectForKey:token];
    return;
  }
  ScreenwideRedactedSource *redacted = presenter.workspaceRedactedSources[token];
  if (redacted != nil && redacted.original == source &&
      [redacted.redactions isEqualToData:redactions])
    return;
  redacted = [ScreenwideRedactedSource new];
  redacted.pixels = [presenter.device newBufferWithLength:source.length
                                                  options:MTLResourceStorageModePrivate];
  if (redacted.pixels == nil) {
    [presenter.workspaceRedactedSources removeObjectForKey:token];
    return;
  }
  redacted.original = source;
  redacted.redactions = redactions;
  redacted.pending = YES;
  presenter.workspaceRedactedSources[token] = redacted;
}

void screenwide_presenter_apply_redactions(ScreenwideStillPresenter *presenter,
                                           id<MTLCommandBuffer> command) {
  for (ScreenwideRedactedSource *redacted in presenter.workspaceRedactedSources.allValues) {
    if (!redacted.pending) continue;
    id<MTLBlitCommandEncoder> blit = [command blitCommandEncoder];
    [blit copyFromBuffer:redacted.original sourceOffset:0 toBuffer:redacted.pixels
       destinationOffset:0 size:redacted.original.length];
    [blit endEncoding];
    screenwide_encode_redactions(command, presenter.redactPipelines, redacted.pixels,
                                 redacted.redactions);
    redacted.pending = NO;
  }
}

id<MTLBuffer> screenwide_presenter_visible_source(ScreenwideStillPresenter *presenter,
                                                  uint64_t token) {
  NSNumber *key = @(token);
  return presenter.workspaceRedactedSources[key].pixels ?: presenter.workspaceSources[key];
}
