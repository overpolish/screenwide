// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A redaction's chrome: the layer selection's own box, with its eight grips
//! and its radius dot, around the chosen one. A hovered one wears the
//! compositor's halo instead.

#import "recording_preview_surface_macos_private.h"
#include "recording_preview_annotation_layers_macos.h"

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
/// twin of `redact::gesture::radius_at`, which reads a dragged dot back.
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
  // The radius percentage rides in `start_head`, which a box has no use for.
  handles[8] = redact_radius_point(frame, item.start_head);
  kinds[8] = ScreenwideAnnotationHandleRadius;
  return 9;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_redact_add_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  NSInteger selected = surface.annotationSelected;
  if (list == NULL || selected < 0 || (NSUInteger)selected >= items ||
      list[selected].kind != ScreenwideAnnotationKindRedact)
    return NO;
  NSRect image = annotation_image_frame(surface);
  if (image.size.width > 0.0 && image.size.height > 0.0)
    screenwide_region_osc_add_selection(vertices, count, size,
                                        redact_frame(image, list[selected]), scale,
                                        list[selected].start_head, YES);
  return YES;
}
