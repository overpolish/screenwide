// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include <math.h>

SCREENWIDE_PREVIEW_PRIVATE void update_crop_magnifier(
    ScreenwidePreviewSurface *surface, NSPoint point, uint32_t edges) {
  if (!surface.workspaceMode || !surface.hasSelection ||
      surface.selection.crop_mode == 0 ||
      surface.selection.pane_index >= surface.editorBaseRects.count) {
    ScreenwideRegionMagnifier cleared = surface.workspaceMagnifier;
    cleared.active = 0;
    surface.workspaceMagnifier = cleared;
    return;
  }
  NSRect transformed = editor_frame(
      surface, surface.editorBaseRects[surface.selection.pane_index].rectValue);
  NSRect pane = NSMakeRect(
      transformed.origin.x,
      surface.interaction.bounds.size.height - NSMaxY(transformed),
      transformed.size.width, transformed.size.height);
  if (NSIsEmptyRect(pane) || surface.selection.image_width <= 0.0 ||
      surface.selection.image_height <= 0.0) {
    ScreenwideRegionMagnifier cleared = surface.workspaceMagnifier;
    cleared.active = 0;
    surface.workspaceMagnifier = cleared;
    return;
  }
  point = screenwide_region_magnifier_anchor(
      point, selection_display_frame(surface), edges);
  double paneX = (point.x - NSMinX(pane)) / pane.size.width;
  double paneY = (point.y - NSMinY(pane)) / pane.size.height;
  BOOL hasRawImage = surface.selection.recenter_width > 0.0 &&
                     surface.selection.recenter_height > 0.0;
  double rawX = hasRawImage ? surface.selection.recenter_x : surface.selection.image_x;
  double rawY = hasRawImage ? surface.selection.recenter_y : surface.selection.image_y;
  double rawWidth = hasRawImage ? surface.selection.recenter_width
                                : surface.selection.image_width;
  double rawHeight = hasRawImage ? surface.selection.recenter_height
                                 : surface.selection.image_height;
  double sourceMinU = (surface.selection.image_x - rawX) / rawWidth;
  double sourceMinV = (surface.selection.image_y - rawY) / rawHeight;
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  uint32_t lightMode = [[[surface.interaction effectiveAppearance]
      bestMatchFromAppearancesWithNames:@[NSAppearanceNameAqua,
                                          NSAppearanceNameDarkAqua]]
      isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
  surface.workspaceMagnifier = screenwide_region_magnifier_make(
      point, scale, edges, lightMode, surface.selection.pane_index,
      surface.selection.layer_id,
      surface.workspaceExplicitPlacements &&
          surface.selection.layer_id != surface.selection.pane_index,
      (float)((paneX - rawX) / rawWidth),
      (float)((paneY - rawY) / rawHeight), (float)sourceMinU,
      (float)sourceMinV,
      (float)(sourceMinU + surface.selection.image_width / rawWidth),
      (float)(sourceMinV + surface.selection.image_height / rawHeight));
}
