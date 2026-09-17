// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <QuartzCore/CAMetalLayer.h>
#import <QuartzCore/CAShapeLayer.h>
#include <stddef.h>
#import "osc_controls.h"
#import "osc_gpu_macos.h"
#import "osc_material_surface_macos.h"
#import "osc_text_texture_macos.h"
#import "screenshot_region_osc_macos_types.h"

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
@property(nonatomic, strong) CAShapeLayer *ocrStatusSpinner;
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

#import "screenshot_region_osc_macos_private_functions.h"
