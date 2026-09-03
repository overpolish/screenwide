// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <QuartzCore/CAMetalLayer.h>
#include <stddef.h>
#import "osc_controls.h"
#import "osc_gpu_macos.h"
#import "osc_material_surface_macos.h"
#import "osc_text_texture_macos.h"

typedef struct {
  uint8_t status, gesture, handle, cursor, has_region;
  double x, y, width, height;
  uint32_t ruler_color;
  uint8_t ruler_flags;
  uint8_t ruler_padding[3];
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
typedef struct {
  uint64_t id;
  double x, y, width, height;
  uint8_t flags;
  uint8_t padding[7];
  double label_anchor_x, label_anchor_y;
} NativeRulerMeasurement;
typedef struct {
  uint32_t display_id;
  uint32_t padding;
  double zoom;
  double origin_x, origin_y;
} NativeRulerViewport;
typedef struct {
  uint64_t id;
  uint32_t display_id;
  uint8_t axis;
  uint8_t flags;
  uint8_t padding[2];
  double start, end, position;
  double label_anchor_x, label_anchor_y;
} NativeRulerProbe;
typedef struct {
  uint64_t id;
  uint32_t display_id;
  uint8_t axis;
  uint8_t flags;
  uint8_t padding[2];
  double position;
} NativeRulerGuide;
typedef struct {
  uint64_t id;
  uint64_t owner_id;
  uint32_t display_id;
  uint8_t axis;
  uint8_t flags;
  uint8_t padding[2];
  double start, end, position;
  double label_anchor_x, label_anchor_y;
} NativeRulerGuideGap;
typedef struct {
  uint64_t id;
  uint32_t display_id;
  uint8_t corner;
  uint8_t flags;
  uint8_t padding[2];
  double x, y, width, height, radius;
  double label_anchor_x, label_anchor_y;
} NativeRulerRadius;
typedef struct {
  uint64_t id;
  double x, y, width, height;
  uint8_t flags;
  uint8_t padding[7];
} NativeRulerCenterline;
typedef struct {
  uint64_t owner_id;
  double x, y, width, height;
  uint8_t flags;
  uint8_t padding[7];
} NativeRulerInnerObject;
typedef struct {
  uint64_t id;
  uint8_t kind;
  uint8_t padding[7];
  NSPoint center;
} ScreenwideRulerLabelHit;
_Static_assert(sizeof(ScreenwideRegionOcrRect) == 40,
               "ScreenwideRegionOcrRect ABI must match Rust");
_Static_assert(offsetof(ScreenwideRegionOcrRect, kind) == 32,
               "ScreenwideRegionOcrRect.kind ABI must match Rust");
_Static_assert(sizeof(NativeRulerMeasurement) == 64,
               "NativeRulerMeasurement ABI must match Rust");
_Static_assert(offsetof(NativeRulerMeasurement, flags) == 40,
               "NativeRulerMeasurement.flags ABI must match Rust");
_Static_assert(offsetof(NativeRulerMeasurement, label_anchor_x) == 48,
               "NativeRulerMeasurement.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerViewport) == 32,
               "NativeRulerViewport ABI must match Rust");
_Static_assert(offsetof(NativeRulerViewport, zoom) == 8,
               "NativeRulerViewport.zoom ABI must match Rust");
_Static_assert(sizeof(NativeRulerProbe) == 56,
               "NativeRulerProbe ABI must match Rust");
_Static_assert(offsetof(NativeRulerProbe, start) == 16,
               "NativeRulerProbe.start ABI must match Rust");
_Static_assert(offsetof(NativeRulerProbe, label_anchor_x) == 40,
               "NativeRulerProbe.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerGuide) == 24,
               "NativeRulerGuide ABI must match Rust");
_Static_assert(offsetof(NativeRulerGuide, position) == 16,
               "NativeRulerGuide.position ABI must match Rust");
_Static_assert(sizeof(NativeRulerGuideGap) == 64,
               "NativeRulerGuideGap ABI must match Rust");
_Static_assert(offsetof(NativeRulerGuideGap, start) == 24,
               "NativeRulerGuideGap.start ABI must match Rust");
_Static_assert(offsetof(NativeRulerGuideGap, label_anchor_x) == 48,
               "NativeRulerGuideGap.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerRadius) == 72,
               "NativeRulerRadius ABI must match Rust");
_Static_assert(offsetof(NativeRulerRadius, x) == 16,
               "NativeRulerRadius.x ABI must match Rust");
_Static_assert(offsetof(NativeRulerRadius, label_anchor_x) == 56,
               "NativeRulerRadius.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerCenterline) == 48,
               "NativeRulerCenterline ABI must match Rust");
_Static_assert(offsetof(NativeRulerCenterline, flags) == 40,
               "NativeRulerCenterline.flags ABI must match Rust");
_Static_assert(sizeof(NativeRulerInnerObject) == 48,
               "NativeRulerInnerObject ABI must match Rust");
_Static_assert(offsetof(NativeRulerInnerObject, flags) == 40,
               "NativeRulerInnerObject.flags ABI must match Rust");
size_t native_osc_ruler_measurements(void *context,
                                     NativeRulerMeasurement *output,
                                     size_t capacity);
size_t native_osc_ruler_viewports(void *context,
                                  NativeRulerViewport *output,
                                  size_t capacity);
size_t native_osc_ruler_probes(void *context, NativeRulerProbe *output,
                              size_t capacity);
size_t native_osc_ruler_guides(void *context, NativeRulerGuide *output,
                              size_t capacity);
size_t native_osc_ruler_guide_gaps(void *context,
                                  NativeRulerGuideGap *output,
                                  size_t capacity);
size_t native_osc_ruler_radii(void *context, NativeRulerRadius *output,
                             size_t capacity);
size_t native_osc_ruler_centerlines(void *context,
                                    NativeRulerCenterline *output,
                                    size_t capacity);
size_t native_osc_ruler_inner_objects(void *context,
                                      NativeRulerInnerObject *output,
                                      size_t capacity);
int native_osc_ruler_viewport_input(void *context, uint32_t display_id,
                                    uint32_t operation, double anchor_x,
                                    double anchor_y, double delta_x,
                                    double delta_y, NativeOscResult *output);
void native_osc_ruler_label_input(
    void *context, uint32_t operation, uint8_t kind, uint64_t id,
    double pointer_x, double pointer_y, double label_center_x,
    double label_center_y, NativeOscResult *output);
void screenwide_set_region_expected_cursor(NSCursor *cursor);

@interface ScreenwideRegionOSC : NSObject
@property(nonatomic, weak) NSView *host;
@property(nonatomic, strong) CAMetalLayer *layer;
@property(nonatomic, strong) CALayer *snapshotLayer;
@property(nonatomic, strong) id<MTLTexture> snapshotTexture;
@property(nonatomic) BOOL snapshotPresented;
@property(nonatomic) BOOL snapshotComposited;
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
@property(nonatomic, strong) id<MTLRenderPipelineState> snapshotPipeline;
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
@property(nonatomic) BOOL rulerVisible;
@property(nonatomic) BOOL rulerTransientChromeVisible;
@property(nonatomic) BOOL rulerInteractionActive;
@property(nonatomic) BOOL rulerCrosshair;
@property(nonatomic) BOOL rulerCopied;
@property(nonatomic) NSPoint rulerPoint;
@property(nonatomic) uint32_t rulerColor;
@property(nonatomic, strong) ScreenwideOscMaterialSurfaceView *rulerSurface;
@property(nonatomic) void *rulerControls;
@property(nonatomic, strong) ScreenwideOscTextTexture *rulerLabel;
@property(nonatomic) CGFloat rulerLabelScale;
@property(nonatomic) uint32_t rulerLabelLightMode;
@property(nonatomic, strong) ScreenwideOscTextTexture *rulerToleranceLabel;
@property(nonatomic) CGFloat rulerToleranceLabelScale;
@property(nonatomic) uint32_t rulerToleranceLabelLightMode;
@property(nonatomic) uint8_t rulerToleranceMode;
@property(nonatomic) BOOL rulerToleranceVisible;
@property(nonatomic) uint64_t rulerToleranceRevision;
@property(nonatomic) uint64_t rulerToleranceAnimationRevision;
@property(nonatomic) CGFloat rulerToleranceAnimationFrom;
@property(nonatomic) CFTimeInterval rulerToleranceAnimationStarted;
@property(nonatomic) BOOL rulerToleranceAnimationTarget;
@property(nonatomic) uint64_t rulerCopiedRevision;
@property(nonatomic) uint64_t rulerAnimationRevision;
@property(nonatomic) CGFloat rulerAnimationFrom;
@property(nonatomic) CFTimeInterval rulerAnimationStarted;
@property(nonatomic) BOOL rulerAnimationTarget;
@property(nonatomic) BOOL rulerDrawInFlight;
@property(nonatomic) BOOL rulerDrawPending;
@property(nonatomic, strong) NSData *rulerMeasurements;
@property(nonatomic, strong) NSMutableArray<ScreenwideOscMaterialSurfaceView *> *rulerMeasurementLabelSurfaces;
@property(nonatomic, strong) NSData *rulerProbes;
@property(nonatomic, strong) NSData *rulerGuides;
@property(nonatomic, strong) NSMutableArray<ScreenwideOscMaterialSurfaceView *> *rulerProbeLabelSurfaces;
@property(nonatomic, strong) NSData *rulerGuideGaps;
@property(nonatomic, strong) NSMutableArray<ScreenwideOscMaterialSurfaceView *> *rulerGuideGapLabelSurfaces;
@property(nonatomic, strong) NSData *rulerRadii;
@property(nonatomic, strong) NSMutableArray<ScreenwideOscMaterialSurfaceView *> *rulerRadiusLabelSurfaces;
@property(nonatomic, strong) NSData *rulerCenterlines;
@property(nonatomic, strong) NSData *rulerInnerObjects;
@property(nonatomic) BOOL rulerSettleScheduled;
@property(nonatomic) uint64_t rulerHoveredArtifactKey;
@property(nonatomic) CGFloat rulerHoverOpacity;
@property(nonatomic) uint64_t rulerHoverPulseRevision;
@property(nonatomic) CFTimeInterval rulerHoverPulseStarted;
@property(nonatomic) CGFloat rulerViewportZoom;
@property(nonatomic) NSPoint rulerViewportOrigin;
@property(nonatomic) BOOL rulerPanActive;
@property(nonatomic) NSPoint rulerPanLastPoint;
@property(nonatomic) uint16_t rulerRangeKeyCode;
@property(nonatomic) uint16_t rulerGuideKeyCode;
@property(nonatomic) uint16_t rulerRadiusKeyCode;
@property(nonatomic) BOOL rulerLabelDragActive;
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
void screenwide_region_osc_ruler_refresh_pointer(void *view_ptr);
void screenwide_region_osc_appearance_install(ScreenwideRegionOSC *s);
void screenwide_region_osc_appearance_teardown(ScreenwideRegionOSC *s);
void screenwide_region_osc_apply_ruler_result(ScreenwideRegionOSC *s,
                                               NativeOscResult result);
void screenwide_region_osc_ruler_attach(ScreenwideRegionOSC *s);
void screenwide_region_osc_ruler_teardown(ScreenwideRegionOSC *s);
void screenwide_region_osc_ruler_update_appearance(ScreenwideRegionOSC *s);
void screenwide_region_osc_ruler_set_transient_chrome(void *view_ptr,
                                                       int visible);
void screenwide_region_osc_ruler_apply_render_state(
    ScreenwideRegionOSC *s, ScreenwideRegionOscRenderState *state);
BOOL screenwide_region_osc_ruler_label_hit(
    ScreenwideRegionOSC *surface, NSPoint point,
    ScreenwideRulerLabelHit *hit);
NSUInteger screenwide_region_osc_ruler_vertex_capacity(
    ScreenwideRegionOSC *surface);
void screenwide_region_osc_ruler_add_vertices(
    ScreenwideRegionOSC *surface, ScreenwideRegionOscVertex *vertices,
    NSUInteger *count, NSSize size, CGFloat scale);
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
void screenwide_region_osc_set_snapshot_composited(void *view_ptr,
                                                    int composited);
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
