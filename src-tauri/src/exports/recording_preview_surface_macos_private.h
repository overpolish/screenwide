// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_H
#define SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_H

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>
#import <QuartzCore/CAMetalLayer.h>
#include <stdint.h>

#import "cursor_export/gpu_compositor_macos.h"

#define SCREENWIDE_PREVIEW_PRIVATE __attribute__((visibility("hidden")))

typedef struct {
  double x;
  double y;
  double width;
  double height;
} ScreenwideDisplayRect;

typedef struct {
  uint64_t id;
  ScreenwideDisplayRect rect;
  uint8_t radius_enabled;
  double radius_percent;
  int32_t z_order;
  uint8_t selected;
  uint8_t visible;
} ScreenwideDisplayTarget;

typedef struct {
  uint8_t found;
  uint64_t target_id;
  uint8_t handle;
} ScreenwideDisplayHit;

typedef struct {
  ScreenwideDisplayRect fit;
  double zoom;
  double pan_x;
  double pan_y;
} ScreenwideDisplayFitRebase;

extern ScreenwideDisplayHit screenwide_workspace_hit_test(
    const ScreenwideDisplayTarget *targets, size_t count, double x, double y,
    double handle_size);
extern ScreenwideDisplayFitRebase screenwide_workspace_rebase_display_fit(
    double viewport_width, double viewport_height, ScreenwideDisplayRect displayed, double natural_width,
    double natural_height, double gutter, uint8_t allow_upscale);
_Static_assert(sizeof(ScreenwideDisplayTarget) == 64,
               "Rust/C display target layout mismatch");
_Static_assert(sizeof(ScreenwideDisplayHit) == 24,
               "Rust/C display hit layout mismatch");
_Static_assert(sizeof(ScreenwideDisplayFitRebase) == 56,
               "Rust/C display fit rebase layout mismatch");

extern SCREENWIDE_PREVIEW_PRIVATE NSCursor *expected_selection_cursor;
extern SCREENWIDE_PREVIEW_PRIVATE BOOL expected_selection_move_cursor;
void install_native_cursor_guard(void);

static const uint32_t ScreenwideFrameLayerId = UINT32_MAX;
static const uint32_t ScreenwideCenteredResizeEdge = 1u << 16;
static const uint32_t ScreenwideAutoFitMoveEdge = 1u << 17;
static const uint32_t ScreenwideAutoFitCommitEdge = 1u << 18;
typedef void (*screenwide_preview_transform_callback)(double zoom_percent,
                                                       void *context);
typedef void (*screenwide_preview_selection_gesture_callback)(uint32_t phase,
                                                               uint32_t pane_index,
                                                               uint32_t operation,
                                                               uint32_t edges,
                                                               double scale,
                                                               double delta_x,
                                                               double delta_y,
                                                               void *context);
typedef void (*screenwide_preview_selection_callback)(int32_t pane_index,
                                                       void *context);
typedef void (*screenwide_preview_pointer_down_callback)(void *context);

typedef struct {
  uint32_t pane_index;
  uint32_t layer_id;
  uint32_t crop_mode, radius_disabled, recenter_mode;
  double x, y, width, height;
  double radius_percent;
  double image_x, image_y, image_width, image_height;
  double recenter_x, recenter_y, recenter_width, recenter_height;
} ScreenwidePreviewSelection;
@class ScreenwidePreviewSurface;

@interface ScreenwidePreviewInteractionView : NSView
@property(nonatomic, weak) ScreenwidePreviewSurface *surface;
@property(nonatomic) NSPoint dragOrigin;
@property(nonatomic) NSPoint dragPan;
@property(nonatomic) NSPoint selectionDragOrigin;
@property(nonatomic) NSRect selectionFrameDragStart;
@property(nonatomic, strong) NSArray<NSValue *> *selectionFramePaneStarts;
@property(nonatomic) double selectionFrameZoomStart;
@property(nonatomic) NSPoint selectionFramePanStart;
@property(nonatomic) NSRect selectionMoveFrameStart;
@property(nonatomic) NSPoint selectionMovePanStart;
@property(nonatomic) double selectionMoveZoomStart;
@property(nonatomic) double selectionMoveDeltaX;
@property(nonatomic) double selectionMoveDeltaY;
@property(nonatomic) BOOL selectionMoveAutoFitActive;
/// The bounds (in mouse-down canvas units) the last auto-fit sample grew the
/// canvas to, so an Option release can re-express the move's starts in the
/// committed canvas and let Option grow it again from there.
@property(nonatomic) NSRect selectionMoveAutoFitBounds;
@property(nonatomic, strong) NSArray<NSValue *> *selectionMoveTargetsStart;
@property(nonatomic) ScreenwidePreviewSelection selectionDragStart;
@property(nonatomic) BOOL selectionDragActive;
@property(nonatomic) BOOL selectionDragCentered;
@property(nonatomic) uint32_t selectionDragOperation;
@property(nonatomic) uint32_t selectionDragEdges;
@property(nonatomic, strong) NSTrackingArea *selectionTrackingArea;
@property(nonatomic) BOOL cursorRectsDisabled;
@property(nonatomic) BOOL panning;
@end

@interface ScreenwidePreviewInteractionView (Editor)
- (void)releaseCursorControl;
@end
@interface ScreenwidePreviewView : NSView
@property(nonatomic) BOOL active;
@property(nonatomic) void *compositor;
/// A resize the webview has laid out but whose matching frame has not been
/// composed yet. Applying it early would show the previous drawable fitted
/// into the new rect for a display tick; the next present applies it in the
/// same Core Animation transaction as the new pixels.
@property(nonatomic) BOOL hasPendingFrame;
@property(nonatomic) NSRect pendingFrame;
@end
@interface ScreenwidePreviewSurface : NSObject
@property(nonatomic, strong) id<MTLDevice> device;
@property(nonatomic, strong) id<MTLCommandQueue> queue;
@property(nonatomic, strong) id<MTLComputePipelineState> pipeline;
@property(nonatomic, strong) id<MTLRenderPipelineState> selectionPipeline;
@property(nonatomic, strong) NSView *host;
@property(nonatomic, weak) NSView *webview;
@property(nonatomic, strong) ScreenwidePreviewView *container;
@property(nonatomic, strong) NSMutableArray<ScreenwidePreviewView *> *views;
@property(nonatomic) BOOL workspaceMode;
@property(nonatomic) uint32_t workspaceLayerCount;
@property(nonatomic) BOOL workspaceRedrawRetried;
@property(nonatomic) BOOL workspaceExplicitPlacements;
@property(nonatomic, strong) NSMutableData *workspacePlacements;
@property(nonatomic, strong) NSArray<NSNumber *> *workspacePaneIndices;
@property(nonatomic, strong) NSSet<NSNumber *> *workspaceActivePaneIndices;
@property(nonatomic) double workspaceNaturalWidth;
@property(nonatomic) double workspaceNaturalHeight;
@property(nonatomic) double workspaceResizeNaturalWidth;
@property(nonatomic) double workspaceResizeNaturalHeight;
@property(nonatomic, strong) NSMutableDictionary<NSString *, NSValue *> *workspaceTransforms;
/// Set when a native gesture has just committed a new canvas size whose
/// zoom/pan are already right on screen. The layout that echoes that size
/// back must not treat it as an undo/redo jump and recentre the workspace.
@property(nonatomic) BOOL keepTransformForCommittedNaturalSize;
@property(nonatomic) BOOL workspaceDrawInFlight;
@property(nonatomic) BOOL workspaceDrawPending;
@property(nonatomic) BOOL workspaceLayoutAwaitsPresent;
@property(nonatomic, strong) NSLock *workspaceLock;
@property(nonatomic, strong) id<MTLCommandBuffer> workspaceEncodingCommand;
@property(nonatomic, strong) id<MTLTexture> workspaceEncodingTexture;
@property(nonatomic) ScreenwideWorkspaceMagnifier workspaceMagnifier;
@property(nonatomic, strong) ScreenwidePreviewInteractionView *interaction;
@property(nonatomic) BOOL editorEnabled;
@property(nonatomic, strong) NSMutableArray<NSValue *> *editorBaseRects;
@property(nonatomic) double editorPanX;
@property(nonatomic) double editorPanY;
@property(nonatomic) double editorZoom;
@property(nonatomic) screenwide_preview_transform_callback transformCallback;
@property(nonatomic) void *transformContext;
@property(nonatomic) screenwide_preview_selection_gesture_callback selectionGestureCallback;
@property(nonatomic) void *selectionGestureContext;
@property(nonatomic) screenwide_preview_selection_callback selectionCallback;
@property(nonatomic) void *selectionContext;
@property(nonatomic) screenwide_preview_pointer_down_callback pointerDownCallback;
@property(nonatomic) void *pointerDownContext;
@property(nonatomic) BOOL selectionHitTestingEnabled;
@property(nonatomic, strong) NSArray<NSValue *> *selectionTargets;
@property(nonatomic) BOOL selectionSnappingEnabled;
@property(nonatomic) BOOL hasSelectionSnapGuideX;
@property(nonatomic) BOOL hasSelectionSnapGuideY;
@property(nonatomic) BOOL selectionSnapGuideXIsObject;
@property(nonatomic) BOOL selectionSnapGuideYIsObject;
@property(nonatomic) double selectionSnapGuideX;
@property(nonatomic) double selectionSnapGuideY;
@property(nonatomic) BOOL hasSelection;
@property(nonatomic) BOOL selectionVisible;
@property(nonatomic) ScreenwidePreviewSelection selection;
@property(nonatomic, strong) CAMetalLayer *selectionLayer;
/// Cached selection label texture and the inputs that invalidate it.
@property(nonatomic, strong) id<MTLTexture> selectionLabelTexture;
/// Transparent texture bound whenever the selection has no label.
@property(nonatomic, strong) id<MTLTexture> selectionLabelPlaceholder;
@property(nonatomic, strong) NSString *selectionLabelText;
@property(nonatomic) CGFloat selectionLabelScale;
@property(nonatomic) uint32_t selectionLabelLightMode;
@property(nonatomic) NSSize selectionLabelSize;
@property(nonatomic) NSRect selectionActionRect;
@property(nonatomic) uint32_t selectionActionOperation;
@property(nonatomic) BOOL selectionActionHovered;
@property(nonatomic) BOOL selectionActionPressed;
@property(nonatomic) double selectionActionTransitionStarted;
@property(nonatomic) float selectionActionFromLight;
@property(nonatomic) float selectionActionFromDark;
@property(nonatomic) float selectionActionToLight;
@property(nonatomic) float selectionActionToDark;
@property(nonatomic) uint64_t selectionActionAnimationRevision;
@property(nonatomic) uint64_t selectionDrawRevision;
@property(nonatomic) BOOL selectionDrawInFlight;
@property(nonatomic) BOOL selectionDrawPending;
/// Drawables and pane frames published by one Core Animation transaction.
@property(nonatomic, strong) NSLock *batchLock;
@property(nonatomic) NSInteger batchDepth;
@property(nonatomic, strong) NSMutableArray<id<CAMetalDrawable>> *batchDrawables;
@property(nonatomic, strong) NSMutableArray<ScreenwidePreviewView *> *batchViews;
/// Tracks the batch's in-flight command buffers: entered per batched present,
/// left from that command buffer's completed handler. `end_present` defers its
/// presenting transaction until the group is empty so the commit never fences
/// on GPU work (see `screenwide_preview_surface_end_present`).
@property(nonatomic, strong) dispatch_group_t batchGroup;
@end
#import "recording_preview_surface_macos_private_functions.h"
#endif
