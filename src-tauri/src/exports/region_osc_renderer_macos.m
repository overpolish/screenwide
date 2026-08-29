// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "region_osc_renderer_macos.h"
#import "region_osc_renderer_macos_shader.h"

_Static_assert(sizeof(ScreenwideRegionOscVertex) == 24,
               "Region OSC vertices must match the Metal struct stride");
_Static_assert(sizeof(ScreenwideOscControlPalette) == 32,
               "OSC palette must match the Rust FFI layout");

NSString *screenwide_region_osc_shader_source(void) {
  return ScreenwideRegionOscMetalSource;
}

id<MTLTexture> screenwide_region_osc_make_placeholder(id<MTLDevice> device) {
  MTLTextureDescriptor *descriptor = [MTLTextureDescriptor
      texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                   width:1
                                  height:1
                               mipmapped:NO];
  descriptor.usage = MTLTextureUsageShaderRead;
  id<MTLTexture> texture = [device newTextureWithDescriptor:descriptor];
  const uint8_t transparent[4] = {0, 0, 0, 0};
  [texture replaceRegion:MTLRegionMake2D(0, 0, 1, 1)
              mipmapLevel:0
                withBytes:transparent
              bytesPerRow:4];
  return texture;
}

static ScreenwideRegionOscPoint ndc(NSSize size, CGFloat x, CGFloat y) {
  return (ScreenwideRegionOscPoint){
      (float)(2.0 * x / MAX(size.width, 1.0) - 1.0),
      (float)(1.0 - 2.0 * y / MAX(size.height, 1.0)),
  };
}

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

static void add_pattern_quad(ScreenwideRegionOscVertex *vertices,
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

static void add_circle(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
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

static void add_pill(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
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

static NSPoint snap_handle_point(NSPoint point, CGFloat scale) {
  return NSMakePoint(snap_handle_center(point.x, scale),
                     snap_handle_center(point.y, scale));
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
  for (NSUInteger pass = 0; pass < 2; pass++) {
    BOOL halo = pass == 0;
    CGFloat half = (halo ? 1.5 : 0.5) / scale;
    uint32_t rect_kind = halo ? 2 : 0;
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
  CGFloat radius = 4.0 + 1.0 / scale;
  for (NSUInteger index = 0; index < 8; index++) {
    NSPoint point = snap_handle_point(points[index], scale);
    if ((index & 1) == 0)
      add_circle(vertices, count, size, point, radius, 1.0 / scale, 3);
    else
      add_pill(vertices, count, size, point, index == 1 || index == 5,
               scale);
  }
  if (radius_enabled) {
    CGFloat offset = MIN(max_x - min_x, max_y - min_y) * radius_percent /
                         100.0 * 0.55 +
                     10.0;
    add_circle(vertices, count, size,
               snap_handle_point(NSMakePoint(min_x + offset, min_y + offset),
                                 scale),
               radius, 1.0 / scale, 3);
  }
}

void screenwide_region_osc_add_crop_with_handles(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect crop, NSRect image, CGFloat scale, BOOL show_frame,
    BOOL show_handles) {
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
  add_pattern_quad(vertices, count, size,
                   NSMakeRect(min_x - half, min_y - half,
                              max_x - min_x + half * 2.0, half * 2.0),
                   8, YES, scale, min_x);
  add_pattern_quad(vertices, count, size,
                   NSMakeRect(min_x - half, max_y - half,
                              max_x - min_x + half * 2.0, half * 2.0),
                   8, YES, scale, min_x);
  add_pattern_quad(vertices, count, size,
                   NSMakeRect(min_x - half, min_y - half, half * 2.0,
                              max_y - min_y + half * 2.0),
                   10, NO, scale, min_y);
  add_pattern_quad(vertices, count, size,
                   NSMakeRect(max_x - half, min_y - half, half * 2.0,
                              max_y - min_y + half * 2.0),
                   10, NO, scale, min_y);
  if (show_handles) {
    CGFloat radius = 4.0 + 1.0 / scale;
    for (NSUInteger index = 0; index < 8; index++) {
      NSPoint point = snap_handle_point(points[index], scale);
      if ((index & 1) == 0)
        add_circle(vertices, count, size, point, radius, 1.0 / scale, 3);
      else
        add_pill(vertices, count, size, point, index == 1 || index == 5,
                 scale);
    }
  }
}

void screenwide_region_osc_add_crop(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    NSRect crop, NSRect image, CGFloat scale) {
  screenwide_region_osc_add_crop_with_handles(
      vertices, count, size, crop, image, scale, YES, YES);
}

void screenwide_region_osc_encode(
    id<MTLRenderCommandEncoder> encoder,
    id<MTLRenderPipelineState> pipeline, id<MTLBuffer> vertices,
    NSUInteger vertex_count, ScreenwideRegionOscRenderState state,
    id<MTLTexture> label, id<MTLTexture> secondary_label) {
  [encoder setRenderPipelineState:pipeline];
  [encoder setVertexBuffer:vertices offset:0 atIndex:0];
  [encoder setFragmentBytes:&state.light_mode
                     length:sizeof(state.light_mode)
                    atIndex:0];
  [encoder setFragmentBytes:state.magnifier_box
                     length:sizeof(state.magnifier_box)
                    atIndex:1];
  [encoder setFragmentBytes:state.action_shades
                     length:sizeof(state.action_shades)
                    atIndex:2];
  float control_colors[8];
  memcpy(control_colors, state.control_fill, sizeof(state.control_fill));
  memcpy(control_colors + 4, state.control_outline,
         sizeof(state.control_outline));
  [encoder setFragmentBytes:control_colors
                     length:sizeof(control_colors)
                    atIndex:3];
  [encoder setFragmentTexture:label atIndex:0];
  [encoder setFragmentTexture:secondary_label atIndex:1];
  [encoder drawPrimitives:MTLPrimitiveTypeTriangle
              vertexStart:0
              vertexCount:vertex_count];
}
