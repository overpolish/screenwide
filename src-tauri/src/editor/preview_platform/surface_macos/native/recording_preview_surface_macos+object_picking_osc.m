// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The objects picked in the camera, drawn over it in the OSC while they are
// picked: a dot for each click, the line for each painting.

#import "recording_preview_surface_macos_private.h"
#import "recording_preview_object_picking_macos.h"

#include <math.h>

/// The OSC vertices a surface draws with, which the objects keep within.
static const NSUInteger kVertexRoom = 4096;

/// How far the halo round the chosen object reaches past it, in points: as
/// far as an annotation's hover halo grows.
static const CGFloat kHalo = 8.0;

/// How many points of the chosen line its halo is drawn through, which the
/// OSC's constants have room for.
static const NSUInteger kHaloPoints = 64;

/// The halo round the chosen line, in its colour: one quad over the line and
/// the reach round it, inside which the shader measures each pixel to the
/// line through `points`, which it is handed in the render `state`.
static void add_line_halo(ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
                          ScreenwideRegionOscRenderState *state, CGFloat scale, NSRect image,
                          const ScreenwidePickedPoint *points, NSUInteger total, BOOL removed) {
  CGFloat reach = 1.5 + kHalo;
  NSUInteger shown = MIN(total, kHaloPoints);
  NSRect bounds = NSZeroRect;
  for (NSUInteger index = 0; index < shown; index++) {
    // Spread over the line where it has more points than there is room for.
    NSUInteger from = shown > 1 ? index * (total - 1) / (shown - 1) : 0;
    NSPoint at = object_picking_view_point(image, points[from]);
    state->picked_points[index * 2] = (float)(at.x * scale);
    state->picked_points[index * 2 + 1] = (float)(at.y * scale);
    NSRect around = NSMakeRect(at.x - reach, at.y - reach, reach * 2.0, reach * 2.0);
    bounds = index == 0 ? around : NSUnionRect(bounds, around);
  }
  state->picked_halo[0] = (float)(reach * scale);
  state->picked_halo[1] = (float)shown;
  screenwide_region_osc_add_texture_quad(vertices, count, size, NSInsetRect(bounds, -1.0, -1.0),
                                         NSMakeRect(0.0, 0.0, 1.0, 1.0), removed ? 59 : 57);
}

/// One object: a dot where it was clicked, or the line painted over it,
/// each haloed in its own colour where it is chosen, as an annotation is.
static void add_object(ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
                       ScreenwideRegionOscRenderState *state, CGFloat scale, NSRect image,
                       const ScreenwidePickedPoint *points, NSUInteger total, BOOL removed,
                       BOOL chosen) {
  CGFloat pad = 2.0 / scale;
  if (total > 1) {
    if (chosen && *count + 6 <= kVertexRoom)
      add_line_halo(vertices, count, size, state, scale, image, points, total, removed);
    // A line three points wide in the object's colour over the controls'
    // outline, which keeps it apart from the camera behind it.
    const CGFloat widths[2] = {5.0, 3.0};
    const uint32_t kinds[2] = {51, removed ? 56 : 55};
    for (NSUInteger pass = 0; pass < 2; pass++)
      for (NSUInteger index = 0; index + 1 < total && *count + 6 <= kVertexRoom; index++)
        screenwide_region_osc_add_line(vertices, count, size,
                                       object_picking_view_point(image, points[index]),
                                       object_picking_view_point(image, points[index + 1]),
                                       widths[pass] + pad, kinds[pass]);
    return;
  }
  if (*count + 12 > kVertexRoom) return;
  // The size of a moment's dot on the timeline, its centre on a whole device
  // pixel, over its halo where it is chosen.
  CGFloat radius = 4.0;
  NSPoint at = object_picking_view_point(image, points[0]);
  CGFloat x = round(at.x * scale) / scale;
  CGFloat y = round(at.y * scale) / scale;
  NSRect whole = NSMakeRect(0.0, 0.0, 1.0, 1.0);
  if (chosen) {
    CGFloat halo = radius + kHalo;
    screenwide_region_osc_add_texture_quad(
        vertices, count, size, NSMakeRect(x - halo, y - halo, halo * 2.0, halo * 2.0), whole,
        removed ? 58 : 54);
  }
  screenwide_region_osc_add_texture_quad(
      vertices, count, size, NSMakeRect(x - radius, y - radius, radius * 2.0, radius * 2.0), whole,
      removed ? 53 : 52);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL object_picking_add_osc(ScreenwideRegionOscVertex *vertices,
                                                       NSUInteger *count,
                                                       ScreenwideRegionOscRenderState *state,
                                                       ScreenwidePreviewSurface *surface,
                                                       NSSize size, CGFloat scale) {
  ScreenwideObjectPicking *picking = object_picking_state(surface);
  if (!picking.active) return NO;
  NSRect image;
  if (!object_picking_camera_image(surface, &image)) return YES;
  const ScreenwidePickedObject *objects = picking.objects.bytes;
  NSUInteger total = picking.objects.length / sizeof(ScreenwidePickedObject);
  // A pick on its way is the one chosen, until it arrives or another is.
  BOOL pendingChosen = picking.hasPending && picking.pendingChosen;
  for (NSUInteger index = 0; index < total; index++) {
    NSUInteger points = 0;
    const ScreenwidePickedPoint *at = object_picking_points(picking, &objects[index], &points);
    add_object(vertices, count, size, state, scale, image, at, points, objects[index].removed != 0,
               objects[index].chosen != 0 && !pendingChosen);
  }
  if (picking.hasPending)
    add_object(vertices, count, size, state, scale, image, picking.pendingPoints.bytes,
               picking.pendingPoints.length / sizeof(ScreenwidePickedPoint),
               picking.pendingRemoves, pendingChosen);
  return YES;
}

