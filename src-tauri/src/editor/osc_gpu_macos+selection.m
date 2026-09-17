// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos_private.h"

static void add_selection_frame(ScreenwideRegionOscVertex *vertices,
                                NSUInteger *count, NSSize size,
                                NSRect frame, CGFloat scale,
                                uint32_t halo_kind,
                                uint32_t line_kind, CGFloat halo_width) {
  CGFloat min_x = screenwide_region_osc_snap(NSMinX(frame), scale);
  CGFloat max_x = screenwide_region_osc_snap(NSMaxX(frame), scale);
  CGFloat min_y = screenwide_region_osc_snap(NSMinY(frame), scale);
  CGFloat max_y = screenwide_region_osc_snap(NSMaxY(frame), scale);
  for (NSUInteger pass = 0; pass < 2; pass++) {
    BOOL halo = pass == 0;
    CGFloat half = halo ? halo_width * 0.5 : 0.5 / scale;
    uint32_t rect_kind = halo ? halo_kind : line_kind;
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(min_x - half, min_y - half, max_x - min_x + half * 2.0,
                   half * 2.0),
        rect_kind);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(min_x - half, max_y - half, max_x - min_x + half * 2.0,
                   half * 2.0),
        rect_kind);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(min_x - half, min_y - half, half * 2.0,
                   max_y - min_y + half * 2.0),
        rect_kind);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(max_x - half, min_y - half, half * 2.0,
                   max_y - min_y + half * 2.0),
        rect_kind);
  }
}

void screenwide_region_osc_add_selection(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect frame, CGFloat scale, double radius_percent, BOOL radius_enabled) {
  CGFloat min_x = screenwide_region_osc_snap(NSMinX(frame), scale);
  CGFloat max_x = screenwide_region_osc_snap(NSMaxX(frame), scale);
  CGFloat min_y = screenwide_region_osc_snap(NSMinY(frame), scale);
  CGFloat max_y = screenwide_region_osc_snap(NSMaxY(frame), scale);
  CGFloat mid_x = screenwide_region_osc_snap((min_x + max_x) / 2.0, scale);
  CGFloat mid_y = screenwide_region_osc_snap((min_y + max_y) / 2.0, scale);
  NSPoint points[8] = {{min_x, min_y}, {mid_x, min_y}, {max_x, min_y},
                       {max_x, mid_y}, {max_x, max_y}, {mid_x, max_y},
                       {min_x, max_y}, {min_x, mid_y}};
  add_selection_frame(vertices, count, size, frame, scale, 2, 0,
                      3.0 / scale);
  CGFloat radius = 4.0 + 1.0 / scale;
  for (NSUInteger index = 0; index < 8; index++) {
    NSPoint point = screenwide_region_osc_snap_handle_point(points[index], scale);
    if ((index & 1) == 0)
      screenwide_region_osc_add_circle(vertices, count, size, point, radius, 1.0 / scale, 3);
    else
      screenwide_region_osc_add_pill(vertices, count, size, point, index == 1 || index == 5,
               scale);
  }
  if (radius_enabled) {
    CGFloat offset = MIN(max_x - min_x, max_y - min_y) * radius_percent /
                         100.0 * 0.55 +
                     10.0;
    screenwide_region_osc_add_circle(vertices, count, size,
               screenwide_region_osc_snap_handle_point(NSMakePoint(min_x + offset, min_y + offset),
                                 scale),
               radius, 1.0 / scale, 3);
  }
}
