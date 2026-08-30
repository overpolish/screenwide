// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_controls.h"
#import "recording_preview_surface_macos_private.h"
#import <QuartzCore/CATransaction.h>
#include <math.h>

_Static_assert(sizeof(ScreenwideOscControlSpec) == 40,
               "OSC control specs must match the Rust C ABI");
_Static_assert(sizeof(ScreenwideOscControlUpdate) == 4,
               "OSC control updates must match the Rust C ABI");
_Static_assert(sizeof(ScreenwideOscControlVisual) == 32,
               "OSC control visuals must match the Rust C ABI");

static void schedule_selection_action_frame(ScreenwidePreviewSurface *surface,
                                             uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
    if (surface.selectionActionAnimationRevision != revision) return;
    redraw_selection(surface);
    if (screenwide_osc_control_group_is_animating(
            surface.selectionActionControls)) {
      schedule_selection_action_frame(surface, revision);
    }
  });
}

static void apply_selection_action_update(
    ScreenwidePreviewSurface *surface, ScreenwideOscControlUpdate update) {
  if (!update.changed) return;
  redraw_selection(surface);
  if (update.animating) {
    uint64_t revision = ++surface.selectionActionAnimationRevision;
    schedule_selection_action_frame(surface, revision);
  }
}

SCREENWIDE_PREVIEW_PRIVATE void selection_action_layout(
    ScreenwidePreviewSurface *surface) {
  ScreenwideOscControlSpec specs[2];
  size_t count = 0;
  if (surface.selectionActionOperation != 0 &&
      !NSIsEmptyRect(surface.selectionActionRect)) {
    NSRect rect = surface.selectionActionRect;
    specs[count++] = (ScreenwideOscControlSpec){
        NSMinX(rect), NSMinY(rect), NSWidth(rect), NSHeight(rect),
        0, 0, 0, 0, 0};
  }
  if (surface.selectionActionOperation != 0 &&
      !NSIsEmptyRect(surface.selectionSecondaryActionRect)) {
    NSRect rect = surface.selectionSecondaryActionRect;
    specs[count++] = (ScreenwideOscControlSpec){
        NSMinX(rect), NSMinY(rect), NSWidth(rect), NSHeight(rect),
        0, 0, 0, 0, 0};
  }
  screenwide_osc_control_group_layout(surface.selectionActionControls,
                                      specs, count);
}

SCREENWIDE_PREVIEW_PRIVATE void selection_action_material_layout(
    ScreenwidePreviewSurface *surface) {
  ScreenwideOscControlMetrics metrics = screenwide_osc_control_metrics(0, 0);
  NSRect rects[2] = {surface.selectionActionRect,
                     surface.selectionSecondaryActionRect};
  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  for (NSUInteger index = 0;
       index < surface.selectionActionSurfaces.count; index++) {
    ScreenwideOscMaterialSurfaceView *material =
        surface.selectionActionSurfaces[index];
    NSRect rect = index < 2 ? rects[index] : NSZeroRect;
    BOOL visible = surface.selectionActionOperation != 0 &&
                   !NSIsEmptyRect(rect);
    material.hidden = !visible;
    if (!visible) continue;
    material.frame = NSMakeRect(
        rect.origin.x,
        surface.selectionActionMaterialContainer.bounds.size.height -
            NSMaxY(rect),
        rect.size.width, rect.size.height);
    material.layer.cornerRadius = metrics.radius;
    material.contentView.frame = material.bounds;
  }
  surface.selectionActionMaterialContainer.hidden =
      surface.selectionActionOperation == 0 ||
      (NSIsEmptyRect(surface.selectionActionRect) &&
       NSIsEmptyRect(surface.selectionSecondaryActionRect));
  [CATransaction commit];
}

static uint64_t action_visual_hash(uint64_t hash, const void *bytes,
                                   size_t length) {
  const uint8_t *cursor = bytes;
  for (size_t index = 0; index < length; index++) {
    hash ^= cursor[index];
    hash *= UINT64_C(1099511628211);
  }
  return hash;
}

SCREENWIDE_PREVIEW_PRIVATE void selection_action_render_surfaces(
    ScreenwidePreviewSurface *surface, CGFloat scale, uint32_t lightMode) {
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(lightMode);
  selection_action_fills(surface, lightMode, state.action_fills);
  NSRect rects[2] = {surface.selectionActionRect,
                     surface.selectionSecondaryActionRect};
  NSSize labelSizes[2] = {surface.selectionLabelSize,
                          surface.selectionSecondaryLabelSize};
  id<MTLTexture> labels[2] = {
      surface.selectionLabelTexture ?: surface.selectionLabelPlaceholder,
      surface.selectionSecondaryLabelTexture ?:
          surface.selectionLabelPlaceholder};
  NSString *texts[2] = {surface.selectionLabelText,
                         surface.selectionSecondaryLabelText};
  for (NSUInteger index = 0;
       index < surface.selectionActionSurfaces.count && index < 2; index++) {
    ScreenwideOscMaterialSurfaceView *control =
        surface.selectionActionSurfaces[index];
    NSRect rect = rects[index];
    if (control.hidden || NSIsEmptyRect(rect)) continue;
    NSSize size = rect.size;
    uint64_t key = UINT64_C(1469598103934665603);
    key = action_visual_hash(key, &size, sizeof(size));
    key = action_visual_hash(key, &scale, sizeof(scale));
    key = action_visual_hash(key, state.action_fills + index * 4,
                             sizeof(float) * 4);
    NSUInteger textHash = texts[index].hash;
    key = action_visual_hash(key, &textHash, sizeof(textHash));
    if (control.visualKey == key) continue;

    ScreenwideRegionOscVertex vertices[12];
    NSUInteger count = 0;
    screenwide_region_osc_add_quad(
        vertices, &count, size, NSMakeRect(0.0, 0.0, size.width, size.height),
        index == 0 ? 12 : 13);
    screenwide_region_osc_add_quad(
        vertices, &count, size,
        NSMakeRect(6.0, 4.0, labelSizes[index].width,
                   labelSizes[index].height),
        index == 0 ? 11 : 15);
    control.contentLayer.contentsScale = scale;
    control.contentLayer.drawableSize =
        CGSizeMake(MAX(size.width * scale, 2.0),
                   MAX(size.height * scale, 2.0));
    id<CAMetalDrawable> drawable = [control.contentLayer nextDrawable];
    if (drawable == nil) continue;
    id<MTLBuffer> buffer = [surface.device
        newBufferWithBytes:vertices length:sizeof(vertices)
                   options:MTLResourceStorageModeShared];
    MTLRenderPassDescriptor *pass =
        [MTLRenderPassDescriptor renderPassDescriptor];
    pass.colorAttachments[0].texture = drawable.texture;
    pass.colorAttachments[0].loadAction = MTLLoadActionClear;
    pass.colorAttachments[0].storeAction = MTLStoreActionStore;
    pass.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);
    id<MTLCommandBuffer> command = [surface.queue commandBuffer];
    id<MTLRenderCommandEncoder> encoder =
        [command renderCommandEncoderWithDescriptor:pass];
    screenwide_region_osc_encode(
        encoder, surface.selectionPipeline, buffer, count, state,
        labels[0], labels[1]);
    [encoder endEncoding];
    [command presentDrawable:drawable];
    [command commit];
    control.visualKey = key;
  }
}

SCREENWIDE_PREVIEW_PRIVATE void selection_action_fills(
    ScreenwidePreviewSurface *surface, uint32_t lightMode, float fills[8]) {
  memset(fills, 0, sizeof(float) * 8);
  ScreenwideOscControlVisual visuals[2];
  size_t count = screenwide_osc_control_group_visuals(
      surface.selectionActionControls, lightMode == 0, visuals, 2);
  for (size_t index = 0; index < count; index++)
    memcpy(fills + index * 4, visuals[index].fill,
           sizeof(visuals[index].fill));
}

SCREENWIDE_PREVIEW_PRIVATE double selection_recenter_scale(
    ScreenwidePreviewSelection start, ScreenwidePreviewSelection resized,
    uint32_t edges) {
  BOOL verticalOnly = (edges & (4 | 8)) != 0 && (edges & (1 | 2)) == 0;
  double startSize = verticalOnly ? start.height : start.width;
  double resizedSize = verticalOnly ? resized.height : resized.width;
  return resizedSize / MAX(startSize, 0.000001);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_hover(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  ScreenwideOscControlUpdate update = surface.selectionActionOperation != 0
      ? screenwide_osc_control_group_hover(
            surface.selectionActionControls, point.x, point.y)
      : screenwide_osc_control_group_clear_hover(
            surface.selectionActionControls);
  apply_selection_action_update(surface, update);
  return update.consumed != 0;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_hit(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  return surface.selectionActionOperation != 0 &&
      screenwide_osc_control_group_hit(
          surface.selectionActionControls, point.x, point.y) != 0;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_clear_hover(
    ScreenwidePreviewSurface *surface) {
  ScreenwideOscControlUpdate update =
      screenwide_osc_control_group_clear_hover(
          surface.selectionActionControls);
  apply_selection_action_update(surface, update);
  return update.changed != 0;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_begin(
    ScreenwidePreviewSurface *surface, NSInteger button, NSPoint point) {
  if (button != 0 || surface.selectionActionOperation == 0)
    return NO;
  ScreenwideOscControlUpdate update = screenwide_osc_control_group_down(
      surface.selectionActionControls, point.x, point.y);
  apply_selection_action_update(surface, update);
  if (!update.consumed) return NO;
  set_selection_cursor([NSCursor arrowCursor]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_drag(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  ScreenwideOscControlUpdate update = screenwide_osc_control_group_hover(
      surface.selectionActionControls, point.x, point.y);
  apply_selection_action_update(surface, update);
  if (!update.consumed) return NO;
  set_selection_cursor([NSCursor arrowCursor]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL selection_action_end(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  ScreenwideOscControlUpdate update = screenwide_osc_control_group_up(
      surface.selectionActionControls, point.x, point.y);
  if (!update.consumed) return NO;
  uint32_t operation = surface.selectionActionOperation;
  if (operation == 8 && update.activated == 2)
    operation = 9;
  apply_selection_action_update(surface, update);
  if (update.activated != 0 && operation != 0)
    emit_selection_gesture(surface, 0, operation, 0, 1.0, 0.0, 0.0);
  set_selection_cursor([NSCursor arrowCursor]);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE ScreenwidePreviewSelection selection_recenter_resize(
    ScreenwidePreviewSelection start, uint32_t edges, double deltaX,
    double deltaY, NSSize pane, double *scale) {
  BOOL verticalOnly = (edges & (4 | 8)) != 0 && (edges & (1 | 2)) == 0;
  double paneWidth = MAX(pane.width, 1.0);
  double paneHeight = MAX(pane.height, 1.0);
  double startInset = verticalOnly
      ? (start.height - start.image_height) * paneHeight / 2.0
      : (start.width - start.image_width) * paneWidth / 2.0;
  double requestedDelta = verticalOnly
      ? ((edges & 4) ? -deltaY : (edges & 8) ? deltaY : 0.0) * paneHeight
      : ((edges & 1) ? -deltaX : (edges & 2) ? deltaX : 0.0) * paneWidth;
  double requestedInset = startInset + requestedDelta;
  double maximumInset = fmin(
      fmin((start.image_x - start.recenter_x) * paneWidth,
           (start.image_y - start.recenter_y) * paneHeight),
      fmin((start.recenter_x + start.recenter_width - start.image_x -
            start.image_width) * paneWidth,
           (start.recenter_y + start.recenter_height - start.image_y -
            start.image_height) * paneHeight));
  double inset = fmin(fmax(maximumInset, 0.0), fmax(requestedInset, 0.0));

  ScreenwidePreviewSelection resized = start;
  resized.x = start.image_x - inset / paneWidth;
  resized.y = start.image_y - inset / paneHeight;
  resized.width = start.image_width + 2.0 * inset / paneWidth;
  resized.height = start.image_height + 2.0 * inset / paneHeight;
  *scale = selection_recenter_scale(start, resized, edges);
  return resized;
}

SCREENWIDE_PREVIEW_PRIVATE void selection_recenter_drag(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection start,
    uint32_t edges, double deltaX, double deltaY, NSSize pane) {
  double scale = 1.0;
  ScreenwidePreviewSelection resized = selection_recenter_resize(
      start, edges, deltaX, deltaY, pane, &scale);
  surface.selection = resized;
  apply_editor_transform(surface);
  emit_selection_gesture(surface, 1, 1, edges, scale,
                         resized.x - start.x, resized.y - start.y);
}
