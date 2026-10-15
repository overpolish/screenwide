// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's chrome: its own frame, tilted and turned with the picture,
//! whose eight grips move the picture's own sides, and its radius dot. The
//! frame is Rust's (`image/frame.rs`), so this side only reads it, draws it
//! and picks from it.

#import "recording_preview_surface_macos_private.h"
#include <math.h>
#include "../../../../../crates/annotations/src/geometry.h"

/// The chosen image's frame in display points. The middle rides in `middle`,
/// the flat turn in `start_head`, the half width and half height in
/// `end_head` and `width` as shares of the drawn width, the tilt in `tilt_x`
/// and `tilt_y`, and the radius in `radius`.
static AnnotationImageFrame annotation_image_own_frame(NSRect image,
                                                       ScreenwidePreviewAnnotation item) {
  AnnotationImageFrame frame;
  screenwide_image_frame((float)(NSMinX(image) + image.size.width * item.middle_x),
                         (float)(NSMinY(image) + image.size.height * item.middle_y),
                         (float)(item.end_head * image.size.width),
                         (float)(item.width * image.size.width), (float)item.start_head,
                         (float)item.tilt_x, (float)item.tilt_y, (float)item.radius, &frame);
  return frame;
}

SCREENWIDE_PREVIEW_PRIVATE NSUInteger annotation_image_grips(
    NSRect image, ScreenwidePreviewAnnotation item, NSPoint *handles, uint32_t *kinds) {
  AnnotationImageFrame frame = annotation_image_own_frame(image, item);
  for (NSUInteger index = 0; index < 9; index++) {
    handles[index] = NSMakePoint(frame.grips[index].x, frame.grips[index].y);
    kinds[index] = frame.handles[index];
  }
  return 9;
}

SCREENWIDE_PREVIEW_PRIVATE NSCursor *annotation_image_cursor(ScreenwidePreviewAnnotation item,
                                                             NSInteger handle) {
  if (item.kind != ScreenwideAnnotationKindImage) return nil;
  // The radius dot drags along the picture's own diagonal, as a box's dot
  // drags along its upright one.
  uint32_t sides = handle == ScreenwideAnnotationHandleRadius ? 1 | 4
      : handle >= ScreenwideAnnotationHandleBox && handle < ScreenwideAnnotationHandleRadius
          ? (uint32_t)(handle - ScreenwideAnnotationHandleBox)
          : 0;
  if (sides == 0) return nil;
  return screenwide_region_resize_cursor(
             screenwide_image_cursor_sides(sides, (float)item.start_head))
      ?: [NSCursor arrowCursor];
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_image_add_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  NSInteger selected = surface.annotationSelected;
  if (list == NULL || selected < 0 || (NSUInteger)selected >= items ||
      list[selected].kind != ScreenwideAnnotationKindImage)
    return NO;
  NSRect image = annotation_image_frame(surface);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return YES;
  ScreenwidePreviewAnnotation item = list[selected];
  AnnotationImageFrame frame = annotation_image_own_frame(image, item);
  NSPoint corners[4];
  for (NSUInteger index = 0; index < 4; index++)
    corners[index] = NSMakePoint(frame.corners[index].x, frame.corners[index].y);
  double storage[9];
  // The radius dot is the ninth grip, after the frame's eight.
  annotation_osc_add_image_frame(vertices, count, size, annotation_chrome_turn(surface, storage),
                                 corners, NSMakePoint(frame.grips[8].x, frame.grips[8].y),
                                 scale);
  return YES;
}
