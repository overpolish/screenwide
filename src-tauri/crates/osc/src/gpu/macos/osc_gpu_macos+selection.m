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

void screenwide_region_osc_add_selection_frame(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect frame, CGFloat scale) {
  add_selection_frame(vertices, count, size, frame, scale, 2, 0, 3.0 / scale);
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

/// The pixel either side of a soft line's quad that its edge fades across.
static CGFloat soft_pad(CGFloat scale) { return 2.0 / scale; }

/// The twin of Rust's `add_turned_outline`.
void screenwide_region_osc_add_turned_outline(ScreenwideRegionOscVertex *vertices,
                                              NSUInteger *count, NSSize size,
                                              const NSPoint corners[4], CGFloat scale) {
  const CGFloat widths[2] = {3.0 / scale, 1.0 / scale};
  const uint32_t kinds[2] = {51, 50};
  for (NSUInteger pass = 0; pass < 2; pass++)
    for (NSUInteger side = 0; side < 4; side++)
      screenwide_region_osc_add_line(vertices, count, size, corners[side],
                                     corners[(side + 1) % 4], widths[pass] + soft_pad(scale),
                                     kinds[pass]);
}

/// The twin of Rust's `add_turned_marquee`.
void screenwide_region_osc_add_turned_marquee(ScreenwideRegionOscVertex *vertices,
                                              NSUInteger *count, NSSize size,
                                              const NSPoint corners[4], CGFloat scale) {
  CGFloat width = 3.0 / scale + soft_pad(scale);
  CGFloat half = width * 0.5;
  for (NSUInteger side = 0; side < 4; side++) {
    NSPoint from = corners[side];
    NSPoint to = corners[(side + 1) % 4];
    CGFloat span = hypot(to.x - from.x, to.y - from.y);
    // The quad runs half its width past each corner, so the periods are
    // counted from there to keep the pattern anchored on the corner.
    screenwide_region_osc_add_line_along(vertices, count, size, from, to, width, 63,
                                         (float)(-half * scale / 12.0),
                                         (float)((span + half) * scale / 12.0));
  }
}

/// The twin of Rust's `add_soft_guide`.
void screenwide_region_osc_add_soft_guide(ScreenwideRegionOscVertex *vertices,
                                          NSUInteger *count, NSSize size, NSPoint from,
                                          NSPoint to, CGFloat width, BOOL object,
                                          CGFloat scale) {
  screenwide_region_osc_add_line(vertices, count, size, from, to, width + soft_pad(scale),
                                 object ? 62 : 61);
}

/// The twin of Rust's `add_turned_selection`. Its sides need not run along
/// the pixel grid, so they are soft lines, each quad padded a pixel either
/// side for the shader to anti-alias across, rather than snapped quads.
void screenwide_region_osc_add_turned_selection(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    const NSPoint corners[4], const NSPoint *radius_dot, CGFloat scale) {
  screenwide_region_osc_add_turned_outline(vertices, count, size, corners, scale);
  CGFloat radius = 4.0 + 1.0 / scale;
  // A side's pill lies along it, the length and thickness of an upright one.
  CGFloat length = 12.0 + 4.0 / scale;
  CGFloat thickness = 6.0 + 4.0 / scale;
  for (NSUInteger side = 0; side < 4; side++) {
    NSPoint from = corners[side];
    NSPoint to = corners[(side + 1) % 4];
    screenwide_region_osc_add_circle(
        vertices, count, size, screenwide_region_osc_snap_handle_point(from, scale), radius,
        1.0 / scale, 3);
    CGFloat span = hypot(to.x - from.x, to.y - from.y);
    if (span <= 0.0001) continue;
    CGFloat reach = (length - thickness) * 0.5 / span;
    NSPoint middle = NSMakePoint((from.x + to.x) * 0.5, (from.y + to.y) * 0.5);
    NSPoint along = NSMakePoint((to.x - from.x) * reach, (to.y - from.y) * reach);
    screenwide_region_osc_add_line(vertices, count, size,
                                   NSMakePoint(middle.x - along.x, middle.y - along.y),
                                   NSMakePoint(middle.x + along.x, middle.y + along.y),
                                   thickness, 16);
  }
  if (radius_dot != NULL)
    screenwide_region_osc_add_circle(vertices, count, size,
                                     screenwide_region_osc_snap_handle_point(*radius_dot, scale),
                                     radius, 1.0 / scale, 3);
}
