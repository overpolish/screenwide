// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos.h"
#import "osc_controls.h"
#import "recording_preview_surface_macos_private.h"


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
  // Chrome is off while suspended: this branch clears the action rects and
  // hides both the OSC layer and its material, which is the whole visible
  // editor overlay.
  if (!surface.hasSelection || !surface.selectionVisible ||
      !surface.editorEnabled || surface.editorSuspended ||
      surface.selectionLayer == nil || surface.selectionPipeline == nil ||
      surface.selection.pane_index >= surface.editorBaseRects.count ||
      !selectedPaneActive) {
    surface.selectionDrawPending = NO;
    surface.selectionActionRect = NSZeroRect;
    surface.selectionSecondaryActionRect = NSZeroRect;
    surface.selectionActionOperation = 0;
    selection_action_material_layout(surface);
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
  BOOL keyboardAction = surface.selection.layer_id == UINT32_MAX - 1;
  BOOL recenterAction = surface.selection.recenter_mode != 0;
  BOOL compactAction = recenterAction || keyboardAction;
  // Only an action earns a label on the frame. The selection's size is read
  // and set in the Selection panel, so the frame no longer repeats it.
  BOOL hasLabel = compactAction;
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
    ScreenwideOscControlMetrics metrics = screenwide_osc_control_metrics(0, 0);
    ScreenwideOscControlSpacing spacing = screenwide_osc_control_spacing();
    // Label textures already contain 2pt horizontal inset. Complete the
    // shared regular button padding around that intrinsic texture.
    CGFloat labelInsetX = MAX(metrics.padding_x - 2.0, 0.0);
    CGFloat buttonGap = spacing.control;
    CGFloat buttonHeight = metrics.height;
    CGFloat primaryWidth = primaryLabel.width + labelInsetX * 2.0;
    CGFloat secondaryWidth = secondaryLabel.width + labelInsetX * 2.0;
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
  } else if (hasLabel && !keyboardAction) {
    NSString *text = @"Recenter";
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
      }
      if (!compactAction)
        screenwide_region_osc_add_quad(vertices, &count, size, labelRect, 11);
    }
  }
  selection_action_layout(surface);
  selection_action_material_layout(surface);
  selection_action_render_surfaces(surface, scale, lightMode);
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
    id<MTLBuffer> buffer = screenwide_osc_vertex_buffer(
        surface.device, vertices, count, size, scale);
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
  surface.selectionLayer.drawableSize = CGSizeMake(MAX(round(size.width * scale), 2.0),
                                                    MAX(round(size.height * scale), 2.0));
  id<CAMetalDrawable> drawable = [surface.selectionLayer nextDrawable];
  if (drawable == nil) {
    surface.selectionDrawInFlight = NO;
    return;
  }
  id<MTLBuffer> buffer = screenwide_osc_vertex_buffer(
      surface.device, vertices, count, size, scale);
  MTLRenderPassDescriptor *pass = [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = drawable.texture;
  pass.colorAttachments[0].loadAction = MTLLoadActionClear;
  pass.colorAttachments[0].storeAction = MTLStoreActionStore;
  pass.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);
  id<MTLCommandBuffer> command = [surface.queue commandBuffer];
  id<MTLRenderCommandEncoder> encoder = [command renderCommandEncoderWithDescriptor:pass];
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(lightMode);
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
      // A draw started before the suspension must not un-hide the OSC after it.
      if (surface.hasSelection && surface.selectionVisible &&
          surface.editorEnabled && !surface.editorSuspended)
        surface.selectionLayer.hidden = NO;
      if (redrawPending) {
        redraw_selection(surface);
      } else if (surface.selectionDrawRevision == revision &&
                 surface.hasSelection && surface.selectionVisible &&
                 surface.editorEnabled && !surface.editorSuspended) {
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
