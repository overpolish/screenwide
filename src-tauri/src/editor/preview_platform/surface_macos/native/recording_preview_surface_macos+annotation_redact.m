// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A box's chrome - a redaction's, a shape's, a spotlight's, a stroke's or a
//! magnifier's zoom area: the layer selection's own box, with its eight grips
//! and, for a redaction, a shape, a spotlight and a magnifier, its radius dot,
//! around the chosen one. A magnifier adds the grip on its loupe's rim. A
//! hovered one wears the compositor's halo instead. An image wears a frame
//! of its own, turned with it: `+annotation_image.m`.

#import "recording_preview_surface_macos_private.h"
#include "recording_preview_annotation_layers_macos.h"

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_kind_is_box(uint32_t kind) {
  return kind == ScreenwideAnnotationKindRedact || kind == ScreenwideAnnotationKindShape ||
         kind == ScreenwideAnnotationKindSpotlight || kind == ScreenwideAnnotationKindDraw ||
         kind == ScreenwideAnnotationKindMagnify;
}

/// Whether a box has a radius dot: a stroke has no corners to round.
static BOOL annotation_box_has_radius(uint32_t kind) {
  return kind != ScreenwideAnnotationKindDraw;
}

/// The grip a box carries beyond its eight and its radius dot, if any: a
/// magnifier's on its loupe's rim.
static BOOL annotation_box_tail(NSRect image, ScreenwidePreviewAnnotation item, NSPoint *grip) {
  if (item.kind != ScreenwideAnnotationKindMagnify) return NO;
  *grip = annotation_magnify_grip(image, item);
  return YES;
}

/// The box on screen, from the normalised corners Rust published.
static NSRect redact_frame(NSRect image, ScreenwidePreviewAnnotation item) {
  CGFloat left = NSMinX(image) + image.size.width * item.start_x;
  CGFloat top = NSMinY(image) + image.size.height * item.start_y;
  CGFloat right = NSMinX(image) + image.size.width * item.end_x;
  CGFloat bottom = NSMinY(image) + image.size.height * item.end_y;
  return NSMakeRect(left, top, right - left, bottom - top);
}

/// Where the radius dot sits, as the layer selection places its own: in
/// from the top-left corner, further in the rounder the corners are. The
/// twin of `box_gesture::radius_at`, which reads a dragged dot back.
static NSPoint redact_radius_point(NSRect frame, double radius_percent) {
  double offset =
      MIN(frame.size.width, frame.size.height) * radius_percent / 100.0 * 0.55 + 10.0;
  return NSMakePoint(NSMinX(frame) + offset, NSMinY(frame) + offset);
}

SCREENWIDE_PREVIEW_PRIVATE NSUInteger annotation_redact_grips(
    NSRect image, ScreenwidePreviewAnnotation item, NSPoint *handles, uint32_t *kinds) {
  NSRect frame = redact_frame(image, item);
  CGFloat xs[3] = {NSMinX(frame), NSMidX(frame), NSMaxX(frame)};
  CGFloat ys[3] = {NSMinY(frame), NSMidY(frame), NSMaxY(frame)};
  // Clockwise from the top-left corner, the order the selection OSC draws
  // them in, with the sides each moves: 1 left, 2 right, 4 top, 8 bottom.
  const struct { uint8_t x, y; uint32_t edges; } grips[8] = {
      {0, 0, 1 | 4}, {1, 0, 4}, {2, 0, 2 | 4}, {2, 1, 2},
      {2, 2, 2 | 8}, {1, 2, 8}, {0, 2, 1 | 8}, {0, 1, 1},
  };
  for (NSUInteger index = 0; index < 8; index++) {
    handles[index] = NSMakePoint(xs[grips[index].x], ys[grips[index].y]);
    kinds[index] = ScreenwideAnnotationHandleBox + grips[index].edges;
  }
  NSUInteger count = 8;
  // The radius percentage rides in `start_head`, which a box has no use for.
  if (annotation_box_has_radius(item.kind)) {
    handles[count] = redact_radius_point(frame, item.start_head);
    kinds[count] = ScreenwideAnnotationHandleRadius;
    count++;
  }
  NSPoint tail;
  if (!annotation_box_tail(image, item, &tail)) return count;
  handles[count] = tail;
  kinds[count] = ScreenwideAnnotationHandleTail;
  return count + 1;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_redact_add_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  NSInteger selected = surface.annotationSelected;
  if (list == NULL || selected < 0 || (NSUInteger)selected >= items ||
      !annotation_kind_is_box(list[selected].kind))
    return NO;
  NSRect image = annotation_image_frame(surface);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return YES;
  BOOL radius = annotation_box_has_radius(list[selected].kind);
  screenwide_region_osc_add_selection(vertices, count, size,
                                      redact_frame(image, list[selected]), scale,
                                      radius ? list[selected].start_head : 0.0, radius);
  NSPoint grip;
  if (annotation_box_tail(image, list[selected], &grip)) {
    // The loupe's grip wears the same disc an arrow's grips do.
    CGFloat extent = 4.0 + 2.0 / scale;
    CGFloat x = round(grip.x * scale) / scale;
    CGFloat y = round(grip.y * scale) / scale;
    screenwide_region_osc_add_texture_quad(
        vertices, count, size, NSMakeRect(x - extent, y - extent, extent * 2.0, extent * 2.0),
        NSMakeRect(0.0, 0.0, 1.0, 1.0), 3);
  }
  return YES;
}
