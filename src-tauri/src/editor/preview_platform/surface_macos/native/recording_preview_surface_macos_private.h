// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_H
#define SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_H

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>
#import <QuartzCore/CAMetalLayer.h>
#include <stdint.h>

#import "../../../../../crates/osc/src/gpu/macos/osc_gpu_macos.h"
#import "recording_preview_workspace_scene_macos.h"
#import "recording_preview_annotation_macos.h"
#import "recording_preview_audio_ribbon_macos.h"
#import "recording_preview_selection_macos.h"
#import "recording_preview_turn_macos.h"

#define SCREENWIDE_PREVIEW_PRIVATE __attribute__((visibility("hidden")))

extern SCREENWIDE_PREVIEW_PRIVATE NSCursor *expected_selection_cursor;
extern SCREENWIDE_PREVIEW_PRIVATE BOOL expected_selection_move_cursor;
void install_native_cursor_guard(void);

static const uint32_t ScreenwideFrameLayerId = UINT32_MAX;
// `KEYBOARD_LAYER_ID` and `CAPTIONS_LAYER_ID` in `preview_platform.rs`.
static const uint32_t ScreenwideKeyboardLayerId = UINT32_MAX - 1;
static const uint32_t ScreenwideCaptionsLayerId = UINT32_MAX - 2;
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
/// A right press (or Control-click) that landed on a video layer or an
/// annotation. `annotation_index` is the annotation's place in the published
/// list, or -1 for a layer. `x` and `y` are logical points from the top-left
/// of the window's content, the frame the webview reports pointer events in,
/// so the web layer can open its own menu exactly where the press was.
typedef void (*screenwide_preview_context_menu_callback)(uint32_t pane_index,
                                                          int32_t annotation_index,
                                                          double x, double y,
                                                          void *context);

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
/// A crop window drawn from a press outside the current one: its anchor in
/// pane fractions, and whether the press has travelled far enough to begin.
@property(nonatomic) NSPoint cropDrawAnchor;
@property(nonatomic) BOOL cropDrawBegun;
/// The arrow tool's live drag. `annotationDragActive` covers the whole press,
/// including one that only chooses an arrow; every edit - a new arrow, a grip,
/// or the whole arrow carried by its shaft - waits for the press to travel far
/// enough to be a drag before it begins at all.
@property(nonatomic) BOOL annotationDragActive;
@property(nonatomic) BOOL annotationDragPending;
@property(nonatomic) BOOL annotationDragBegun;
@property(nonatomic) NSPoint annotationDragOrigin;
@property(nonatomic) uint32_t annotationDragTargetKind;
@property(nonatomic) uint32_t annotationDragIndex;
@property(nonatomic) uint32_t annotationDragHandle;
@property(nonatomic) BOOL panning;
@property(nonatomic) BOOL pinching;
@property(nonatomic) BOOL suppressPinchScroll;
@property(nonatomic) NSPoint pinchAnchor;
/// A ring of the rotation gizmo held: the drag, where it was pressed, and the
/// resize edges whose cursor runs the way it turns there.
@property(nonatomic) BOOL turnDragActive;
@property(nonatomic) ScreenwideTurnRingDrag turnDrag;
@property(nonatomic) NSPoint turnDragOrigin;
@property(nonatomic) uint32_t turnDragEdges;
/// A ring of the chosen image's gizmo held, `+annotation_image_turn.m`.
@property(nonatomic) BOOL imageTurnActive;
@property(nonatomic) ScreenwideImageTurnHold imageTurnHold;
@end

@interface ScreenwidePreviewInteractionView (Cursor)
- (void)releaseCursorControl;
@end
@interface ScreenwidePreviewInteractionView (ContextMenu)
- (BOOL)reportContextMenuAtPoint:(NSPoint)point;
@end
@interface ScreenwidePreviewInteractionView (WorkspaceMove)
- (void)beginWorkspaceMove;
@end
@interface ScreenwidePreviewView : NSView
@property(nonatomic) BOOL active;
/// A resize the webview has laid out but whose matching frame has not been
/// composed yet. Applying it early would show the previous drawable fitted
/// into the new rect for a display tick; the next present applies it in the
/// same Core Animation transaction as the new pixels.
@property(nonatomic) BOOL hasPendingFrame;
@property(nonatomic) NSRect pendingFrame;
@end
@interface ScreenwidePreviewSurface : NSObject
/// The shared wgpu device's Metal device and queue: the OSC is encoded on
/// that queue after the scene's draw, so it lands over the drawn workspace.
@property(nonatomic, strong) id<MTLDevice> device;
@property(nonatomic, strong) id<MTLCommandQueue> queue;
/// The workspace's retained scene, a strong reference released when the
/// surface is deallocated.
@property(nonatomic) const void *scene;
@property(nonatomic, strong) NSView *host;
@property(nonatomic, weak) NSView *webview;
@property(nonatomic, strong) ScreenwidePreviewView *container;
@property(nonatomic, strong) NSMutableArray<ScreenwidePreviewView *> *views;
/// The audio-only preview's bars. They fill the viewport when the layout has
/// no video panes, and hide themselves whenever one appears.
@property(nonatomic, strong) ScreenwideAudioRibbon *audioRibbon;
@property(nonatomic) BOOL workspaceMode;
@property(nonatomic) uint32_t workspaceLayerCount;
@property(nonatomic) BOOL workspaceRedrawRetried;
@property(nonatomic) BOOL workspaceExplicitPlacements;
/// Whether the layout that is on screen laid out any pane at all. The
/// audio-only preview lays out none, and a decoded frame that arrives anyway
/// must not un-hide a pane view over the bars.
@property(nonatomic) BOOL workspaceHasPanes;
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
@property(nonatomic, strong) id<MTLTexture> workspaceEncodingTexture;
/// The workspace composed off screen with its chrome, copied into a drawable
/// taken off the main thread (`+workspace_present.m`).
@property(nonatomic, strong) id<MTLTexture> workspaceFrameTexture;
/// Whether the scene may have changed since it was last composed. A redraw
/// of the selection chrome alone leaves it, and shows the last composed
/// picture under the new chrome.
@property(nonatomic) BOOL workspaceSceneDirty;
/// The drawable size last presented. A redraw at another size presents in
/// the main thread's own transaction, beside the layout that resized it.
@property(nonatomic) CGSize workspacePresentedSize;
@property(nonatomic) ScreenwideRegionMagnifier workspaceMagnifier;
@property(nonatomic, strong) ScreenwidePreviewInteractionView *interaction;
@property(nonatomic) BOOL editorEnabled;
/// Set while React covers the workarea with its own chrome (a save sheet,
/// the export window). Input and native chrome go away; `editorEnabled`, the
/// transform callback and the pan/zoom stay exactly as they were, so the
/// workspace comes back where the user left it.
@property(nonatomic) BOOL editorSuspended;
@property(nonatomic, strong) NSMutableArray<NSValue *> *editorBaseRects;
@property(nonatomic) double editorPanX;
@property(nonatomic) double editorPanY;
@property(nonatomic) double editorZoom;
/// The width the double-click reset fits into: the space beside an open tool
/// panel while one holds the basis, 0 for the whole viewport. Only the fit
/// commands move it; a resize or a gesture never does.
@property(nonatomic) double editorPanelFitWidth;
@property(nonatomic) screenwide_preview_transform_callback transformCallback;
@property(nonatomic) void *transformContext;
@property(nonatomic) screenwide_preview_selection_gesture_callback selectionGestureCallback;
@property(nonatomic) void *selectionGestureContext;
@property(nonatomic) screenwide_preview_selection_callback selectionCallback;
@property(nonatomic) void *selectionContext;
@property(nonatomic) screenwide_preview_pointer_down_callback pointerDownCallback;
@property(nonatomic) void *pointerDownContext;
@property(nonatomic) screenwide_preview_context_menu_callback contextMenuCallback;
@property(nonatomic) void *contextMenuContext;
@property(nonatomic) BOOL selectionHitTestingEnabled;
@property(nonatomic, strong) NSArray<NSValue *> *selectionTargets;
@property(nonatomic) BOOL selectionSnappingEnabled;
@property(nonatomic) BOOL hasSelectionSnapGuideX;
@property(nonatomic) BOOL hasSelectionSnapGuideY;
@property(nonatomic) BOOL selectionSnapGuideXIsObject;
@property(nonatomic) BOOL selectionSnapGuideYIsObject;
@property(nonatomic) double selectionSnapGuideX;
@property(nonatomic) double selectionSnapGuideY;
/// The arrow tool's scene: every grip on the selected layer and the strokes'
/// lines they point into, which one is chosen, and whether the tool has it.
@property(nonatomic, strong) NSMutableData *annotations;
@property(nonatomic, strong) NSData *annotationPaths;
@property(nonatomic) NSInteger annotationSelected;
@property(nonatomic) ScreenwideAnnotationMode annotationMode;
/// What the last gesture sample snapped to, for the chrome to draw. Zeroed
/// whenever a sample snaps to nothing, and when the gesture ends.
@property(nonatomic) ScreenwideAnnotationSnap annotationSnap;
@property(nonatomic) screenwide_preview_annotation_gesture_callback annotationGestureCallback;
@property(nonatomic) void *annotationGestureContext;
/// The arrow the pointer is resting on, its pulse, and the revision that
/// retires a pulse the pointer has already moved away from.
@property(nonatomic) NSInteger annotationHovered;
@property(nonatomic) CFTimeInterval annotationHoverStarted;
@property(nonatomic) uint64_t annotationHoverRevision;
@property(nonatomic) screenwide_preview_annotation_hover_callback annotationHoverCallback;
@property(nonatomic) void *annotationHoverContext;
@property(nonatomic) BOOL hasSelection;
@property(nonatomic) BOOL selectionVisible;
@property(nonatomic) ScreenwidePreviewSelection selection;
/// The rotation gizmo ring under the pointer: its axis plus one, zero for
/// none.
@property(nonatomic) NSInteger turnHoveredRing;
@property(nonatomic, strong) CAMetalLayer *selectionLayer;
@property(nonatomic) uint64_t selectionDrawRevision;
@property(nonatomic) BOOL selectionDrawInFlight;
@property(nonatomic) BOOL selectionDrawPending;
/// Drawables and pane frames published by one Core Animation transaction.
@property(nonatomic, strong) NSLock *batchLock;
@property(nonatomic, strong) id windowScreenObserver;
@property(nonatomic, strong) id windowBackingObserver;
@property(nonatomic) NSInteger batchDepth;
@property(nonatomic, strong) NSMutableArray<id<CAMetalDrawable>> *batchDrawables;
@property(nonatomic, strong) NSMutableArray<ScreenwidePreviewView *> *batchViews;
/// Tracks the batch's in-flight command buffers: entered per batched present,
/// left from that command buffer's completed handler. `end_present` defers its
/// presenting transaction until the group is empty so the commit never fences
/// on GPU work (see `screenwide_preview_surface_end_present`).
@property(nonatomic, strong) dispatch_group_t batchGroup;
@end
#import "recording_preview_annotation_text_macos.h"
#import "recording_preview_annotation_group_macos.h"
#import "recording_preview_image_drop_macos.h"
#import "recording_preview_surface_macos_private_functions.h"
#endif
