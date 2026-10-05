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
/// An image's corner radius rides in `radius`, the one slot none of its
/// others is free for; every other kind leaves it zero.
///
/// Rust solves the Bezier and owns the stroke's units; this side only places
/// and hit-tests.
typedef struct {
  double start_x, start_y;
  double middle_x, middle_y;
  double end_x, end_y;
  double start_head, end_head;
  double width;
  double radius;
  int32_t layer_id;
  uint32_t index;
  /// `ScreenwideAnnotationKind`.
  uint32_t kind;
  /// `ScreenwideAnnotationFlag` bits.
  uint32_t flags;
} ScreenwidePreviewAnnotation;
_Static_assert(sizeof(ScreenwidePreviewAnnotation) == 96,
               "Rust/C annotation handle layout mismatch");
/// What a record's `flags` say. `Grouped` is an annotation chosen together
/// with others: a press on it carries the group rather than choosing it.
typedef NS_OPTIONS(uint32_t, ScreenwideAnnotationFlag) {
  ScreenwideAnnotationFlagGrouped = 1u << 0,
};
/// One box the chrome draws round a group, normalised over its layer's
/// source image, matching Rust's `NativeAnnotationGroupBox`: round one member
/// shown at the playhead, or round every member on the layer, shown or not.
typedef struct {
  double left, top, right, bottom;
  int32_t layer_id;
  /// `ScreenwideAnnotationGroupBoxKind`.
  uint32_t kind;
} ScreenwideAnnotationGroupBox;
_Static_assert(sizeof(ScreenwideAnnotationGroupBox) == 40,
               "Rust/C annotation group box layout mismatch");
typedef NS_ENUM(uint32_t, ScreenwideAnnotationGroupBoxKind) {
  ScreenwideAnnotationGroupBoxMember = 0,
  ScreenwideAnnotationGroupBoxWhole = 1,
};
/// Which shape an annotation is, matching the compositor's own kinds.
///
/// A text box reads the grip slots its own way: `start` is the box's top-left
/// corner; `end` is its pointer as Rust's `TextPointer::encoded` carries it,
/// held against the box and never placed; `middle` is its text block's width
/// and height and `width` its type size, both as shares of the drawn width;
/// `start_head` is its alignment.
///
/// A redaction puts its box's top-left corner in `start`, its bottom-right in
/// `end` and its centre in `middle`, all normalised like an arrow's grips,
/// and its corner radius in `start_head`. A shape reads them the same way,
/// and adds its stroke's hand in `end_head` and its pen in `width`; a
/// spotlight reads them exactly as a redaction does.
///
/// A highlight puts its first band's top-left corner in `start` and its last
/// band's bottom-right in `end`; `middle_x` and `middle_y` are how far left
/// and right its block of bands reaches, `start_head` its first band's bottom
/// and `end_head` its last band's top. Its two grips are the selection's ends.
///
/// A stroke puts its box in `start` and `end` and its centre in `middle`,
/// as a redaction does, its pen in `width`, and in `start_head` and
/// `end_head` where its fitted line starts in the published paths and how
/// many points it has.
///
/// A magnifier puts its zoom area's box in `start` and `end` and its corner
/// radius in `start_head`, as a spotlight does; its loupe's centre in
/// `middle`, the loupe's longer side as a share of the drawn width in
/// `end_head` and its rim's pen in `width`. Its grips are the zoom area's
/// box's and one on the loupe's rim, which sizes the loupe; a press inside
/// the loupe carries the loupe.
///
/// An image puts the upright box its turned picture fits in in `start` and
/// `end`, its middle in `middle`, its turn in `start_head`, and its half
/// width and half height as shares of the drawn width in `end_head` and
/// `width`. Its grips are that box's, which size the picture, and one above
/// its top side, which turns it.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationKind) {
  ScreenwideAnnotationKindArrow = 0,
  ScreenwideAnnotationKindCounter = 1,
  ScreenwideAnnotationKindText = 2,
  ScreenwideAnnotationKindRedact = 3,
  ScreenwideAnnotationKindHighlight = 4,
  ScreenwideAnnotationKindShape = 5,
  ScreenwideAnnotationKindSpotlight = 6,
  ScreenwideAnnotationKindDraw = 7,
  ScreenwideAnnotationKindMagnify = 8,
  ScreenwideAnnotationKindImage = 9,
};
/// What the pointer does over the picture. `Select` hit-tests the annotations
/// that are already there and lets everything else fall through to the
/// layer; the drawing tools also make a new annotation on empty picture; the
/// marquee picks up as `Select` does and draws a band over empty picture.
typedef NS_ENUM(int32_t, ScreenwideAnnotationMode) {
  ScreenwideAnnotationModeNone = 0,
  ScreenwideAnnotationModeSelect = 1,
  ScreenwideAnnotationModeArrow = 2,
  ScreenwideAnnotationModeCounter = 3,
  ScreenwideAnnotationModeText = 4,
  ScreenwideAnnotationModeRedact = 5,
  ScreenwideAnnotationModeHighlight = 6,
  ScreenwideAnnotationModeShape = 7,
  ScreenwideAnnotationModeSpotlight = 8,
  ScreenwideAnnotationModeDraw = 9,
  ScreenwideAnnotationModeMagnify = 10,
  ScreenwideAnnotationModeMarquee = 11,
  ScreenwideAnnotationModeImage = 12,
};
/// Which grip a press took hold of. A box's grips - a redaction's, a shape's
/// or a spotlight's - report `Box` plus the sides they move - 1 left, 2 right, 4 top,
/// 8 bottom - the same bits the resize cursors are chosen by and Rust's
/// `AnnotationHandle::Edges` reads. Its radius dot, the layer selection's
/// own, reports `Radius`.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationHandle) {
  ScreenwideAnnotationHandleStart = 0,
  ScreenwideAnnotationHandleMiddle = 1,
  ScreenwideAnnotationHandleEnd = 2,
  ScreenwideAnnotationHandleBody = 3,
  ScreenwideAnnotationHandleTail = 4,
  ScreenwideAnnotationHandleBox = 16,
  ScreenwideAnnotationHandleRadius = 32,
};
/// The most grips one annotation shows: a box's eight, its radius dot and a
/// magnifier's loupe grip.
#define SCREENWIDE_ANNOTATION_MAX_GRIPS 10
/// What a gesture acts on: a new annotation (0), a grip or body of the
/// annotation at `index` (1), nothing at all (2), which only clears the choice,
/// a press on the body of the annotation at `index` (3), which only chooses
/// it, a carry of every annotation chosen together (4), a click with the
/// toggle modifier on the annotation at `index` (5), which adds it to the
/// choice or takes it out, or a marquee band (6), whose begin and end are its
/// two corners.
typedef NS_ENUM(uint32_t, ScreenwideAnnotationTarget) {
  ScreenwideAnnotationTargetNew = 0,
  ScreenwideAnnotationTargetExisting = 1,
  ScreenwideAnnotationTargetNone = 2,
  ScreenwideAnnotationTargetSelect = 3,
  ScreenwideAnnotationTargetGroup = 4,
  ScreenwideAnnotationTargetToggle = 5,
  ScreenwideAnnotationTargetMarquee = 6,
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

/// A text box being typed into. `phase` is 0 when a double-click asks to open
/// the box at `index` in `layer`, 1 when its text changed to `text`, and 2
/// when typing ended. `revision` counts the changes of one session, so a
/// change delivered late never overwrites a newer one.
typedef void (*screenwide_preview_annotation_text_callback)(
    uint32_t phase, uint32_t layer, uint32_t index, const uint8_t *text,
    uint32_t length, uint64_t revision, void *context);

#endif
