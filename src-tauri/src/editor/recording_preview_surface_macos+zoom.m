// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

#include <math.h>

static const double ScreenwideMinimumEditorZoomCeiling = 16.0;
static const double ScreenwideNativePixelZoomHeadroom = 4.0;

/// The union of the pane rects the workspace is currently composed from, in
/// pre-zoom points - the same bounds `selection_pixel_size` converts output
/// pixels with.
SCREENWIDE_PREVIEW_PRIVATE NSRect editor_base_bounds(ScreenwidePreviewSurface *surface) {
  NSRect bounds = NSZeroRect;
  BOOL hasBounds = NO;
  for (NSUInteger index = 0; index < surface.editorBaseRects.count; index++) {
    if (surface.workspaceActivePaneIndices != nil &&
        ![surface.workspaceActivePaneIndices containsObject:@(index)])
      continue;
    NSRect frame = surface.editorBaseRects[index].rectValue;
    if (NSIsEmptyRect(frame)) continue;
    bounds = hasBounds ? NSUnionRect(bounds, frame) : frame;
    hasBounds = YES;
  }
  return hasBounds ? bounds : NSZeroRect;
}

/// The largest zoom this surface currently allows, as a fit-relative factor.
///
/// A zoom of 1.0 is fit-to-pane, so one output pixel covers one physical
/// screen pixel at `natural / (points * backingScale)` - the native pixel
/// scale. A tall scrolling capture fits so small that the fixed 16x ceiling
/// lands far below 100% actual pixels, so the ceiling follows the content and
/// leaves room to inspect it, while never dropping under the fixed one for
/// ordinary images.
SCREENWIDE_PREVIEW_PRIVATE double maximum_editor_zoom(
    ScreenwidePreviewSurface *surface) {
  if (!surface.workspaceMode || surface.workspaceNaturalWidth <= 0.0 ||
      surface.workspaceNaturalHeight <= 0.0)
    return ScreenwideMinimumEditorZoomCeiling;
  NSRect bounds = editor_base_bounds(surface);
  if (NSIsEmptyRect(bounds)) return ScreenwideMinimumEditorZoomCeiling;
  CGFloat scale = surface.host.window.backingScaleFactor;
  if (!(scale > 0.0)) scale = 1.0;
  // Both axes, so an aspect ratio the fit rounds differently still reaches
  // native pixel scale on the axis that fits tightest.
  double nativeScale =
      fmax(surface.workspaceNaturalWidth / (bounds.size.width * scale),
           surface.workspaceNaturalHeight / (bounds.size.height * scale));
  return fmax(ScreenwideMinimumEditorZoomCeiling,
              ScreenwideNativePixelZoomHeadroom * nativeScale);
}

SCREENWIDE_PREVIEW_PRIVATE void set_editor_zoom(ScreenwidePreviewSurface *surface,
                            double zoom, NSPoint anchor) {
  double previous = surface.editorZoom;
  zoom = fmin(maximum_editor_zoom(surface), fmax(0.1, zoom));
  if (fabs(previous - zoom) < 0.000001) return;
  double centeredX = anchor.x - NSMidX(surface.interaction.bounds);
  double centeredY = anchor.y - NSMidY(surface.interaction.bounds);
  double ratio = zoom / previous;
  surface.editorPanX =
      centeredX - (centeredX - surface.editorPanX) * ratio;
  surface.editorPanY =
      centeredY - (centeredY - surface.editorPanY) * ratio;
  surface.editorZoom = zoom;
  apply_editor_transform(surface);
  if (surface.transformCallback)
    surface.transformCallback(zoom * 100.0, surface.transformContext);
}

/// A layout that grows the pane or shrinks the canvas lowers the ceiling under
/// the zoom the user is already at, so every layout re-clamps and tells the
/// toolbar where the zoom ended up.
SCREENWIDE_PREVIEW_PRIVATE void clamp_editor_zoom_to_ceiling(
    ScreenwidePreviewSurface *surface) {
  if (!surface.editorEnabled) return;
  if (surface.interaction.selectionDragActive) return;
  double ceiling = maximum_editor_zoom(surface);
  if (surface.editorZoom <= ceiling) return;
  set_editor_zoom(surface, ceiling,
                  NSMakePoint(NSMidX(surface.interaction.bounds),
                              NSMidY(surface.interaction.bounds)));
}

@implementation ScreenwidePreviewInteractionView (Zoom)
- (void)scrollWheel:(NSEvent *)event {
  if (self.pinching) return;
  // Discard the overlapping scroll sequence, including its momentum tail.
  // A fresh finger scroll (or a phase-less mouse wheel) can pan again.
  if (event.phase & NSEventPhaseBegan ||
      (event.phase == NSEventPhaseNone &&
       event.momentumPhase == NSEventPhaseNone))
    self.suppressPinchScroll = NO;
  if (self.suppressPinchScroll) return;
  if (event.modifierFlags & NSEventModifierFlagControl) {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    set_editor_zoom(self.surface,
                    self.surface.editorZoom * exp(-event.scrollingDeltaY * 0.01),
                    point);
  } else {
    self.surface.editorPanX += event.scrollingDeltaX;
    self.surface.editorPanY += event.scrollingDeltaY;
    apply_editor_transform(self.surface);
  }
}
- (void)magnifyWithEvent:(NSEvent *)event {
  BOOL finished = (event.phase & (NSEventPhaseEnded | NSEventPhaseCancelled)) != 0;
  if (!self.pinching || (event.phase & NSEventPhaseBegan)) {
    if (finished) return;
    self.pinchAnchor = [self convertPoint:event.locationInWindow fromView:nil];
    self.pinching = YES;
  }
  self.suppressPinchScroll = YES;
  // Keep the same canvas point under the initial anchor for the whole pinch.
  if (!(event.phase & NSEventPhaseCancelled))
    set_editor_zoom(self.surface,
                    self.surface.editorZoom * (1.0 + event.magnification),
                    self.pinchAnchor);
  if (finished) self.pinching = NO;
}
@end
