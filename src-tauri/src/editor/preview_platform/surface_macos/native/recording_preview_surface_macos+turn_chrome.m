// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The annotation chrome, drawn upright or through the selected layer's turn:
// the twin of the Windows overlay's `selection/annotation_chrome.rs`. Upright,
// its lines are snapped quads on the pixel grid. Turned, each piece is placed
// through the turn first and drawn in soft lines, since snapped quads carried
// through a turn step along every side; a grip stays a round disc where its
// middle lands rather than squashing with the picture.

#import "recording_preview_surface_macos_private.h"
#include <math.h>

/// How far past the view a carried guide may run, in points, so its caps
/// stay off screen.
static const CGFloat kTurnGuideOverrun = 8.0;

SCREENWIDE_PREVIEW_PRIVATE const double *annotation_chrome_turn(ScreenwidePreviewSurface *surface,
                                                                double storage[9]) {
  double inverse[9];
  return surface.hasSelection && selection_turn(surface, surface.selection, storage, inverse)
             ? storage
             : NULL;
}

static void turn_carry_rect(const double *turn, NSRect rect, NSPoint corners[4]) {
  corners[0] = turn_carry(turn, NSMakePoint(NSMinX(rect), NSMinY(rect)));
  corners[1] = turn_carry(turn, NSMakePoint(NSMaxX(rect), NSMinY(rect)));
  corners[2] = turn_carry(turn, NSMakePoint(NSMaxX(rect), NSMaxY(rect)));
  corners[3] = turn_carry(turn, NSMakePoint(NSMinX(rect), NSMaxY(rect)));
}

/// The part of the segment from `a` to `b` inside `view`, grown by
/// `kTurnGuideOverrun` all round. NO where none of it is.
static BOOL clip_to_view(NSPoint *a, NSPoint *b, NSSize view) {
  CGFloat dx = b->x - a->x, dy = b->y - a->y;
  CGFloat low = 0.0, high = 1.0;
  const CGFloat ps[4] = {-dx, dx, -dy, dy};
  const CGFloat qs[4] = {a->x + kTurnGuideOverrun, view.width + kTurnGuideOverrun - a->x,
                         a->y + kTurnGuideOverrun, view.height + kTurnGuideOverrun - a->y};
  for (NSUInteger index = 0; index < 4; index++) {
    if (ps[index] == 0.0) {
      if (qs[index] < 0.0) return NO;
      continue;
    }
    CGFloat t = qs[index] / ps[index];
    if (ps[index] < 0.0) low = MAX(low, t);
    else high = MIN(high, t);
  }
  if (low >= high) return NO;
  NSPoint from = *a;
  *a = NSMakePoint(from.x + dx * low, from.y + dy * low);
  *b = NSMakePoint(from.x + dx * high, from.y + dy * high);
  return YES;
}

/// The segment from `from` to `to`, on the selected layer standing upright,
/// placed through `turn` and cut to `view`: the twin of Rust's
/// `carried_segment`. A guide runs the whole view, and the turn can tip part
/// of that behind the eye or fling it far off screen, where its ends would
/// lose all precision on the way to the GPU.
static BOOL turn_carry_segment(const double *turn, NSPoint from, NSPoint to, NSSize view,
                               NSPoint *a, NSPoint *b) {
  // The carried weight runs linearly along the upright segment, so the part
  // in front of the eye is one stretch of it.
  double w0 = turn[6] * from.x + turn[7] * from.y + turn[8];
  double w1 = turn[6] * to.x + turn[7] * to.y + turn[8];
  if (MAX(w0, w1) <= 0.0) return NO;
  double least = 1e-6 * MAX(w0, w1);
  double start = w0 < least ? (least - w0) / (w1 - w0) : 0.0;
  double end = w1 < least ? 1.0 - (least - w1) / (w0 - w1) : 1.0;
  *a = turn_carry(turn, NSMakePoint(from.x + (to.x - from.x) * start,
                                    from.y + (to.y - from.y) * start));
  *b = turn_carry(turn, NSMakePoint(from.x + (to.x - from.x) * end,
                                    from.y + (to.y - from.y) * end));
  return clip_to_view(a, b, view);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_osc_add_grip(ScreenwideRegionOscVertex *vertices,
                                                        NSUInteger *count, NSSize size,
                                                        const double *turn, NSPoint point,
                                                        CGFloat scale) {
  if (turn != NULL) point = turn_carry(turn, point);
  // The same disc the selection OSC draws its corner grips with: a 4pt fill
  // with a one-device-pixel ring, snapped to a whole device pixel.
  CGFloat extent = 4.0 + 2.0 / scale;
  CGFloat x = round(point.x * scale) / scale;
  CGFloat y = round(point.y * scale) / scale;
  screenwide_region_osc_add_texture_quad(
      vertices, count, size, NSMakeRect(x - extent, y - extent, extent * 2.0, extent * 2.0),
      NSMakeRect(0.0, 0.0, 1.0, 1.0), 3);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_osc_add_hairline(ScreenwideRegionOscVertex *vertices,
                                                            NSUInteger *count, NSSize size,
                                                            const double *turn, NSRect rect,
                                                            uint32_t kind, CGFloat scale) {
  if (turn == NULL) {
    screenwide_region_osc_add_quad(vertices, count, size, rect, kind);
    return;
  }
  // Its middle line along its longer side, as wide as it is thin.
  BOOL across = rect.size.width >= rect.size.height;
  NSPoint from = across ? NSMakePoint(NSMinX(rect), NSMidY(rect))
                        : NSMakePoint(NSMidX(rect), NSMinY(rect));
  NSPoint to = across ? NSMakePoint(NSMaxX(rect), NSMidY(rect))
                      : NSMakePoint(NSMidX(rect), NSMaxY(rect));
  CGFloat thickness = across ? rect.size.height : rect.size.width;
  NSPoint a, b;
  if (!turn_carry_segment(turn, from, to, size, &a, &b)) return;
  screenwide_region_osc_add_soft_guide(vertices, count, size, a, b, thickness, kind == 5, scale);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_osc_add_group_frame(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size, const double *turn,
    NSRect rect, CGFloat scale) {
  if (turn == NULL) {
    screenwide_region_osc_add_selection_frame(vertices, count, size, rect, scale);
    return;
  }
  NSPoint corners[4];
  turn_carry_rect(turn, rect, corners);
  screenwide_region_osc_add_turned_outline(vertices, count, size, corners, scale);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_osc_add_marquee(ScreenwideRegionOscVertex *vertices,
                                                           NSUInteger *count, NSSize size,
                                                           const double *turn, NSRect band,
                                                           CGFloat scale) {
  if (turn == NULL) {
    // The region selector's own marching ants, unfilled. The band is its own
    // image, so no shade is drawn round it, and it has no handles to offer.
    screenwide_region_osc_add_crop_with_handles(vertices, count, size, band, band, scale, 0.0,
                                                YES, NO);
    return;
  }
  NSPoint corners[4];
  turn_carry_rect(turn, band, corners);
  screenwide_region_osc_add_turned_marquee(vertices, count, size, corners, scale);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_osc_add_box(ScreenwideRegionOscVertex *vertices,
                                                       NSUInteger *count, NSSize size,
                                                       const double *turn, NSRect frame,
                                                       double radius_percent, BOOL has_radius,
                                                       CGFloat scale) {
  if (turn == NULL) {
    screenwide_region_osc_add_selection(vertices, count, size, frame, scale,
                                        has_radius ? radius_percent : 0.0, has_radius);
    return;
  }
  NSPoint corners[4];
  turn_carry_rect(turn, frame, corners);
  // The radius dot sits where the upright selection draws it.
  CGFloat offset = MIN(frame.size.width, frame.size.height) * radius_percent / 100.0 * 0.55 + 10.0;
  NSPoint dot = turn_carry(turn, NSMakePoint(NSMinX(frame) + offset, NSMinY(frame) + offset));
  screenwide_region_osc_add_turned_selection(vertices, count, size, corners,
                                             has_radius ? &dot : NULL, scale);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_osc_add_image_frame(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size, const double *turn,
    const NSPoint corners[4], NSPoint dot, CGFloat scale) {
  NSPoint placed[4];
  for (NSUInteger index = 0; index < 4; index++)
    placed[index] = turn != NULL ? turn_carry(turn, corners[index]) : corners[index];
  if (turn != NULL) dot = turn_carry(turn, dot);
  screenwide_region_osc_add_turned_selection(vertices, count, size, placed, &dot, scale);
}
