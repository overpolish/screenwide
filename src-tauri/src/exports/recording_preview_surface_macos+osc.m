// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#import "region_osc_renderer_macos.h"


static NSRect selection_image_frame_for(
    ScreenwidePreviewSurface *surface,
    ScreenwidePreviewSelection selection) {
  ScreenwidePreviewSelection image = selection;
  image.x = selection.image_x;
  image.y = selection.image_y;
  image.width = selection.image_width;
  image.height = selection.image_height;
  return selection_display_frame_for(surface, image);
}

/// Size of the current selection in OUTPUT pixels, or NO when the workspace
/// has no pixel scale to convert with.
///
/// `workspaceNaturalWidth/Height` is the canvas size in output pixels and the
/// pane rects are pre-zoom points, so pixels-per-point is simply natural over
/// the union of the ACTIVE pane rects. That relation holds in every gesture
/// path by construction: the screenshot workspace has a single pane whose rect
/// is the canvas (and `update_workspace_frame_resize` /
/// `update_workspace_auto_fit_move` keep natural live during a drag), and the
/// recording workspace's `rebase_recording_workspace_fit` scales natural by
/// exactly the union-bounds ratio it rebases the pane rects with.
static BOOL selection_pixel_size(ScreenwidePreviewSurface *surface,
                                 double *width, double *height) {
  if (!surface.workspaceMode || !surface.hasSelection) return NO;
  if (surface.workspaceNaturalWidth <= 0.0 ||
      surface.workspaceNaturalHeight <= 0.0) return NO;
  if (surface.selection.pane_index >= surface.editorBaseRects.count) return NO;
  NSRect bounds = NSZeroRect;
  BOOL hasBounds = NO;
  for (NSNumber *value in surface.workspaceActivePaneIndices) {
    NSUInteger index = value.unsignedIntegerValue;
    if (index >= surface.editorBaseRects.count) continue;
    NSRect frame = surface.editorBaseRects[index].rectValue;
    if (NSIsEmptyRect(frame)) continue;
    bounds = hasBounds ? NSUnionRect(bounds, frame) : frame;
    hasBounds = YES;
  }
  if (!hasBounds || NSIsEmptyRect(bounds)) return NO;
  NSRect pane = surface.editorBaseRects[surface.selection.pane_index].rectValue;
  double perPointX = surface.workspaceNaturalWidth / bounds.size.width;
  double perPointY = surface.workspaceNaturalHeight / bounds.size.height;
  *width = surface.selection.width * pane.size.width * perPointX;
  *height = surface.selection.height * pane.size.height * perPointY;
  return YES;
}


static void redraw_selection_impl(ScreenwidePreviewSurface *surface) {
  surface.selectionDrawRevision += 1;
  uint64_t revision = surface.selectionDrawRevision;
  BOOL workspaceEncoding = surface.workspaceMode &&
      surface.workspaceEncodingCommand != nil &&
      surface.workspaceEncodingTexture != nil;
  if (surface.workspaceMode && !workspaceEncoding) {
    surface.selectionLayer.hidden = YES;
    redraw_workspace(surface);
    return;
  }
  BOOL selectedPaneActive = surface.workspaceMode
      ? [surface.workspaceActivePaneIndices
            containsObject:@(surface.selection.pane_index)]
      : surface.selection.pane_index < surface.views.count &&
            surface.views[surface.selection.pane_index].active;
  if (!surface.hasSelection || !surface.selectionVisible ||
      !surface.editorEnabled ||
      surface.selectionLayer == nil || surface.selectionPipeline == nil ||
      surface.selection.pane_index >= surface.editorBaseRects.count ||
      !selectedPaneActive) {
    surface.selectionDrawPending = NO;
    surface.selectionLayer.hidden = YES;
    return;
  }
  // Keep at most one OSC drawable in flight. `nextDrawable` otherwise waits
  // for display presentation when pointer events arrive faster than the
  // monitor refreshes, blocking AppKit for most of a frame on every move and
  // eventually hitting CAMetalLayer's one-second drawable timeout. A newer
  // gesture sample simply replaces the pending draw.
  if (!workspaceEncoding && surface.selectionDrawInFlight) {
    surface.selectionDrawPending = YES;
    return;
  }
  if (!workspaceEncoding) {
    surface.selectionDrawInFlight = YES;
    surface.selectionDrawPending = NO;
  }
  NSSize size = surface.interaction.bounds.size;
  NSRect pane = surface.editorBaseRects[surface.selection.pane_index].rectValue;
  NSRect base = NSMakeRect(pane.origin.x + pane.size.width * surface.selection.x,
                           pane.origin.y + pane.size.height * surface.selection.y,
                           pane.size.width * surface.selection.width,
                           pane.size.height * surface.selection.height);
  NSRect transformed = editor_frame(surface, base);
  NSRect frame = NSMakeRect(transformed.origin.x,
                            size.height - transformed.origin.y - transformed.size.height,
                            transformed.size.width, transformed.size.height);
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  // Resolved before the vertices are built because the size readout rasterises
  // its own colours from it; both encode paths below reuse this value.
  NSString *appearance = [surface.interaction.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[NSAppearanceNameAqua,
                                          NSAppearanceNameDarkAqua]];
  uint32_t lightMode = [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
  ScreenwideRegionOscVertex vertices[512];
  NSUInteger count = 0;
  // Match Keyframeless's contrast-safe OSC construction: hard-edged quads
  // snapped to drawable-pixel centres, with a 3px dark halo underneath a 1px
  // white core. Handles keep their 8pt fill and gain a 1-device-pixel ring.
  if (surface.selection.crop_mode != 0)
    screenwide_region_osc_add_crop(
        vertices, &count, size, frame,
        selection_image_frame_for(surface, surface.selection), scale);
  else
    screenwide_region_osc_add_selection(
        vertices, &count, size, frame, scale,
        surface.selection.radius_percent,
        surface.selection.radius_disabled == 0);
  double pixelWidth = 0.0;
  double pixelHeight = 0.0;
  BOOL keyboardAction = surface.selection.layer_id == UINT32_MAX - 1;
  BOOL recenterAction = surface.selection.recenter_mode != 0;
  BOOL compactAction = recenterAction || keyboardAction;
  BOOL hasLabel = compactAction ||
      selection_pixel_size(surface, &pixelWidth, &pixelHeight);
  surface.selectionActionRect = NSZeroRect;
  surface.selectionSecondaryActionRect = NSZeroRect;
  surface.selectionActionOperation = keyboardAction ? 8 : recenterAction ? 7 : 0;
  if (keyboardAction &&
      [surface updateSelectionLabel:@"Reset" scale:scale
                          lightMode:lightMode action:YES] &&
      [surface updateSelectionSecondaryLabel:@"Apply to all" scale:scale
                                    lightMode:lightMode]) {
    NSSize primaryLabel = surface.selectionLabelSize;
    NSSize secondaryLabel = surface.selectionSecondaryLabelSize;
    CGFloat buttonGap = 4.0;
    CGFloat buttonHeight = MAX(primaryLabel.height, secondaryLabel.height) + 8.0;
    CGFloat primaryWidth = primaryLabel.width + 12.0;
    CGFloat secondaryWidth = secondaryLabel.width + 12.0;
    CGFloat totalWidth = primaryWidth + buttonGap + secondaryWidth;
    CGFloat actionX = NSMidX(frame) - totalWidth / 2.0;
    CGFloat actionY = NSMaxY(frame) + 6.0;
    if (actionY + buttonHeight > size.height)
      actionY = NSMinY(frame) - 6.0 - buttonHeight;
    actionX = MAX(0.0, MIN(actionX, size.width - totalWidth));
    actionY = MAX(0.0, MIN(actionY, size.height - buttonHeight));
    actionX = floor(actionX * scale) / scale;
    actionY = floor(actionY * scale) / scale;
    surface.selectionActionRect = NSMakeRect(
        actionX, actionY, primaryWidth, buttonHeight);
    surface.selectionSecondaryActionRect = NSMakeRect(
        actionX + primaryWidth + buttonGap, actionY,
        secondaryWidth, buttonHeight);
    screenwide_region_osc_add_quad(vertices, &count, size,
                                   surface.selectionActionRect, 12);
    screenwide_region_osc_add_quad(vertices, &count, size,
                                   surface.selectionSecondaryActionRect, 13);
    screenwide_region_osc_add_quad(vertices, &count, size,
        NSMakeRect(NSMinX(surface.selectionActionRect) + 6.0,
                   NSMinY(surface.selectionActionRect) + 4.0,
                   primaryLabel.width, primaryLabel.height), 11);
    screenwide_region_osc_add_quad(vertices, &count, size,
        NSMakeRect(NSMinX(surface.selectionSecondaryActionRect) + 6.0,
                   NSMinY(surface.selectionSecondaryActionRect) + 4.0,
                   secondaryLabel.width, secondaryLabel.height), 15);
  } else if (hasLabel && !keyboardAction) {
    NSString *text = compactAction ? @"Recenter" :
        [NSString stringWithFormat:@"%lld × %lld",
         (long long)MAX(1, llround(pixelWidth)),
         (long long)MAX(1, llround(pixelHeight))];
    if ([surface updateSelectionLabel:text
                                scale:scale
                            lightMode:lightMode
                               action:compactAction]) {
      NSSize label = surface.selectionLabelSize;
      // The compact action's 4pt top padding leaves the visible button at the
      // same 6pt distance from the selection frame as before.
      CGFloat gap = compactAction ? 10.0 : 4.0;
      CGFloat x = compactAction ? NSMidX(frame) - label.width / 2.0
                                 : NSMaxX(frame) - label.width;
      CGFloat y = NSMaxY(frame) + gap;
      if (y + label.height > size.height)
        y = NSMaxY(frame) - gap - label.height;
      x = MAX(0.0, MIN(x, size.width - label.width));
      CGFloat minimumX = NSMinX(frame);
      CGFloat maximumX = NSMaxX(frame) - label.width;
      if (minimumX <= maximumX)
        x = MAX(minimumX, MIN(x, maximumX));
      else
        x = NSMidX(frame) - label.width / 2.0;
      y = MIN(MAX(0.0, y), NSMaxY(frame) + gap);
      x = floor(x * scale) / scale;
      y = floor(y * scale) / scale;
      NSRect labelRect = NSMakeRect(x, y, label.width, label.height);
      if (compactAction) {
        // The label bitmap has 2pt horizontal inset and a 16pt text-xs line
        // box; these insets complete React's px-2/py-1 compact Button geometry.
        NSRect actionRect = NSInsetRect(labelRect, -6.0, -4.0);
        surface.selectionActionRect = actionRect;
        screenwide_region_osc_add_quad(vertices, &count, size,
                                       surface.selectionActionRect, 12);
      }
      screenwide_region_osc_add_quad(vertices, &count, size, labelRect, 11);
    }
  }
  if (surface.hasSelectionSnapGuideX) {
    ScreenwidePreviewSelection guide = surface.selection;
    guide.x = surface.selectionSnapGuideX;
    guide.y = 0.0;
    guide.width = 0.0;
    guide.height = 0.0;
    CGFloat x = screenwide_region_osc_snap(
        NSMinX(selection_display_frame_for(surface, guide)), scale);
    CGFloat half = 0.5 / scale;
    screenwide_region_osc_add_quad(
        vertices, &count, size,
        NSMakeRect(x - half, 0.0, half * 2.0, size.height),
        surface.selectionSnapGuideXIsObject ? 5 : 4);
  }
  if (surface.hasSelectionSnapGuideY) {
    ScreenwidePreviewSelection guide = surface.selection;
    guide.x = 0.0;
    guide.y = surface.selectionSnapGuideY;
    guide.width = 0.0;
    guide.height = 0.0;
    CGFloat y = screenwide_region_osc_snap(
        NSMinY(selection_display_frame_for(surface, guide)), scale);
    CGFloat half = 0.5 / scale;
    screenwide_region_osc_add_quad(
        vertices, &count, size,
        NSMakeRect(0.0, y - half, size.width, half * 2.0),
        surface.selectionSnapGuideYIsObject ? 5 : 4);
  }
  if (workspaceEncoding) {
    id<MTLBuffer> buffer = [surface.device newBufferWithBytes:vertices
        length:count * sizeof(*vertices)
        options:MTLResourceStorageModeShared];
    MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
    pass.colorAttachments[0].texture = surface.workspaceEncodingTexture;
    pass.colorAttachments[0].loadAction = MTLLoadActionLoad;
    pass.colorAttachments[0].storeAction = MTLStoreActionStore;
    id<MTLRenderCommandEncoder> encoder =
        [surface.workspaceEncodingCommand renderCommandEncoderWithDescriptor:pass];
    ScreenwideRegionMagnifier magnifier = surface.workspaceMagnifier;
    ScreenwideRegionOscRenderState state =
        screenwide_region_osc_render_state(lightMode);
    state.magnifier_box[0] = magnifier.active != 0 ? magnifier.box_x : 0;
    state.magnifier_box[1] = magnifier.active != 0 ? magnifier.box_y : 0;
    state.magnifier_box[2] = magnifier.active != 0 ? magnifier.box_width : 0;
    state.magnifier_box[3] = magnifier.active != 0 ? magnifier.box_height : 0;
    selection_action_shades(surface, state.action_shades);
    screenwide_region_osc_encode(
        encoder, surface.selectionPipeline, buffer, count, state,
        surface.selectionLabelTexture ?: surface.selectionLabelPlaceholder,
        surface.selectionSecondaryLabelTexture ?:
            surface.selectionLabelPlaceholder);
    [encoder endEncoding];
    return;
  }
  surface.selectionLayer.frame = surface.interaction.bounds;
  surface.selectionLayer.contentsScale = scale;
  surface.selectionLayer.drawableSize = CGSizeMake(MAX(size.width * scale, 2.0),
                                                    MAX(size.height * scale, 2.0));
  id<CAMetalDrawable> drawable = [surface.selectionLayer nextDrawable];
  if (drawable == nil) {
    surface.selectionDrawInFlight = NO;
    return;
  }
  id<MTLBuffer> buffer = [surface.device newBufferWithBytes:vertices
                                                      length:count * sizeof(*vertices)
                                                     options:MTLResourceStorageModeShared];
  MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = drawable.texture;
  pass.colorAttachments[0].loadAction = MTLLoadActionClear;
  pass.colorAttachments[0].storeAction = MTLStoreActionStore;
  pass.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);
  id<MTLCommandBuffer> command = [surface.queue commandBuffer];
  id<MTLRenderCommandEncoder> encoder = [command renderCommandEncoderWithDescriptor:pass];
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(lightMode);
  selection_action_shades(surface, state.action_shades);
  screenwide_region_osc_encode(
      encoder, surface.selectionPipeline, buffer, count, state,
      surface.selectionLabelTexture ?: surface.selectionLabelPlaceholder,
      surface.selectionSecondaryLabelTexture ?:
          surface.selectionLabelPlaceholder);
  [encoder endEncoding];
  [command presentDrawable:drawable];
  [command addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
    dispatch_async(dispatch_get_main_queue(), ^{
      surface.selectionDrawInFlight = NO;
      BOOL redrawPending = surface.selectionDrawPending;
      surface.selectionDrawPending = NO;
      if (surface.hasSelection && surface.selectionVisible &&
          surface.editorEnabled)
        surface.selectionLayer.hidden = NO;
      if (redrawPending) {
        redraw_selection(surface);
      } else if (surface.selectionDrawRevision == revision &&
                 surface.hasSelection && surface.selectionVisible &&
                 surface.editorEnabled) {
        surface.selectionLayer.hidden = NO;
      }
    });
  }];
  [command commit];
}

SCREENWIDE_PREVIEW_PRIVATE void invalidate_selection_cursor_rects(ScreenwidePreviewSurface *surface);

/// Native GPU overlay extension point for future ruler and annotation OSCs.
@implementation ScreenwidePreviewSurface (OSC)

- (void)redrawSelection {
  redraw_selection_impl(self);
}

@end
