// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>

// Shared OSC contract. Region and Editor build scene geometry here and draw
// it through the shared WGSL OSC renderer (`osc/gpu/macos.rs`); tool-specific
// interaction remains in their surface adapters.

typedef struct {
  float x;
  float y;
} ScreenwideRegionOscPoint;

/// The twin of Rust's OSC `Vertex`.
typedef struct {
  ScreenwideRegionOscPoint position;
  ScreenwideRegionOscPoint uv;
  uint32_t kind;
  uint32_t padding;
} ScreenwideRegionOscVertex;
_Static_assert(sizeof(ScreenwideRegionOscVertex) == 24,
               "OSC vertices must match the Rust layout");

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
  float overlay_shade[4];
  float action_fills[8];
  float control_fill[4];
  float control_outline[4];
  float ocr_colors[32];
  float ruler_colors[8];
  float ruler_sample[4];
  float ruler_animation[4];
} ScreenwideRegionOscRenderState;

typedef struct {
  float fill[4];
  float outline[4];
} ScreenwideOscControlPalette;
_Static_assert(sizeof(ScreenwideOscControlPalette) == 32,
               "OSC palette must match the Rust FFI layout");

typedef struct {
  float shade[4];
} ScreenwideOscOverlayPalette;

typedef struct {
  float primary_fill[4];
  float primary_outline[4];
  float qr_fill[4];
  float qr_outline[4];
  float error_fill[4];
  float error_outline[4];
  float selection_fill[4];
  float selection_outline[4];
  float loading_fill[4];
  float loading_foreground[4];
  float status_error_fill[4];
  float status_error_foreground[4];
} ScreenwideOscOcrPalette;
typedef struct {
  float primary[4];
  float info[4];
} ScreenwideOscRulerPalette;
_Static_assert(sizeof(ScreenwideOscOcrPalette) == 192,
               "ScreenwideOscOcrPalette ABI must match Rust");

ScreenwideOscControlPalette screenwide_osc_control_palette(
    uint32_t light_mode);
ScreenwideOscOverlayPalette screenwide_osc_overlay_palette(void);
ScreenwideOscOcrPalette screenwide_osc_ocr_palette(uint32_t light_mode);
ScreenwideOscRulerPalette screenwide_osc_ruler_palette(uint32_t light_mode);
ScreenwideRegionOscRenderState screenwide_region_osc_render_state(
    uint32_t light_mode);
NSCursor *screenwide_region_resize_cursor(uint32_t edges);

/// The shared wgpu device's Metal device and queue. Every OSC layer and
/// texture is made on this device, and a command buffer committed to this
/// queue runs after the OSC draws made before it.
void *screenwide_shared_metal_device(void);
void *screenwide_shared_metal_queue(void);

/// What a draw does with its magnifier.
enum {
  /// Nothing.
  ScreenwideOscLensNone = 0,
  /// Keeps the lens's box clear for a lens drawn before.
  ScreenwideOscLensCutout = 1,
  /// Draws the lens last, from the magnifier source.
  ScreenwideOscLensDrawn = 2,
};

/// Draws `count` vertices into `target`, a drawable's `id<MTLTexture>`, on
/// the shared queue, clearing it first when `clear` is set. `label`,
/// `secondary_label`, `snapshot` and `magnifier_source` are `id<MTLTexture>`s
/// or NULL. Present with a command buffer from the shared queue committed
/// afterwards. Answers 0 when nothing was drawn.
int screenwide_osc_draw(void *target, int clear,
                        const ScreenwideRegionOscVertex *vertices, uint32_t count,
                        const ScreenwideRegionOscRenderState *state,
                        const ScreenwideRegionMagnifier *magnifier, uint32_t lens,
                        void *label, void *secondary_label, void *snapshot,
                        void *magnifier_source);

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
void screenwide_region_osc_add_texture_quad(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count,
    NSSize view_size, NSRect rect, NSRect texture_rect, uint32_t kind);
void screenwide_region_osc_add_line(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize view_size,
                                    NSPoint start, NSPoint end,
                                    CGFloat width, uint32_t kind);
void screenwide_region_osc_add_selection(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize view_size,
    NSRect frame, CGFloat scale, double radius_percent, BOOL radius_enabled);
/// The selection's own frame - a light core over a dark halo - without its
/// handles or radius dot.
void screenwide_region_osc_add_selection_frame(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize view_size,
    NSRect frame, CGFloat scale);
/// A turned picture's own frame: the selection's outline along its four
/// sides, its corner discs, the pills on its sides lying along them, and its
/// radius dot. `corners` run clockwise from the picture's own top-left.
void screenwide_region_osc_add_turned_selection(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize view_size,
    const NSPoint corners[4], NSPoint radius_dot, CGFloat scale);
void screenwide_region_osc_add_ruler_box(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize view_size,
    NSRect frame, CGFloat scale, BOOL hovered, CGFloat hover_width);
void screenwide_region_osc_add_ruler_arc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count,
    NSSize view_size, NSPoint center, CGFloat radius, uint8_t corner,
    CGFloat scale, BOOL hovered, CGFloat hover_width, BOOL low_confidence);
/// `radius_percent` is the layer's corner radius as a percentage of the crop
/// rectangle's shorter side. The shade follows it into the corners so the
/// preview under the crop window matches the rounded result.
void screenwide_region_osc_add_crop(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize view_size,
                                    NSRect crop, NSRect image, CGFloat scale,
                                    double radius_percent);
void screenwide_region_osc_add_crop_with_handles(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize view_size,
    NSRect crop, NSRect image, CGFloat scale, double radius_percent,
    BOOL show_frame, BOOL show_handles);

#import "osc_pixel_alignment_macos.h"
