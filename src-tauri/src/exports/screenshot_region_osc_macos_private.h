// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <QuartzCore/CAMetalLayer.h>
#include <stddef.h>
#import "osc_controls.h"
#import "region_osc_renderer_macos.h"
#import "osc_material_surface_macos.h"
#import "osc_text_texture_macos.h"

typedef struct {
  uint8_t status, gesture, handle, cursor, has_region;
  double x, y, width, height;
} NativeOscResult;
typedef void (*NativeOscInput)(void *, uint32_t, double, double, uint8_t,
                               NativeOscResult *);
typedef void (*NativeOscLayout)(void *);
typedef struct {
  uint32_t id;
  double x, y, width, height, scale;
} ScreenwideRegionDesktopDisplay;
typedef struct {
  double x, y, width, height;
  uint8_t kind;
  uint8_t padding[7];
} ScreenwideRegionOcrRect;
typedef struct {
  double x, y, width, height;
} ScreenwideOcrToolbarRect;
_Static_assert(sizeof(ScreenwideRegionOcrRect) == 40,
               "ScreenwideRegionOcrRect ABI must match Rust");
_Static_assert(offsetof(ScreenwideRegionOcrRect, kind) == 32,
               "ScreenwideRegionOcrRect.kind ABI must match Rust");
void screenwide_set_region_expected_cursor(NSCursor *cursor);

@interface ScreenwideRegionOSC : NSObject
@property(nonatomic, weak) NSView *host;
@property(nonatomic, strong) CAMetalLayer *layer;
@property(nonatomic, strong) CALayer *snapshotLayer;
@property(nonatomic, strong) NSMutableData *ocrRects;
@property(nonatomic) uint32_t ocrPhase;
@property(nonatomic, strong) ScreenwideOscMaterialSurfaceView *ocrStatusSurface;
@property(nonatomic, strong) NSTextField *ocrStatusLabel;
@property(nonatomic, strong) ScreenwideOscMaterialSurfaceView *ocrCancelSurface;
@property(nonatomic) void *ocrCancelControls;
@property(nonatomic) NSRect ocrCancelRect;
@property(nonatomic) BOOL ocrCancelVisible;
@property(nonatomic) uint64_t ocrCancelAnimationRevision;
@property(nonatomic, strong) ScreenwideOscTextTexture *ocrCancelLabel;
@property(nonatomic) CGFloat ocrCancelLabelScale;
@property(nonatomic) uint32_t ocrCancelLabelLightMode;
@property(nonatomic) void *ocrToolbarControls;
@property(nonatomic) void *ocrToolbarConfirm;
@property(nonatomic, strong) NSMutableArray<ScreenwideOscMaterialSurfaceView *> *ocrToolbarSurfaces;
@property(nonatomic, strong) NSArray<ScreenwideOscTextTexture *> *ocrToolbarLabels;
@property(nonatomic) CGFloat ocrToolbarLabelScale;
@property(nonatomic) uint32_t ocrToolbarLabelLightMode;
@property(nonatomic) BOOL ocrToolbarVisible;
@property(nonatomic) BOOL ocrToolbarCloseArmed;
@property(nonatomic) uint64_t ocrToolbarCloseRevision;
@property(nonatomic) uint64_t ocrToolbarAnimationRevision;
@property(nonatomic, strong) id<MTLDevice> device;
@property(nonatomic, strong) id<MTLCommandQueue> queue;
@property(nonatomic, strong) id<MTLRenderPipelineState> pipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> magnifierPipeline;
@property(nonatomic, strong) id<MTLTexture> placeholder;
@property(nonatomic, strong) id<MTLBuffer> magnifierSource;
@property(nonatomic) uint32_t magnifierSourceWidth;
@property(nonatomic) uint32_t magnifierSourceHeight;
@property(nonatomic) ScreenwideRegionMagnifier magnifier;
@property(nonatomic) NSRect region;
@property(nonatomic) BOOL visible;
@property(nonatomic) BOOL showFrame;
@property(nonatomic) BOOL showHandles;
@property(nonatomic) void *rustContext;
@property(nonatomic) void (*releaseContext)(void *);
@property(nonatomic) NativeOscInput input;
@property(nonatomic) NativeOscLayout layoutChanged;
@property(nonatomic) id eventMonitor;
@property(nonatomic) BOOL inputEnabled;
@property(nonatomic) BOOL gestureActive;
@property(nonatomic) BOOL cursorRectsDisabled;
@property(nonatomic) BOOL cursorHidden;
@property(nonatomic, weak) ScreenwideRegionOSC *cursorOwner;
@property(nonatomic) uint64_t cursorClaimGeneration;
@property(nonatomic) BOOL drawInFlight;
@property(nonatomic) BOOL drawPending;
@property(nonatomic) uint64_t drawRevision;
@property(nonatomic) NSRect exclusionRect;
@property(nonatomic, strong) NSView *appearanceObserver;
@property(nonatomic, weak) ScreenwideRegionOSC *desktopRoot;
@property(nonatomic, strong) NSMutableArray<ScreenwideRegionOSC *> *desktopPeers;
@property(nonatomic, strong) NSMutableArray<NSWindow *> *desktopWindows;
@property(nonatomic) NSPoint desktopOffset;
@property(nonatomic) NSSize desktopSize;
@property(nonatomic) NSRect desktopRegion;
@property(nonatomic) uint32_t displayID;
@property(nonatomic, strong) id screenObserver;
@end

void screenwide_region_osc_draw(ScreenwideRegionOSC *s);
void screenwide_region_osc_add_icon(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    uint8_t icon, CGFloat left, CGFloat top,
                                    CGFloat icon_size);
void screenwide_region_osc_input_install(ScreenwideRegionOSC *s);
void screenwide_region_osc_input_teardown(ScreenwideRegionOSC *s);
void screenwide_region_osc_cursor_claim(ScreenwideRegionOSC *s);
void screenwide_region_osc_cursor_release(ScreenwideRegionOSC *s);
void screenwide_region_osc_cancel_pointer_claim(ScreenwideRegionOSC *s);
void screenwide_region_osc_claim_pointer_surface(void *view_ptr);
void screenwide_region_osc_appearance_install(ScreenwideRegionOSC *s);
void screenwide_region_osc_appearance_teardown(ScreenwideRegionOSC *s);
void *screenwide_region_osc_attach(void *view_ptr, void *context,
                                   void (*release)(void *),
                                   NativeOscInput input,
                                   NativeOscLayout layout_changed);
ScreenwideRegionOSC *screenwide_region_osc_for_view(void *view_ptr);
ScreenwideRegionOSC *screenwide_region_osc_root(ScreenwideRegionOSC *s);
NSArray<ScreenwideRegionOSC *> *
screenwide_region_osc_surfaces(ScreenwideRegionOSC *s);
void screenwide_region_osc_apply_region(ScreenwideRegionOSC *s, NSRect region,
                                        BOOL visible);
size_t screenwide_region_osc_configure_desktop(
    void *view_ptr, uint32_t anchor_id,
    ScreenwideRegionDesktopDisplay *displays, size_t capacity,
    double *desktop_width, double *desktop_height,
    uint32_t *resolved_anchor_id, int *layout_changed);
void screenwide_region_osc_set_desktop_presented(void *view_ptr,
                                                  int presented);
int screenwide_region_osc_set_snapshot(void *view_ptr, uint32_t display_id,
                                       const uint8_t *rgba, size_t length,
                                       uint32_t width, uint32_t height);
void screenwide_region_osc_set_snapshot_presented(void *view_ptr,
                                                   int presented);
void screenwide_region_osc_ocr_attach(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_teardown(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_update_appearance(ScreenwideRegionOSC *surface);
BOOL screenwide_region_osc_ocr_control_input(ScreenwideRegionOSC *surface,
                                             NSPoint point, uint32_t phase);
BOOL screenwide_region_osc_ocr_cancel_input(ScreenwideRegionOSC *surface,
                                            NSPoint point, uint32_t phase);
void screenwide_region_osc_ocr_cancel_attach(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_cancel_teardown(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_cancel_update_appearance(
    ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_attach(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_teardown(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_layout(ScreenwideRegionOSC *surface,
                                              BOOL visible);
void screenwide_region_osc_ocr_toolbar_render(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_apply_update(
    ScreenwideRegionOSC *surface, ScreenwideOscControlUpdate update);
void screenwide_region_osc_ocr_toolbar_apply_confirm_update(
    ScreenwideRegionOSC *surface, ScreenwideOscConfirmUpdate update);
uint8_t screenwide_region_osc_ocr_toolbar_icon(
    ScreenwideRegionOSC *surface, NSUInteger index);
void screenwide_region_osc_ocr_set_cancel_visible(void *view_ptr, int visible);
NSUInteger screenwide_region_osc_ocr_vertex_capacity(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_add_vertices(
    ScreenwideRegionOSC *surface, ScreenwideRegionOscVertex *vertices,
    NSUInteger *count, NSSize size, CGFloat scale);
int screenwide_region_osc_set_ocr(void *view_ptr, uint32_t phase,
                                  const ScreenwideRegionOcrRect *rects,
                                  size_t count, const char *message);
size_t screenwide_ocr_toolbar_layout(
    double selection_x, double selection_y, double selection_width,
    double selection_height, double viewport_width, double viewport_height,
    const double *widths, double height, ScreenwideOcrToolbarRect *output,
    size_t capacity);
