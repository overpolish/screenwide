// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos_private.h"

void screenwide_region_osc_add_quad(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    NSRect rect, uint32_t kind) {
  ScreenwideRegionOscPoint a = ndc(size, NSMinX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint b = ndc(size, NSMaxX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint c = ndc(size, NSMaxX(rect), NSMaxY(rect));
  ScreenwideRegionOscPoint d = ndc(size, NSMinX(rect), NSMaxY(rect));
  ScreenwideRegionOscVertex quad[6] = {
      {a, {0, 0}, kind, 0}, {b, {1, 0}, kind, 0},
      {c, {1, 1}, kind, 0}, {a, {0, 0}, kind, 0},
      {c, {1, 1}, kind, 0}, {d, {0, 1}, kind, 0},
  };
  memcpy(vertices + *count, quad, sizeof(quad));
  *count += 6;
}

void screenwide_region_osc_add_texture_quad(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect rect, NSRect texture_rect, uint32_t kind) {
  ScreenwideRegionOscPoint a = ndc(size, NSMinX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint b = ndc(size, NSMaxX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint c = ndc(size, NSMaxX(rect), NSMaxY(rect));
  ScreenwideRegionOscPoint d = ndc(size, NSMinX(rect), NSMaxY(rect));
  float min_u = NSMinX(texture_rect);
  float min_v = NSMinY(texture_rect);
  float max_u = NSMaxX(texture_rect);
  float max_v = NSMaxY(texture_rect);
  ScreenwideRegionOscVertex quad[6] = {
      {a, {min_u, min_v}, kind, 0}, {b, {max_u, min_v}, kind, 0},
      {c, {max_u, max_v}, kind, 0}, {a, {min_u, min_v}, kind, 0},
      {c, {max_u, max_v}, kind, 0}, {d, {min_u, max_v}, kind, 0},
  };
  memcpy(vertices + *count, quad, sizeof(quad));
  *count += 6;
}

void screenwide_region_osc_add_line(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    NSPoint start, NSPoint end,
                                    CGFloat width, uint32_t kind) {
  CGFloat dx = end.x - start.x;
  CGFloat dy = end.y - start.y;
  CGFloat length = hypot(dx, dy);
  if (length <= 0.0001 || width <= 0.0)
    return;
  CGFloat half = width * 0.5;
  CGFloat ux = dx / length;
  CGFloat uy = dy / length;
  CGFloat px = -uy * half;
  CGFloat py = ux * half;
  NSPoint extendedStart = NSMakePoint(start.x - ux * half,
                                      start.y - uy * half);
  NSPoint extendedEnd = NSMakePoint(end.x + ux * half,
                                    end.y + uy * half);
  ScreenwideRegionOscPoint a =
      ndc(size, extendedStart.x + px, extendedStart.y + py);
  ScreenwideRegionOscPoint b =
      ndc(size, extendedEnd.x + px, extendedEnd.y + py);
  ScreenwideRegionOscPoint c =
      ndc(size, extendedEnd.x - px, extendedEnd.y - py);
  ScreenwideRegionOscPoint d =
      ndc(size, extendedStart.x - px, extendedStart.y - py);
  ScreenwideRegionOscVertex quad[6] = {
      {a, {0, 0}, kind, 0}, {b, {1, 0}, kind, 0},
      {c, {1, 1}, kind, 0}, {a, {0, 0}, kind, 0},
      {c, {1, 1}, kind, 0}, {d, {0, 1}, kind, 0},
  };
  memcpy(vertices + *count, quad, sizeof(quad));
  *count += 6;
}

void screenwide_region_osc_add_pattern_quad(ScreenwideRegionOscVertex *vertices,
                             NSUInteger *count, NSSize size, NSRect rect,
                             uint32_t kind, BOOL horizontal, CGFloat scale,
                             CGFloat origin) {
  ScreenwideRegionOscPoint a = ndc(size, NSMinX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint b = ndc(size, NSMaxX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint c = ndc(size, NSMaxX(rect), NSMaxY(rect));
  ScreenwideRegionOscPoint d = ndc(size, NSMinX(rect), NSMaxY(rect));
  float start = (float)(((horizontal ? NSMinX(rect) : NSMinY(rect)) - origin) *
                        scale / 12.0);
  float end = (float)(((horizontal ? NSMaxX(rect) : NSMaxY(rect)) - origin) *
                      scale / 12.0);
  ScreenwideRegionOscPoint uv_b = horizontal
      ? (ScreenwideRegionOscPoint){end, 0}
      : (ScreenwideRegionOscPoint){1, start};
  ScreenwideRegionOscPoint uv_c = horizontal
      ? (ScreenwideRegionOscPoint){end, 1}
      : (ScreenwideRegionOscPoint){1, end};
  ScreenwideRegionOscPoint uv_d = horizontal
      ? (ScreenwideRegionOscPoint){start, 1}
      : (ScreenwideRegionOscPoint){0, end};
  ScreenwideRegionOscVertex quad[6] = {
      {a, horizontal ? (ScreenwideRegionOscPoint){start, 0}
                     : (ScreenwideRegionOscPoint){0, start}, kind, 0},
      {b, uv_b, kind, 0}, {c, uv_c, kind, 0},
      {a, horizontal ? (ScreenwideRegionOscPoint){start, 0}
                     : (ScreenwideRegionOscPoint){0, start}, kind, 0},
      {c, uv_c, kind, 0}, {d, uv_d, kind, 0},
  };
  memcpy(vertices + *count, quad, sizeof(quad));
  *count += 6;
}

void screenwide_region_osc_add_circle(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                       NSSize size, NSPoint center, CGFloat radius,
                       CGFloat margin, uint32_t kind) {
  CGFloat extent = radius + margin;
  NSRect rect = NSMakeRect(center.x - extent, center.y - extent,
                           extent * 2.0, extent * 2.0);
  ScreenwideRegionOscPoint a = ndc(size, NSMinX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint b = ndc(size, NSMaxX(rect), NSMinY(rect));
  ScreenwideRegionOscPoint c = ndc(size, NSMaxX(rect), NSMaxY(rect));
  ScreenwideRegionOscPoint d = ndc(size, NSMinX(rect), NSMaxY(rect));
  ScreenwideRegionOscVertex quad[6] = {
      {a, {0, 0}, kind, 0}, {b, {1, 0}, kind, 0},
      {c, {1, 1}, kind, 0}, {a, {0, 0}, kind, 0},
      {c, {1, 1}, kind, 0}, {d, {0, 1}, kind, 0},
  };
  memcpy(vertices + *count, quad, sizeof(quad));
  *count += 6;
}

void screenwide_region_osc_add_pill(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                     NSSize size, NSPoint center, BOOL horizontal,
                     CGFloat scale) {
  CGFloat width = (horizontal ? 12.0 : 6.0) + 4.0 / scale;
  CGFloat height = (horizontal ? 6.0 : 12.0) + 4.0 / scale;
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(center.x - width * 0.5, center.y - height * 0.5, width,
                 height),
      16);
}

CGFloat screenwide_region_osc_snap(CGFloat value, CGFloat scale) {
  return (floor(value * scale) + 0.5) / scale;
}

static CGFloat snap_handle_center(CGFloat value, CGFloat scale) {
  return round(value * scale) / scale;
}

NSPoint screenwide_region_osc_snap_handle_point(NSPoint point, CGFloat scale) {
  return NSMakePoint(snap_handle_center(point.x, scale),
                     snap_handle_center(point.y, scale));
}
