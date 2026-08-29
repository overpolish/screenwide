// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <QuartzCore/CAMetalLayer.h>
#import "region_osc_renderer_macos.h"

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
void screenwide_set_region_expected_cursor(NSCursor *cursor);

@interface ScreenwideRegionOSC : NSObject
@property(nonatomic, weak) NSView *host;
@property(nonatomic, strong) CAMetalLayer *layer;
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
