// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the arrow chrome looks like: which mode it is in, the cursor it asks
//! for, its grips, and the halo that grows under a hovered arrow.

#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// How long the hover halo takes to grow, matching the ruler's own hover
/// pulse. The widths it grows between live with the halo itself, on the Rust
/// side that turns this progress into the shader's uniform.
static const CFTimeInterval kAnnotationHoverDuration = 0.160;

SCREENWIDE_PREVIEW_PRIVATE ScreenwideAnnotationMode annotation_active_mode(
    ScreenwidePreviewSurface *surface) {
  if (surface == nil || !surface.editorEnabled || surface.editorSuspended ||
      surface.editorBaseRects.count == 0)
    return ScreenwideAnnotationModeNone;
  return surface.annotationMode;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_owns_chrome(
    ScreenwidePreviewSurface *surface) {
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  // The arrow tool always draws its own chrome; the select tool only once it
  // is holding an arrow, so an ordinary layer selection is untouched.
  return mode == ScreenwideAnnotationModeArrow ||
         (mode == ScreenwideAnnotationModeSelect &&
          surface.annotationSelected != -1);
}

SCREENWIDE_PREVIEW_PRIVATE NSCursor *annotation_cursor(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  if (mode == ScreenwideAnnotationModeNone) return nil;
  // An arrow under the pointer is something to take hold of, so the pointer
  // says so - never the crosshair the empty picture draws with.
  if (annotation_handle_at_point(surface, point) >= 0 ||
      annotation_shaft_at_point(surface, point) >= 0)
    return [NSCursor arrowCursor];
  // Empty picture: the arrow tool draws rather than picks things up. The
  // select tool leaves the choice to the layer underneath.
  return mode == ScreenwideAnnotationModeArrow ? [NSCursor crosshairCursor]
                                               : nil;
}

static CGFloat annotation_hover_progress(ScreenwidePreviewSurface *surface) {
  CFTimeInterval elapsed =
      CACurrentMediaTime() - surface.annotationHoverStarted;
  return MIN(MAX(elapsed / kAnnotationHoverDuration, 0.0), 1.0);
}

/// Tells Rust where the halo stands, with the picture's on-screen width so it
/// can turn the halo's points into the canvas pixels the shader draws in.
static void report_annotation_hover(ScreenwidePreviewSurface *surface) {
  if (surface.annotationHoverCallback == NULL) return;
  NSRect image = annotation_image_frame(surface);
  surface.annotationHoverCallback(
      (int32_t)surface.annotationHovered,
      surface.annotationHovered < 0 ? 0.0 : annotation_hover_progress(surface),
      image.size.width, surface.annotationHoverContext);
}

/// One frame of the pulse. Only the pulse re-presents; an ordinary pointer
/// move over the same arrow reports nothing at all.
static void schedule_annotation_hover_frame(
    ScreenwidePreviewSurface *surface, uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   if (surface.annotationHoverRevision != revision ||
                       surface.annotationHovered < 0)
                     return;
                   report_annotation_hover(surface);
                   if (CACurrentMediaTime() - surface.annotationHoverStarted <
                       kAnnotationHoverDuration)
                     schedule_annotation_hover_frame(surface, revision);
                 });
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_update_hover(
    ScreenwidePreviewSurface *surface, NSPoint point, BOOL dragging) {
  if (surface == nil) return;
  NSInteger hovered = -1;
  if (!dragging &&
      annotation_active_mode(surface) != ScreenwideAnnotationModeNone) {
    // A grip belongs to the arrow it was drawn for, so resting on one halos
    // that whole arrow, exactly as resting on its shaft does.
    hovered = annotation_handle_at_point(surface, point) >= 0
        ? surface.annotationSelected
        : annotation_shaft_at_point(surface, point);
  }
  if (hovered == surface.annotationHovered) return;
  surface.annotationHovered = hovered;
  surface.annotationHoverStarted = CACurrentMediaTime();
  surface.annotationHoverRevision += 1;
  report_annotation_hover(surface);
  if (hovered >= 0)
    schedule_annotation_hover_frame(surface, surface.annotationHoverRevision);
}
