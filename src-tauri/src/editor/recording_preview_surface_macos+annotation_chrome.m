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
  // A drawing tool always draws its own chrome; the select tool only once it
  // is holding an annotation, so an ordinary layer selection is untouched.
  return annotation_drawing_mode(mode) ||
         (mode == ScreenwideAnnotationModeSelect &&
          surface.annotationSelected != -1);
}

SCREENWIDE_PREVIEW_PRIVATE NSCursor *annotation_cursor(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  if (mode == ScreenwideAnnotationModeNone) return nil;
  // An annotation under the pointer is something to take hold of, so the
  // pointer says so - never the crosshair the empty picture draws with.
  if (annotation_handle_at_point(surface, point) >= 0 ||
      annotation_shaft_at_point(surface, point) >= 0)
    return [NSCursor arrowCursor];
  // Empty picture: a drawing tool makes an annotation rather than picking one
  // up. The select tool leaves the choice to the layer underneath.
  return annotation_drawing_mode(mode) ? [NSCursor crosshairCursor] : nil;
}

/// How far an equal-gap bar's end ticks reach either side of it, in points.
/// Short enough to read as a measurement rather than as another guide.
static const CGFloat kAnnotationGapTick = 3.0;

/// One equal gap: a hairline the length of the gap, with a tick across each
/// end. `horizontal` is a gap measured across the picture, whose bar runs
/// left to right; the other runs down it.
static void annotation_add_gap_bar(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect image, CGFloat scale, ScreenwideAnnotationGapSpan span,
    BOOL horizontal) {
  CGFloat half = 0.5 / scale;
  CGFloat origin = horizontal ? NSMinX(image) : NSMinY(image);
  CGFloat extent = horizontal ? image.size.width : image.size.height;
  CGFloat from = origin + extent * span.from;
  CGFloat to = origin + extent * span.to;
  if (to < from) return;
  CGFloat cross = screenwide_region_osc_snap(
      (horizontal ? NSMinY(image) : NSMinX(image)) +
          (horizontal ? image.size.height : image.size.width) * span.cross,
      scale);
  if (to > from)
    screenwide_region_osc_add_quad(
        vertices, count, size,
        horizontal ? NSMakeRect(from, cross - half, to - from, half * 2.0)
                   : NSMakeRect(cross - half, from, half * 2.0, to - from),
        5);
  for (NSUInteger index = 0; index < 2; index++) {
    CGFloat end = screenwide_region_osc_snap(index == 0 ? from : to, scale);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        horizontal ? NSMakeRect(end - half, cross - kAnnotationGapTick,
                                half * 2.0, kAnnotationGapTick * 2.0)
                   : NSMakeRect(cross - kAnnotationGapTick, end - half,
                                kAnnotationGapTick * 2.0, half * 2.0),
        5);
  }
}

/// The snap chrome: the axis guides a counter's disc landed on, the equal
/// gaps it lined up with, and the element an arrow's tip took hold of.
/// Placed inside the same picture the grips are, because that is the space
/// Rust normalises them in.
SCREENWIDE_PREVIEW_PRIVATE void annotation_add_snap_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  ScreenwideAnnotationSnap snap = surface.annotationSnap;
  if (snap.flags == 0) return;
  NSRect image = annotation_image_frame(surface);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return;
  // The same lines the layer engine's guides are drawn with, in the same two
  // colours: 4 for one of the canvas's own, 5 for another annotation's.
  CGFloat half = 0.5 / scale;
  if ((snap.flags & ScreenwideAnnotationSnapGuideX) != 0) {
    CGFloat x = screenwide_region_osc_snap(
        NSMinX(image) + image.size.width * snap.guide_x, scale);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(x - half, 0.0, half * 2.0, size.height),
        snap.guide_x_object != 0 ? 5 : 4);
  }
  if ((snap.flags & ScreenwideAnnotationSnapGuideY) != 0) {
    CGFloat y = screenwide_region_osc_snap(
        NSMinY(image) + image.size.height * snap.guide_y, scale);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(0.0, y - half, size.width, half * 2.0),
        snap.guide_y_object != 0 ? 5 : 4);
  }
  for (NSUInteger index = 0; index < 2; index++) {
    if ((snap.flags & ScreenwideAnnotationSnapGapX) != 0)
      annotation_add_gap_bar(vertices, count, size, image, scale,
                             snap.gap_x[index], YES);
    if ((snap.flags & ScreenwideAnnotationSnapGapY) != 0)
      annotation_add_gap_bar(vertices, count, size, image, scale,
                             snap.gap_y[index], NO);
  }
  if ((snap.flags & ScreenwideAnnotationSnapAnchor) == 0) return;
  CGFloat pixel = 1.0 / scale;
  CGFloat left = screenwide_region_osc_snap(
      NSMinX(image) + image.size.width * snap.box_x, scale);
  CGFloat top = screenwide_region_osc_snap(
      NSMinY(image) + image.size.height * snap.box_y, scale);
  CGFloat right = screenwide_region_osc_snap(
      left + image.size.width * snap.box_width, scale);
  CGFloat bottom = screenwide_region_osc_snap(
      top + image.size.height * snap.box_height, scale);
  NSRect outline[4] = {
      NSMakeRect(left, top, right - left, pixel),
      NSMakeRect(left, bottom - pixel, right - left, pixel),
      NSMakeRect(left, top, pixel, bottom - top),
      NSMakeRect(right - pixel, top, pixel, bottom - top),
  };
  for (NSUInteger index = 0; index < 4; index++)
    screenwide_region_osc_add_quad(vertices, count, size, outline[index], 5);
  // The point itself wears the grips' own disc, so a snapped tip reads as
  // something the hand has hold of.
  CGFloat extent = 4.0 + 2.0 / scale;
  CGFloat x = round((NSMinX(image) + image.size.width * snap.anchor_x) * scale) / scale;
  CGFloat y = round((NSMinY(image) + image.size.height * snap.anchor_y) * scale) / scale;
  screenwide_region_osc_add_texture_quad(
      vertices, count, size,
      NSMakeRect(x - extent, y - extent, extent * 2.0, extent * 2.0),
      NSMakeRect(0.0, 0.0, 1.0, 1.0), 3);
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

SCREENWIDE_PREVIEW_PRIVATE void annotation_refresh_hover(
    ScreenwidePreviewSurface *surface) {
  if (surface == nil || surface.annotationHovered < 0) return;
  // A frame late rather than straight away: the transform is often applied
  // from a layout Rust is in the middle of, and the report's side never
  // waits on that lock.
  uint64_t revision = surface.annotationHoverRevision;
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   if (surface.annotationHoverRevision != revision ||
                       surface.annotationHovered < 0)
                     return;
                   report_annotation_hover(surface);
                 });
}
