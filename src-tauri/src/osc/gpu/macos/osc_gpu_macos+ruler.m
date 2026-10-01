// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos_private.h"

void screenwide_region_osc_add_ruler_box(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect frame, CGFloat scale, BOOL hovered, CGFloat hover_width) {
  CGFloat min_x = screenwide_region_osc_snap(NSMinX(frame), scale);
  CGFloat max_x = screenwide_region_osc_snap(NSMaxX(frame), scale);
  CGFloat min_y = screenwide_region_osc_snap(NSMinY(frame), scale);
  CGFloat max_y = screenwide_region_osc_snap(NSMaxY(frame), scale);
  CGFloat halo_width = hovered ? hover_width : 3.0 / scale;
  CGFloat margin = halo_width * 0.5 + 1.0 / scale;
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(min_x - margin, min_y - margin,
                 max_x - min_x + margin * 2.0,
                 max_y - min_y + margin * 2.0),
      hovered ? 34 : 35);

  CGFloat half = 0.5 / scale;
  CGFloat vertical_height = MAX(max_y - min_y - half * 2.0, 0.0);
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(min_x - half, min_y - half,
                 max_x - min_x + half * 2.0, half * 2.0),
      28);
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(min_x - half, max_y - half,
                 max_x - min_x + half * 2.0, half * 2.0),
      28);
  if (vertical_height > 0.0) {
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(min_x - half, min_y + half, half * 2.0,
                   vertical_height),
        28);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(max_x - half, min_y + half, half * 2.0,
                   vertical_height),
        28);
  }
}

static void add_ruler_arc_quad(ScreenwideRegionOscVertex *vertices,
                               NSUInteger *count, NSSize size,
                               NSPoint center, CGFloat radius,
                               uint8_t corner, CGFloat margin,
                               uint32_t kind) {
  BOOL right = corner == 2 || corner == 4;
  BOOL bottom = corner == 3 || corner == 4;
  CGFloat sign_x = right ? 1.0 : -1.0;
  CGFloat sign_y = bottom ? 1.0 : -1.0;
  CGFloat min_x = right ? center.x - margin : center.x - radius - margin;
  CGFloat max_x = right ? center.x + radius + margin : center.x + margin;
  CGFloat min_y = bottom ? center.y - margin : center.y - radius - margin;
  CGFloat max_y = bottom ? center.y + radius + margin : center.y + margin;
  ScreenwideRegionOscPoint a = ndc(size, min_x, min_y);
  ScreenwideRegionOscPoint b = ndc(size, max_x, min_y);
  ScreenwideRegionOscPoint c = ndc(size, max_x, max_y);
  ScreenwideRegionOscPoint d = ndc(size, min_x, max_y);
  ScreenwideRegionOscPoint uv_a = {
      (float)((min_x - center.x) * sign_x / radius),
      (float)((min_y - center.y) * sign_y / radius)};
  ScreenwideRegionOscPoint uv_b = {
      (float)((max_x - center.x) * sign_x / radius),
      (float)((min_y - center.y) * sign_y / radius)};
  ScreenwideRegionOscPoint uv_c = {
      (float)((max_x - center.x) * sign_x / radius),
      (float)((max_y - center.y) * sign_y / radius)};
  ScreenwideRegionOscPoint uv_d = {
      (float)((min_x - center.x) * sign_x / radius),
      (float)((max_y - center.y) * sign_y / radius)};
  ScreenwideRegionOscVertex quad[6] = {
      {a, uv_a, kind, 0}, {b, uv_b, kind, 0},
      {c, uv_c, kind, 0}, {a, uv_a, kind, 0},
      {c, uv_c, kind, 0}, {d, uv_d, kind, 0},
  };
  memcpy(vertices + *count, quad, sizeof(quad));
  *count += 6;
}

void screenwide_region_osc_add_ruler_arc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSPoint center, CGFloat radius, uint8_t corner, CGFloat scale,
    BOOL hovered, CGFloat hover_width, BOOL low_confidence) {
  if (radius <= 0.0 || scale <= 0.0)
    return;
  center.x = screenwide_region_osc_snap(center.x, scale);
  center.y = screenwide_region_osc_snap(center.y, scale);
  radius = MAX(round(radius * scale) / scale, 1.0 / scale);
  if (hovered) {
    CGFloat margin = hover_width * 0.5 + 1.0 / scale;
    add_ruler_arc_quad(vertices, count, size, center, radius, corner,
                       margin, 40);
  }
  add_ruler_arc_quad(vertices, count, size, center, radius, corner,
                     1.5 / scale, low_confidence ? 41 : 39);
}
