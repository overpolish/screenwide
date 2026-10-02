// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

#include <math.h>

SCREENWIDE_PREVIEW_PRIVATE void remember_workspace_transform(
    ScreenwidePreviewSurface *surface, double width, double height);
SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);
SCREENWIDE_PREVIEW_PRIVATE void present_in_transaction(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewView *view,
    id<MTLCommandBuffer> command, id<CAMetalDrawable> drawable);

/// Main thread only. Releases the workspace draw slot and re-runs the redraw
/// that was coalesced away while this one was in flight.
static void clear_workspace_draw_in_flight(ScreenwidePreviewSurface *surface) {
  [surface.workspaceLock lock];
  surface.workspaceDrawInFlight = NO;
  BOOL pending = surface.workspaceDrawPending;
  surface.workspaceDrawPending = NO;
  [surface.workspaceLock unlock];
  if (pending) redraw_workspace(surface);
}

/// Draws the OSC over the drawn workspace and presents both. Both draws were
/// submitted on the shared queue before `command`, so it runs after them.
static void present_workspace_drawable(ScreenwidePreviewSurface *surface,
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

static ScreenwideWorkspacePlacement workspace_placement(
    ScreenwidePreviewSurface *surface) {
  if (surface.editorBaseRects.count == 0)
    return (ScreenwideWorkspacePlacement){0};
  NSRect transformed = editor_frame(
      surface, surface.editorBaseRects[0].rectValue);
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  CGFloat top = surface.container.bounds.size.height - NSMaxY(transformed);
  return (ScreenwideWorkspacePlacement){
    (int32_t)llround(transformed.origin.x * scale),
    (int32_t)llround(top * scale),
    (uint32_t)MAX(llround(transformed.size.width * scale), 1),
    (uint32_t)MAX(llround(transformed.size.height * scale), 1),
  };
}

SCREENWIDE_PREVIEW_PRIVATE void begin_workspace_frame_resize(ScreenwidePreviewSurface *surface) {
  if (!surface.workspaceMode || surface.views.count == 0) return;
  remember_workspace_transform(surface, surface.workspaceNaturalWidth,
                               surface.workspaceNaturalHeight);
  surface.workspaceResizeNaturalWidth = surface.workspaceNaturalWidth;
  surface.workspaceResizeNaturalHeight = surface.workspaceNaturalHeight;
  screenwide_workspace_scene_begin_resize(surface.scene);
}

SCREENWIDE_PREVIEW_PRIVATE void update_workspace_frame_resize(
    ScreenwidePreviewSurface *surface, NSRect start, NSRect resized) {
  if (!surface.workspaceMode || surface.views.count == 0 ||
      start.size.width <= 0.0 || start.size.height <= 0.0) return;
  double originX = (resized.origin.x - start.origin.x) / start.size.width;
  double originY = (resized.origin.y - start.origin.y) / start.size.height;
  double width = resized.size.width / start.size.width;
  double height = resized.size.height / start.size.height;
  surface.workspaceNaturalWidth = surface.workspaceResizeNaturalWidth * width;
  surface.workspaceNaturalHeight = surface.workspaceResizeNaturalHeight * height;
  screenwide_workspace_scene_update_resize(surface.scene, originX, originY, width, height);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL update_workspace_auto_fit_move(
    ScreenwidePreviewSurface *surface, uint32_t selected_layer,
    double move_x, double move_y, NSRect start, NSRect resized) {
  if (!surface.workspaceMode || surface.views.count == 0 ||
      start.size.width <= 0.0 || start.size.height <= 0.0) return NO;
  double originX = (resized.origin.x - start.origin.x) / start.size.width;
  double originY = (resized.origin.y - start.origin.y) / start.size.height;
  double width = resized.size.width / start.size.width;
  double height = resized.size.height / start.size.height;
  surface.workspaceNaturalWidth = surface.workspaceResizeNaturalWidth * width;
  surface.workspaceNaturalHeight = surface.workspaceResizeNaturalHeight * height;
  return screenwide_workspace_scene_update_auto_fit_move(
             surface.scene, selected_layer, move_x, move_y, originX, originY, width,
             height) != 0;
}

SCREENWIDE_PREVIEW_PRIVATE void end_workspace_frame_resize(
    ScreenwidePreviewSurface *surface, BOOL commit) {
  if (!surface.workspaceMode || surface.views.count == 0) return;
  screenwide_workspace_scene_end_resize(surface.scene, commit ? 1 : 0);
  if (!commit) {
    surface.workspaceNaturalWidth = surface.workspaceResizeNaturalWidth;
    surface.workspaceNaturalHeight = surface.workspaceResizeNaturalHeight;
  } else {
    remember_workspace_transform(surface, surface.workspaceNaturalWidth,
                                 surface.workspaceNaturalHeight);
  }
}

/// Main thread only, with `workspaceLock` held. Each layer's placement in
/// drawable pixels: recording panes each where their own pane is laid out,
/// screenshot layers all on the one workspace canvas.
static NSMutableData *workspace_placements(ScreenwidePreviewSurface *surface) {
  ScreenwideWorkspacePlacement placement = workspace_placement(surface);
  NSMutableData *data = [NSMutableData
      dataWithLength:sizeof(placement) * surface.workspaceLayerCount];
  ScreenwideWorkspacePlacement *placements = data.mutableBytes;
  if (!surface.workspaceExplicitPlacements ||
      surface.workspacePlacements.length < data.length) {
    for (uint32_t index = 0; index < surface.workspaceLayerCount; index++)
      placements[index] = placement;
    return data;
  }
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  for (uint32_t index = 0; index < surface.workspaceLayerCount; index++) {
    uint32_t pane_index = surface.workspacePaneIndices[index].unsignedIntValue;
    if (pane_index >= surface.editorBaseRects.count ||
        ![surface.workspaceActivePaneIndices containsObject:@(pane_index)]) {
      placements[index] = (ScreenwideWorkspacePlacement){0};
      continue;
    }
    NSRect transformed = editor_frame(surface,
                                      surface.editorBaseRects[pane_index].rectValue);
    placements[index] = (ScreenwideWorkspacePlacement){
      (int32_t)llround(transformed.origin.x * scale),
      (int32_t)llround((surface.container.bounds.size.height - NSMaxY(transformed)) * scale),
      (uint32_t)MAX(llround(transformed.size.width * scale), 1),
      (uint32_t)MAX(llround(transformed.size.height * scale), 1),
    };
  }
  return data;
}

SCREENWIDE_PREVIEW_PRIVATE void redraw_workspace(ScreenwidePreviewSurface *surface) {
  if (!surface.workspaceMode || surface.workspaceLayerCount == 0 ||
      surface.views.count == 0 || !surface.workspaceHasPanes) return;
  ScreenwidePreviewView *workspace = surface.views[0];
  if (!workspace.active || workspace.hidden) return;
  [surface.workspaceLock lock];
  if (surface.workspaceDrawInFlight) {
    surface.workspaceDrawPending = YES;
    [surface.workspaceLock unlock];
    return;
  }
  surface.workspaceDrawInFlight = YES;
  BOOL drawn = NO;
  // A scene that cannot draw is checked before a drawable is taken: one
  // acquired here must be presented in this turn.
  if (screenwide_workspace_scene_ready(surface.scene, surface.workspaceLayerCount)) {
    id<CAMetalDrawable> drawable = [(CAMetalLayer *)workspace.layer nextDrawable];
    if (drawable != nil) {
      ScreenwideRegionMagnifier magnifier = surface.workspaceMagnifier;
      screenwide_workspace_scene_draw(surface.scene, (__bridge void *)drawable.texture,
                                      workspace_placements(surface).bytes,
                                      surface.workspaceLayerCount, &magnifier);
      present_workspace_drawable(surface, [surface.queue commandBuffer], drawable);
      drawn = YES;
    }
  }
  if (!drawn) {
    surface.workspaceDrawInFlight = NO;
    if (!surface.workspaceRedrawRetried) {
      surface.workspaceRedrawRetried = YES;
      dispatch_async(dispatch_get_main_queue(), ^{
        redraw_workspace(surface);
      });
    }
  } else {
    surface.workspaceRedrawRetried = NO;
  }
  [surface.workspaceLock unlock];
}

int screenwide_preview_surface_present_screenshot_workspace(void *handle,
                                                            uint32_t layer_count) {
  if (handle == NULL || layer_count == 0) return 0;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  if (!surface.workspaceMode || surface.views.count == 0) return 0;
  ScreenwidePreviewView *workspace = surface.views[0];
  // The pane is made active by the layout block queued on the main thread; a
  // present that arrives before it has run is reported as not staged so the
  // caller can come back once the pane exists.
  if (!workspace.active) return 0;
  [surface.workspaceLock lock];
  surface.workspaceLayerCount = layer_count;
  surface.workspaceRedrawRetried = NO;
  surface.workspaceExplicitPlacements = NO;
  surface.workspacePlacements = nil;
  surface.workspacePaneIndices = nil;
  BOOL drawInFlight = surface.workspaceDrawInFlight;
  if (drawInFlight)
    surface.workspaceDrawPending = YES;
  [surface.workspaceLock unlock];
  if (!drawInFlight) {
    dispatch_async(dispatch_get_main_queue(), ^{
      if (!surface.workspaceHasPanes) return;
      workspace.hidden = NO;
      redraw_workspace(surface);
    });
  }
  return 1;
}

/// Shows a staged recording scene whose layers carry explicit workspace
/// placements. Unlike the screenshot helper, placements are not collapsed to
/// the primary canvas: unbaked screen/camera panes remain independently
/// editable inside one native drawable.
int screenwide_preview_surface_present_recording_workspace(
    void *handle, const uint32_t *pane_indices,
    const ScreenwideWorkspacePlacement *layer_placements, uint32_t layer_count) {
  if (handle == NULL || pane_indices == NULL || layer_placements == NULL ||
      layer_count == 0) return 0;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  if (surface.views.count == 0) return 0;
  ScreenwidePreviewView *workspace = surface.views[0];
  [surface.workspaceLock lock];
  surface.workspaceLayerCount = layer_count;
  surface.workspaceRedrawRetried = NO;
  surface.workspaceExplicitPlacements = YES;
  NSMutableArray<NSNumber *> *paneIndices = [NSMutableArray arrayWithCapacity:layer_count];
  for (uint32_t index = 0; index < layer_count; index++)
    [paneIndices addObject:@(pane_indices[index])];
  surface.workspacePaneIndices = paneIndices;
  surface.workspacePlacements = [NSMutableData
      dataWithBytes:layer_placements
             length:sizeof(ScreenwideWorkspacePlacement) * layer_count];
  BOOL drawInFlight = surface.workspaceDrawInFlight;
  if (drawInFlight) surface.workspaceDrawPending = YES;
  [surface.workspaceLock unlock];
  if (!drawInFlight) {
    dispatch_async(dispatch_get_main_queue(), ^{
      // A frame decoded for a layout that has since dropped its panes is
      // staged but never shown: un-hiding here would put a stale picture
      // over the audio-only preview's ribbon.
      if (!surface.workspaceHasPanes) return;
      workspace.hidden = NO;
      redraw_workspace(surface);
    });
  }
  return 1;
}

int screenwide_preview_surface_redraw_workspace(void *handle) {
  if (handle == NULL) return 0;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  if (!surface.workspaceMode || surface.views.count == 0) return 0;
  // The draw reads the pane geometry that the layout setters now apply
  // asynchronously, so it has to queue behind them instead of racing them
  // from the caller's thread. The result still reports what it always did:
  // whether a workspace pane exists to draw into.
  on_main_async(^{
    redraw_workspace(surface);
  });
  return 1;
}
