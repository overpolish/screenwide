// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_MACOS_H

#import <AppKit/AppKit.h>
#include <stdint.h>

/// One annotation's grips, normalised over the whole source image, and how to
/// read them.
///
/// An arrow fills every slot: its three grips - the two tips and the point the
/// curve passes through at t = 0.5 - how far each head reaches back from its
/// tip, and the stroke's own width, both as a fraction of the image's drawn
/// width. A head reach of zero means that end carries no head; the half-base is
/// half the length, which is the shader's four-to-two proportions. `width`
/// rides separately because a headless annotation still has a stroke to pick.
///
/// A counter puts its disc's centre in every point slot, where its tail points
/// in `start_head` - radians clockwise from east - and its disc's diameter in
/// `width`. Its one grip, the tail's tip, is placed from those here: a
/// normalised offset is a different length in each axis on a picture that is
/// not square, and this side works in isotropic display points.
///
/// Rust solves the Bezier and owns the stroke's units; this side only places
/// and hit-tests.
typedef struct {
  double start_x, start_y;
  double middle_x, middle_y;
  double end_x, end_y;
  double start_head, end_head;
  double width;
  int32_t layer_id;
  uint32_t index;
  /// `ScreenwideAnnotationKind`.
  uint32_t kind;
  uint32_t padding;
} ScreenwidePreviewAnnotation;
_Static_assert(sizeof(ScreenwidePreviewAnnotation) == 88,
               "Rust/C annotation handle layout mismatch");
/// How many annotations one layer can carry, matching `MAX_ANNOTATIONS`.
static const NSUInteger ScreenwideMaxAnnotations = 64;
/// Which shape an annotation is, matching the compositor's own kinds.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationKind) {
  ScreenwideAnnotationKindArrow = 0,
  ScreenwideAnnotationKindCounter = 1,
};
/// What the pointer does over the picture. `Select` hit-tests the annotations
/// that are already there and lets everything else fall through to the
/// layer; `Arrow` and `Counter` also make a new annotation on empty picture.
typedef NS_ENUM(int32_t, ScreenwideAnnotationMode) {
  ScreenwideAnnotationModeNone = 0,
  ScreenwideAnnotationModeSelect = 1,
  ScreenwideAnnotationModeArrow = 2,
  ScreenwideAnnotationModeCounter = 3,
};
/// Which grip a press took hold of.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationHandle) {
  ScreenwideAnnotationHandleStart = 0,
  ScreenwideAnnotationHandleMiddle = 1,
  ScreenwideAnnotationHandleEnd = 2,
  ScreenwideAnnotationHandleBody = 3,
  ScreenwideAnnotationHandleTail = 4,
};
/// What a gesture acts on: a new annotation (0), a grip or body of the
/// annotation at `index` (1), nothing at all (2), which only clears the choice,
/// or a press on the body of the annotation at `index` (3), which only chooses
/// it.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationTarget) {
  ScreenwideAnnotationTargetNew = 0,
  ScreenwideAnnotationTargetExisting = 1,
  ScreenwideAnnotationTargetNone = 2,
  ScreenwideAnnotationTargetSelect = 3,
};
/// One equal gap the snap chrome draws a bar across, normalised over the
/// source: where it starts and ends along its own axis, and where it sits
/// across it.
typedef struct {
  double from, to, cross;
} ScreenwideAnnotationGapSpan;
/// What the snap chrome draws, normalised over the source image exactly as
/// the grips are, matching Rust's `NativeAnnotationSnap`. `flags` says which
/// members are live; a zeroed value draws nothing.
typedef struct {
  uint32_t flags;
  uint32_t guide_x_object, guide_y_object;
  uint32_t padding;
  double guide_x, guide_y;
  double anchor_x, anchor_y;
  double box_x, box_y, box_width, box_height;
  ScreenwideAnnotationGapSpan gap_x[2], gap_y[2];
} ScreenwideAnnotationSnap;
_Static_assert(sizeof(ScreenwideAnnotationSnap) == 176,
               "Rust/C annotation snap layout mismatch");
/// Which members of `ScreenwideAnnotationSnap` are live.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationSnapFlag) {
  ScreenwideAnnotationSnapGuideX = 1 << 0,
  ScreenwideAnnotationSnapGuideY = 1 << 1,
  ScreenwideAnnotationSnapAnchor = 1 << 2,
  ScreenwideAnnotationSnapGapX = 1 << 3,
  ScreenwideAnnotationSnapGapY = 1 << 4,
};
/// `snap` is which snapping modifiers were held for this sample: bit 0 Shift,
/// which holds a counter's tail to the quarter turns, and bit 1 Command,
/// which snaps the position itself to the candidates around it. `image_points`
/// is how wide the layer's picture is drawn on screen, which is what turns
/// the snap's reach in screen points into the source pixels Rust works in.
typedef void (*screenwide_preview_annotation_gesture_callback)(
    uint32_t phase, uint32_t pane_index, uint32_t target_kind, uint32_t index,
    uint32_t handle, double x, double y, uint32_t snap, double image_points,
    void *context);
/// The hover halo's progress. `index` is the arrow the pointer is over, or -1
/// for none; `progress` runs 0..1 over the pulse; `image_points` is how wide
/// the layer's picture is drawn on screen, which is all Rust needs to turn
/// the halo's points into the canvas pixels the compositor draws in.
typedef void (*screenwide_preview_annotation_hover_callback)(
    int32_t index, double progress, double image_points, void *context);

#endif
