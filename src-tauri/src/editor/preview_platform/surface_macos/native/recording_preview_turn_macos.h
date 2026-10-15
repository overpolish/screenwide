// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_TURN_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_TURN_MACOS_H

#import <AppKit/AppKit.h>
#include <stdint.h>

#import "../../../../../crates/osc/src/gpu/macos/osc_gpu_macos.h"
#import "recording_preview_selection_macos.h"

// The rotation gizmo and the turned layer's chrome, `+turn.m`. Where a scene
// turns the selected layer, its chrome is laid out as for the upright layer
// and carried through the turn, and presses on it are carried back.

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
/// Carries the chrome vertices from `from` up to `count`, laid out on the
/// selected layer upright, onto it as turned.
void turn_vertices(ScreenwidePreviewSurface *surface, ScreenwideRegionOscVertex *vertices,
                   NSUInteger from, NSUInteger count, NSSize size);
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

#endif
