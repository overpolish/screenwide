// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include <math.h>

static NSPoint crop_magnifier_anchor(NSPoint point, NSRect crop) {
  return NSMakePoint(fmin(NSMaxX(crop), fmax(NSMinX(crop), point.x)),
                     fmin(NSMaxY(crop), fmax(NSMinY(crop), point.y)));
}

SCREENWIDE_PREVIEW_PRIVATE void update_crop_magnifier(
    ScreenwidePreviewSurface *surface, NSPoint point, uint32_t edges) {
  if (!surface.workspaceMode || !surface.hasSelection ||
      surface.selection.crop_mode == 0 ||
      surface.selection.pane_index >= surface.editorBaseRects.count) {
    ScreenwideWorkspaceMagnifier cleared = surface.workspaceMagnifier;
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
    ScreenwideWorkspaceMagnifier cleared = surface.workspaceMagnifier;
    cleared.active = 0;
    surface.workspaceMagnifier = cleared;
    return;
  }
  point = crop_magnifier_anchor(point, selection_display_frame(surface));
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
  int32_t size = (int32_t)MAX(llround(96.0 * scale), 1);
  int32_t centerX = (int32_t)llround(point.x * scale);
  int32_t centerY = (int32_t)llround(point.y * scale);
  surface.workspaceMagnifier = (ScreenwideWorkspaceMagnifier){
    .active = 1,
    .pane_index = surface.selection.pane_index,
    .layer_id = surface.selection.layer_id,
    .sample_camera = surface.workspaceExplicitPlacements &&
                     surface.selection.layer_id != surface.selection.pane_index,
    .edges = edges,
    .light_mode = [[[surface.interaction effectiveAppearance]
        bestMatchFromAppearancesWithNames:@[NSAppearanceNameAqua,
                                            NSAppearanceNameDarkAqua]]
        isEqualToString:NSAppearanceNameAqua] ? 1 : 0,
    .sample_u = (float)fmin(1.0, fmax(0.0, (paneX - rawX) / rawWidth)),
    .sample_v = (float)fmin(1.0, fmax(0.0, (paneY - rawY) / rawHeight)),
    .source_min_u = (float)fmin(1.0, fmax(0.0, sourceMinU)),
    .source_min_v = (float)fmin(1.0, fmax(0.0, sourceMinV)),
    .source_max_u = (float)fmin(1.0, fmax(0.0,
        sourceMinU + surface.selection.image_width / rawWidth)),
    .source_max_v = (float)fmin(1.0, fmax(0.0,
        sourceMinV + surface.selection.image_height / rawHeight)),
    .box_x = centerX - size / 2,
    .box_y = centerY - size / 2,
    .box_width = (uint32_t)size,
    .box_height = (uint32_t)size,
  };
}
