// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_TURN_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_TURN_MACOS_H

#import <AppKit/AppKit.h>
#include <stdint.h>

#import "../../../../../crates/osc/src/gpu/macos/osc_gpu_macos.h"
#import "recording_preview_selection_macos.h"

// The rotation gizmo and the turned layer's chrome, `+turn.m` and
// `+turn_chrome.m`. Where a turn tips the selected layer, its chrome is
// placed through the turn and drawn in soft lines, and presses on it are
// carried back.

/// One drag of a rotation gizmo ring, the twin of Rust's `RingDrag`
/// (`osc/src/turn.rs`): the axis, the turn at the press and the one the last
/// sample left, in radians, the way the pressed point travels, Y up, and the
/// ring's radius in display points.
typedef struct {
  uint32_t axis;
  double press[3];
  double last[3];
  double tangent[2];
  double radius;
} ScreenwideTurnRingDrag;
_Static_assert(sizeof(ScreenwideTurnRingDrag) == 80,
               "Rust/C ring drag layout mismatch");

/// The ring a press takes hold of: its axis, or -1 for none, the resize edges
/// whose cursor runs the way it turns there, and the angle along it.
typedef struct {
  int32_t axis;
  uint32_t edges;
  double angle;
} ScreenwideTurnRingHit;

// The gizmo's geometry and the turn, in Rust (`osc/src/turn/ffi.rs`).
extern double screenwide_turn_ring_radius(double short_side);
extern ScreenwideTurnRingHit screenwide_turn_ring_at(double x, double y, double z,
                                                     double radius, double px, double py);
extern void screenwide_turn_ring_basis(double x, double y, double z, double *out);
extern ScreenwideTurnRingDrag screenwide_turn_ring_begin(double x, double y, double z,
                                                         double radius, uint32_t axis,
                                                         double angle);
extern void screenwide_turn_ring_drag(ScreenwideTurnRingDrag *drag, double dx, double dy,
                                      uint8_t snap, double *out);
extern uint8_t screenwide_turn_homographies(double x, double y, double z, double cx,
                                            double cy, double extent, double *forward,
                                            double *inverse);
extern void screenwide_turn_compose(const double *outer, const double *inner, double *out);
extern void screenwide_turn_within(const double *outer, const double *seen, const double *near,
                                   double *out);

/// The chosen image's gizmo, `+annotation_image_turn.m`: the rings' middle
/// and radius in display points, the turn they show, the image's own turn,
/// each in degrees, and its place in the published annotations.
typedef struct {
  NSPoint centre;
  double radius;
  double seen[3];
  double own[3];
  NSInteger index;
} ScreenwideImageGizmo;

/// A ring of the chosen image's gizmo held: the drag, the image's place in
/// the published annotations, its own turn and the turn the rings show as the
/// last sample left them, where the press landed, and the resize edges whose
/// cursor runs the way the ring turns there.
typedef struct {
  ScreenwideTurnRingDrag drag;
  NSInteger index;
  double own[3];
  double seen[3];
  NSPoint origin;
  uint32_t edges;
} ScreenwideImageTurnHold;

@class ScreenwidePreviewSurface;
@class ScreenwidePreviewInteractionView;

/// `point` as it lands on `selection`'s upright layer: the press itself
/// where the layer stands upright.
NSPoint selection_unturned_point(ScreenwidePreviewSurface *surface,
                                 ScreenwidePreviewSelection selection, NSPoint point);
/// `point` as it lands on the selected layer standing upright.
NSPoint selected_layer_point(ScreenwidePreviewSurface *surface, NSPoint point);
/// How far a selection drag has travelled from its press to `point`, as the
/// drag measures it. A turned layer's grips are dragged across the layer as
/// its turn stood at the press, so `point` is carried back for them; a move
/// carries the whole turn with it, so its travel on screen is already the
/// layer's.
NSPoint selection_drag_travel(ScreenwidePreviewInteractionView *view, NSPoint *point);
/// Whether `selection` is turned, which is when its chrome is carried.
BOOL selection_is_turned(ScreenwidePreviewSelection selection);
/// The turn `selection`'s chrome goes through, in display points, and its
/// inverse. NO where the layer stands upright, or is seen edge on, which
/// leaves nothing of it to press.
BOOL selection_turn(ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection,
                    double forward[9], double inverse[9]);
/// Where `h` carries `point`, or far off every layer where it lands behind
/// the eye.
NSPoint turn_carry(const double h[9], NSPoint point);
/// The turn the annotation chrome is placed through, written to `storage`:
/// the selected layer's, or NULL where it stands upright.
const double *annotation_chrome_turn(ScreenwidePreviewSurface *surface, double storage[9]);
/// An annotation's grip disc over `point`, on the selected layer standing
/// upright, placed through `turn` where it is not NULL.
void annotation_osc_add_grip(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                             NSSize size, const double *turn, NSPoint point, CGFloat scale);
/// A snap guide, gap bar or outline edge laid out upright as the thin `rect`
/// in guide `kind` 4 or 5: the quad itself upright, or its middle line placed
/// through `turn` and drawn soft.
void annotation_osc_add_hairline(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                                 NSSize size, const double *turn, NSRect rect, uint32_t kind,
                                 CGFloat scale);
/// The box round a group, or round one of its members.
void annotation_osc_add_group_frame(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                                    NSSize size, const double *turn, NSRect rect,
                                    CGFloat scale);
/// The marquee band's marching ants.
void annotation_osc_add_marquee(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                                NSSize size, const double *turn, NSRect band, CGFloat scale);
/// A box annotation's box, grips and radius dot where it `has_radius`.
void annotation_osc_add_box(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                            NSSize size, const double *turn, NSRect frame,
                            double radius_percent, BOOL has_radius, CGFloat scale);
/// An image's own frame: its corners clockwise from the picture's own
/// top-left, and its radius dot.
void annotation_osc_add_image_frame(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                                    NSSize size, const double *turn,
                                    const NSPoint corners[4], NSPoint dot, CGFloat scale);
/// The selection's frame, grips and radius dot carried through its turn.
void turn_add_selection(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                        NSSize size, ScreenwidePreviewSurface *surface, CGFloat scale);
/// Whether the gizmo's rings are shown: the select tool's chrome over a
/// layer a scene turns.
BOOL turn_rings_shown(ScreenwidePreviewSurface *surface);
/// The rings' quad, and the rings themselves in `state`.
void turn_add_rings(ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
                    ScreenwideRegionOscRenderState *state, ScreenwidePreviewSurface *surface,
                    CGFloat scale);
/// The cursor a ring under `point` asks for, or nil.
NSCursor *turn_cursor(ScreenwidePreviewSurface *surface, NSPoint point);
/// Re-reads which ring the pointer rests on, redrawing when it changed.
void turn_update_hover(ScreenwidePreviewSurface *surface, NSPoint point);
/// A press, drag and release of a ring. Each answers YES when the event was
/// the gizmo's.
BOOL turn_mouse_down(ScreenwidePreviewInteractionView *view, NSEvent *event,
                     NSPoint point);
BOOL turn_mouse_dragged(ScreenwidePreviewInteractionView *view, NSEvent *event);
BOOL turn_mouse_up(ScreenwidePreviewInteractionView *view, NSEvent *event);
/// Whether a drag keeps the selection a layout hands over: a move, a resize
/// or a ring of the gizmo, for as long as it lasts. A layout that takes the
/// selection away lets go of a ring without keeping the turn it reached.
BOOL selection_held_by_drag(ScreenwidePreviewSurface *surface, BOOL has_selection);
/// The gizmo over the chosen image, where its rings are shown: one image
/// chosen alone, with the select or image tool in hand.
BOOL annotation_image_gizmo(ScreenwidePreviewSurface *surface, ScreenwideImageGizmo *gizmo);
/// The ring of the chosen image's gizmo under `point`; its axis is -1 where
/// there is none.
ScreenwideTurnRingHit annotation_image_ring_at(ScreenwidePreviewSurface *surface,
                                               NSPoint point);
/// A press, drag and release of a ring of the chosen image's gizmo. Each
/// answers YES when the event was the gizmo's.
BOOL image_turn_mouse_down(ScreenwidePreviewInteractionView *view, NSEvent *event,
                           NSPoint point);
BOOL image_turn_mouse_dragged(ScreenwidePreviewInteractionView *view, NSEvent *event);
BOOL image_turn_mouse_up(ScreenwidePreviewInteractionView *view, NSEvent *event);

#endif
