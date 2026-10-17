// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// How a drawn workspace reaches the screen. The workspace layer presents
// with Core Animation transactions, and such a transaction waits for its
// drawable's GPU work, holding the main thread and with it the pointer, the
// selection chrome and the webview. So a redraw is composed off screen on
// the main thread, which only encodes, and a drawable is taken, filled and
// waited for on a queue of its own; the present then lands on the main
// thread with the GPU work already done. A redraw at a new drawable size
// still presents inline, beside the layout that resized the workspace.

#import <QuartzCore/CATransaction.h>

#import "recording_preview_surface_macos_private.h"

SCREENWIDE_PREVIEW_PRIVATE void present_in_transaction(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewView *view,
    id<MTLCommandBuffer> command, id<CAMetalDrawable> drawable);

/// Serial, so presents reach the main thread in the order they were drawn.
static dispatch_queue_t present_queue(void) {
  static dispatch_queue_t queue;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    queue = dispatch_queue_create(
        "app.screenwide.workspace-present",
        dispatch_queue_attr_make_with_qos_class(DISPATCH_QUEUE_SERIAL,
                                                QOS_CLASS_USER_INTERACTIVE, 0));
  });
  return queue;
}

SCREENWIDE_PREVIEW_PRIVATE id<MTLTexture> workspace_frame_texture(
    ScreenwidePreviewSurface *surface, CGSize size) {
  NSUInteger width = (NSUInteger)size.width, height = (NSUInteger)size.height;
  if (width == 0 || height == 0) return nil;
  id<MTLTexture> frame = surface.workspaceFrameTexture;
  if (frame != nil && frame.width == width && frame.height == height) return frame;
  MTLTextureDescriptor *descriptor =
      [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm
                                                         width:width
                                                        height:height
                                                     mipmapped:NO];
  descriptor.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
  descriptor.storageMode = MTLStorageModePrivate;
  frame = [surface.device newTextureWithDescriptor:descriptor];
  frame.label = @"Screenwide workspace frame";
  surface.workspaceFrameTexture = frame;
  return frame;
}

SCREENWIDE_PREVIEW_PRIVATE void present_workspace_drawable(ScreenwidePreviewSurface *surface,
                                                           id<MTLCommandBuffer> command,
                                                           id<CAMetalDrawable> drawable) {
  // Pixels and OSC land in one drawable, then use the same explicit Core
  // Animation transaction handoff as the proven multi-pane path. Direct
  // `presentDrawable` completed on the GPU quickly but could remain queued
  // for seconds before Core Animation displayed it.
  surface.workspaceEncodingTexture = drawable.texture;
  redraw_selection(surface);
  surface.workspaceEncodingTexture = nil;
  ScreenwidePreviewView *workspace = surface.views.firstObject;
  // SAME-TURN CONSTRAINT: a workspace redraw acquires its drawable and
  // encodes on the main thread, so the present MUST happen in that same
  // runloop turn. Deferring it - to this command buffer's completed handler,
  // or to the batch's group-notify block - leaves the turn holding an
  // acquired-but-unpresented drawable of a `presentsWithTransaction` layer;
  // the turn's closing Core Animation flush then blocks waiting for a present
  // that is itself queued behind that flush on the main queue, and gives up
  // only at its ~1s watchdog. That is the measured 1-second hang. So hand the
  // buffer to `present_in_transaction` unconditionally: batched or not, its
  // main-thread path commits, waits for scheduled and presents inline here.
  //
  // Registered before the commit that `present_in_transaction` performs -
  // Metal rejects completed handlers added after `commit`. This clear re-arms
  // `workspaceDrawPending` coalescing, so it must stay.
  [command addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
    dispatch_async(dispatch_get_main_queue(), ^{
      clear_workspace_draw_in_flight(surface);
    });
  }];
  present_in_transaction(surface, workspace, command, drawable);
}

/// Main thread. Lands a drawable filled off the main thread, its GPU work
/// done, unless the workspace went away or was resized meanwhile, and lets
/// the next redraw in.
static void land_workspace_drawable(ScreenwidePreviewSurface *surface,
                                    ScreenwidePreviewView *workspace,
                                    id<CAMetalDrawable> drawable) {
  BOOL shown = surface.workspaceMode && workspace.active && !workspace.hidden &&
      surface.views.firstObject == workspace;
  if (drawable != nil && shown) {
    [CATransaction begin];
    [CATransaction setDisableActions:YES];
    commit_frames_and_drawables(surface, @[ drawable ], @[ workspace ]);
    [CATransaction commit];
  } else if (shown) {
    // The drawable no longer fit the workspace: draw it again at its size.
    [surface.workspaceLock lock];
    surface.workspaceDrawPending = YES;
    [surface.workspaceLock unlock];
  }
  clear_workspace_draw_in_flight(surface);
}

SCREENWIDE_PREVIEW_PRIVATE void present_workspace_frame(ScreenwidePreviewSurface *surface,
                                                        ScreenwidePreviewView *workspace,
                                                        id<MTLTexture> frame) {
  surface.workspaceEncodingTexture = frame;
  redraw_selection(surface);
  surface.workspaceEncodingTexture = nil;
  // Committed after the scene and the chrome were submitted to the shared
  // queue, so it completes once the frame is whole.
  id<MTLCommandBuffer> composed = [surface.queue commandBuffer];
  composed.label = @"Screenwide workspace composed";
  [composed commit];
  CAMetalLayer *layer = (CAMetalLayer *)workspace.layer;
  id<MTLCommandQueue> queue = surface.queue;
  dispatch_async(present_queue(), ^{
    [composed waitUntilCompleted];
    // Taken here, off the main thread, so nothing on the main thread holds
    // a drawable it has not presented; see `present_workspace_drawable`.
    id<CAMetalDrawable> drawable = [layer nextDrawable];
    if (drawable != nil && (drawable.texture.width != frame.width ||
                            drawable.texture.height != frame.height))
      drawable = nil;
    if (drawable != nil) {
      id<MTLCommandBuffer> copy = [queue commandBuffer];
      copy.label = @"Screenwide workspace present";
      id<MTLBlitCommandEncoder> blit = [copy blitCommandEncoder];
      [blit copyFromTexture:frame toTexture:drawable.texture];
      [blit endEncoding];
      [copy commit];
      [copy waitUntilCompleted];
    }
    dispatch_async(dispatch_get_main_queue(), ^{
      land_workspace_drawable(surface, workspace, drawable);
    });
  });
}
