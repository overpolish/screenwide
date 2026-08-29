// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>

typedef struct {
  float x;
  float y;
} ScreenwideRegionOscPoint;

typedef struct {
  ScreenwideRegionOscPoint position;
  ScreenwideRegionOscPoint uv;
  uint32_t kind;
  uint32_t padding;
} ScreenwideRegionOscVertex;

typedef struct {
  uint32_t active;
  uint32_t pane_index;
  uint32_t layer_id;
  uint32_t sample_camera;
  uint32_t edges;
  uint32_t light_mode;
  float sample_u;
  float sample_v;
  float source_min_u;
  float source_min_v;
  float source_max_u;
  float source_max_v;
  int32_t box_x;
  int32_t box_y;
  uint32_t box_width;
  uint32_t box_height;
} ScreenwideRegionMagnifier;

typedef struct {
  uint32_t light_mode;
  float magnifier_box[4];
  float action_shades[4];
  float control_fill[4];
  float control_outline[4];
} ScreenwideRegionOscRenderState;

typedef struct {
  float fill[4];
  float outline[4];
} ScreenwideOscControlPalette;

ScreenwideOscControlPalette screenwide_osc_control_palette(
    uint32_t light_mode);
ScreenwideRegionOscRenderState screenwide_region_osc_render_state(
    uint32_t light_mode);

NSString *screenwide_region_osc_shader_source(void);
id<MTLRenderPipelineState> screenwide_region_osc_make_pipeline(
    id<MTLDevice> device, id<MTLLibrary> library, NSError **error);
id<MTLComputePipelineState> screenwide_region_magnifier_make_pipeline(
    id<MTLDevice> device, id<MTLLibrary> library, NSError **error);
id<MTLTexture> screenwide_region_osc_make_placeholder(id<MTLDevice> device);
NSCursor *screenwide_region_resize_cursor(uint32_t edges);

NSPoint screenwide_region_magnifier_anchor(NSPoint point, NSRect frame,
                                           uint32_t edges);
ScreenwideRegionMagnifier screenwide_region_magnifier_make(
    NSPoint point, CGFloat scale, uint32_t edges, uint32_t light_mode,
    uint32_t pane_index, uint32_t layer_id, uint32_t sample_camera,
    float sample_u, float sample_v, float source_min_u, float source_min_v,
    float source_max_u, float source_max_v);

CGFloat screenwide_region_osc_snap(CGFloat value, CGFloat scale);
void screenwide_region_osc_add_quad(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize view_size,
                                    NSRect rect, uint32_t kind);
void screenwide_region_osc_add_selection(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize view_size,
    NSRect frame, CGFloat scale, double radius_percent, BOOL radius_enabled);
void screenwide_region_osc_add_crop(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize view_size,
                                    NSRect crop, NSRect image, CGFloat scale);
void screenwide_region_osc_add_crop_with_handles(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize view_size,
    NSRect crop, NSRect image, CGFloat scale, BOOL show_frame,
    BOOL show_handles);

void screenwide_region_osc_encode(
    id<MTLRenderCommandEncoder> encoder,
    id<MTLRenderPipelineState> pipeline, id<MTLBuffer> vertices,
    NSUInteger vertex_count, ScreenwideRegionOscRenderState state,
    id<MTLTexture> label, id<MTLTexture> secondary_label);
void screenwide_region_magnifier_encode(
    id<MTLComputeCommandEncoder> encoder,
    id<MTLComputePipelineState> pipeline, id<MTLBuffer> source,
    id<MTLTexture> output, const uint32_t source_dimensions[2],
    ScreenwideRegionMagnifier magnifier);
