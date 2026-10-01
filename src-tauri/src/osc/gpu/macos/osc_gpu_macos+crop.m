// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos_private.h"

/// Shades the sliver between a crop corner and the rounded corner the layer
/// will actually have.
///
/// The four shade quads stop at the crop rectangle, so with a corner radius
/// the little wedge inside the rectangle but outside the rounded shape would
/// read as kept. `uv` is the distance from the arc centre in radii, which
/// lets the fragment shade exactly what falls outside the arc.
static void add_crop_corner_shade(ScreenwideRegionOscVertex *vertices,
                                  NSUInteger *count, NSSize size,
                                  NSPoint center, CGFloat radius,
                                  CGFloat sign_x, CGFloat sign_y) {
  if (radius <= 0.0)
    return;
  CGFloat outer_x = center.x + radius * sign_x;
  CGFloat outer_y = center.y + radius * sign_y;
  CGFloat min_x = MIN(center.x, outer_x), max_x = MAX(center.x, outer_x);
  CGFloat min_y = MIN(center.y, outer_y), max_y = MAX(center.y, outer_y);
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
      {a, uv_a, 45, 0}, {b, uv_b, 45, 0}, {c, uv_c, 45, 0},
      {a, uv_a, 45, 0}, {c, uv_c, 45, 0}, {d, uv_d, 45, 0},
  };
  memcpy(vertices + *count, quad, sizeof(quad));
  *count += 6;
}

void screenwide_region_osc_add_crop_with_handles(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect crop, NSRect image, CGFloat scale, double radius_percent,
    BOOL show_frame, BOOL show_handles) {
  NSRect shade[4] = {
      NSMakeRect(NSMinX(image), NSMinY(image), image.size.width,
                 MAX(NSMinY(crop) - NSMinY(image), 0.0)),
      NSMakeRect(NSMinX(image), NSMaxY(crop), image.size.width,
                 MAX(NSMaxY(image) - NSMaxY(crop), 0.0)),
      NSMakeRect(NSMinX(image), NSMinY(crop),
                 MAX(NSMinX(crop) - NSMinX(image), 0.0), crop.size.height),
      NSMakeRect(NSMaxX(crop), NSMinY(crop),
                 MAX(NSMaxX(image) - NSMaxX(crop), 0.0), crop.size.height),
  };
  for (NSUInteger index = 0; index < 4; index++)
    if (!NSIsEmptyRect(shade[index]))
      screenwide_region_osc_add_quad(vertices, count, size, shade[index], 6);

  CGFloat shortest = MIN(crop.size.width, crop.size.height);
  CGFloat corner_radius =
      MIN(MAX(radius_percent, 0.0), 50.0) / 100.0 * shortest;
  if (corner_radius > 0.0) {
    const CGFloat signs[4][2] = {{-1.0, -1.0}, {1.0, -1.0},
                                 {-1.0, 1.0}, {1.0, 1.0}};
    NSPoint centers[4] = {
        {NSMinX(crop) + corner_radius, NSMinY(crop) + corner_radius},
        {NSMaxX(crop) - corner_radius, NSMinY(crop) + corner_radius},
        {NSMinX(crop) + corner_radius, NSMaxY(crop) - corner_radius},
        {NSMaxX(crop) - corner_radius, NSMaxY(crop) - corner_radius},
    };
    for (NSUInteger index = 0; index < 4; index++)
      add_crop_corner_shade(vertices, count, size, centers[index],
                            corner_radius, signs[index][0], signs[index][1]);
  }

  if (!show_frame)
    return;

  CGFloat min_x = screenwide_region_osc_snap(NSMinX(crop), scale);
  CGFloat max_x = screenwide_region_osc_snap(NSMaxX(crop), scale);
  CGFloat min_y = screenwide_region_osc_snap(NSMinY(crop), scale);
  CGFloat max_y = screenwide_region_osc_snap(NSMaxY(crop), scale);
  CGFloat mid_x = screenwide_region_osc_snap((min_x + max_x) / 2.0, scale);
  CGFloat mid_y = screenwide_region_osc_snap((min_y + max_y) / 2.0, scale);
  NSPoint points[8] = {{min_x, min_y}, {mid_x, min_y}, {max_x, min_y},
                       {max_x, mid_y}, {max_x, max_y}, {mid_x, max_y},
                       {min_x, max_y}, {min_x, mid_y}};
  CGFloat half = 1.5 / scale;
  screenwide_region_osc_add_pattern_quad(vertices, count, size,
                   NSMakeRect(min_x - half, min_y - half,
                              max_x - min_x + half * 2.0, half * 2.0),
                   8, YES, scale, min_x);
  screenwide_region_osc_add_pattern_quad(vertices, count, size,
                   NSMakeRect(min_x - half, max_y - half,
                              max_x - min_x + half * 2.0, half * 2.0),
                   8, YES, scale, min_x);
  screenwide_region_osc_add_pattern_quad(vertices, count, size,
                   NSMakeRect(min_x - half, min_y - half, half * 2.0,
                              max_y - min_y + half * 2.0),
                   10, NO, scale, min_y);
  screenwide_region_osc_add_pattern_quad(vertices, count, size,
                   NSMakeRect(max_x - half, min_y - half, half * 2.0,
                              max_y - min_y + half * 2.0),
                   10, NO, scale, min_y);
  if (show_handles) {
    CGFloat radius = 4.0 + 1.0 / scale;
    for (NSUInteger index = 0; index < 8; index++) {
      NSPoint point = screenwide_region_osc_snap_handle_point(points[index], scale);
      if ((index & 1) == 0)
        screenwide_region_osc_add_circle(vertices, count, size, point, radius, 1.0 / scale, 3);
      else
        screenwide_region_osc_add_pill(vertices, count, size, point, index == 1 || index == 5,
                 scale);
    }
  }
}

void screenwide_region_osc_add_crop(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    NSRect crop, NSRect image, CGFloat scale,
                                    double radius_percent) {
  screenwide_region_osc_add_crop_with_handles(
      vertices, count, size, crop, image, scale, radius_percent, YES, YES);
}
