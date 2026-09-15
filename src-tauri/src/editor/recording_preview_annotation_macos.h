// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_MACOS_H

#import <AppKit/AppKit.h>
#include <stdint.h>

/// One arrow's three grips, normalised over the whole source image - the two
/// tips and the point the curve passes through at t = 0.5 - and how far each
/// head reaches back from its tip, as a fraction of the image's drawn width.
/// Zero means that end carries no head; the half-base is half the length,
/// which is the shader's four-to-two proportions. Rust solves the Bezier and
/// owns the stroke's units; this side only places and hit-tests the points.
typedef struct {
  double start_x, start_y;
  double middle_x, middle_y;
  double end_x, end_y;
  double start_head, end_head;
  int32_t layer_id;
  uint32_t index;
} ScreenwidePreviewAnnotation;
_Static_assert(sizeof(ScreenwidePreviewAnnotation) == 72,
               "Rust/C annotation handle layout mismatch");
/// How many arrows one layer can carry, matching `MAX_ANNOTATIONS`.
static const NSUInteger ScreenwideMaxAnnotations = 64;
/// What the pointer does over the picture. `Select` hit-tests the arrows
/// that are already there and lets everything else fall through to the
/// layer; `Arrow` also draws a new one on empty picture.
typedef NS_ENUM(int32_t, ScreenwideAnnotationMode) {
  ScreenwideAnnotationModeNone = 0,
  ScreenwideAnnotationModeSelect = 1,
  ScreenwideAnnotationModeArrow = 2,
};
/// Which grip a press took hold of.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationHandle) {
  ScreenwideAnnotationHandleStart = 0,
  ScreenwideAnnotationHandleMiddle = 1,
  ScreenwideAnnotationHandleEnd = 2,
  ScreenwideAnnotationHandleBody = 3,
};
/// What a gesture acts on: a new arrow (0), a grip or shaft of the arrow at
/// `index` (1), nothing at all (2), which only clears the choice, or a press
/// on the shaft of the arrow at `index` (3), which only chooses it.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationTarget) {
  ScreenwideAnnotationTargetNew = 0,
  ScreenwideAnnotationTargetExisting = 1,
  ScreenwideAnnotationTargetNone = 2,
  ScreenwideAnnotationTargetSelect = 3,
};
typedef void (*screenwide_preview_annotation_gesture_callback)(
    uint32_t phase, uint32_t pane_index, uint32_t target_kind, uint32_t index,
    uint32_t handle, double x, double y, void *context);
/// The hover halo's progress. `index` is the arrow the pointer is over, or -1
/// for none; `progress` runs 0..1 over the pulse; `image_points` is how wide
/// the layer's picture is drawn on screen, which is all Rust needs to turn
/// the halo's points into the canvas pixels the compositor draws in.
typedef void (*screenwide_preview_annotation_hover_callback)(
    int32_t index, double progress, double image_points, void *context);

#endif
