// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "../../../../osc/gpu/macos/osc_gpu_macos.h"
#import "recording_preview_surface_macos_private.h"
#include <math.h>
#include "recording_preview_annotation_layers_macos.h"

/// A press has to travel this far before it draws an arrow rather than
/// clearing the choice: a click and a very short drag are the same gesture to
/// a hand, and neither should leave a stub behind.
static const CGFloat kAnnotationDragSlop = 3.0;

SCREENWIDE_PREVIEW_PRIVATE const ScreenwidePreviewAnnotation *annotation_items(
    ScreenwidePreviewSurface *surface, NSUInteger *count) {
  NSUInteger items = surface.annotations.length / sizeof(ScreenwidePreviewAnnotation);
  *count = items;
  return *count == 0 ? NULL : surface.annotations.bytes;
}

/// The whole source image's rectangle on screen, in this flipped view's
/// coordinates. Every normalised handle is placed inside it.
SCREENWIDE_PREVIEW_PRIVATE NSRect annotation_image_frame(
    ScreenwidePreviewSurface *surface) {
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  NSInteger selected = surface.annotationSelected;
  int32_t layer = selected >= 0 && (NSUInteger)selected < count ? items[selected].layer_id : -1;
  return annotation_layer_image(surface, layer);
}

/// Whether a tool in hand makes a new annotation on empty picture.
SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_drawing_mode(ScreenwideAnnotationMode mode) {
  return mode == ScreenwideAnnotationModeArrow ||
         mode == ScreenwideAnnotationModeCounter ||
         mode == ScreenwideAnnotationModeText || mode == ScreenwideAnnotationModeRedact ||
         mode == ScreenwideAnnotationModeHighlight || mode == ScreenwideAnnotationModeShape ||
         mode == ScreenwideAnnotationModeSpotlight || mode == ScreenwideAnnotationModeDraw ||
         mode == ScreenwideAnnotationModeMagnify;
}

/// What a press does if it never travels: an ordinary press, a toggle of the
/// annotation under it, a carry of the group it landed on, or a marquee band
/// that has not been drawn out yet.
enum {
  AnnotationPressOrdinary = 0,
  AnnotationPressToggle = 1,
  AnnotationPressGroup = 2,
  AnnotationPressMarquee = 3,
};

/// One sample of a marquee band: its corners go to Rust at its two ends, and
/// the band itself is drawn from here in between.
static void annotation_marquee_sample(ScreenwidePreviewInteractionView *view, uint32_t phase,
                                      NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  NSPoint origin = view.annotationDragOrigin;
  surface.annotationMarquee =
      phase == 2 ? NSZeroRect
                 : NSMakeRect(MIN(origin.x, point.x), MIN(origin.y, point.y),
                              fabs(point.x - origin.x), fabs(point.y - origin.y));
  if (phase != 1)
    emit_annotation_layer_gesture(surface, phase, ScreenwideAnnotationTargetMarquee,
                                  view.annotationDragLayer, point);
  redraw_selection(surface);
}

/// One sample of the drag the press under way turned into.
static void annotation_emit_drag(ScreenwidePreviewInteractionView *view, uint32_t phase,
                                 NSPoint point) {
  if (view.annotationPressKind == AnnotationPressMarquee)
    annotation_marquee_sample(view, phase, point);
  else if (view.annotationPressKind == AnnotationPressGroup)
    emit_annotation_layer_gesture(view.surface, phase, ScreenwideAnnotationTargetGroup,
                                  view.annotationDragLayer, point);
  else
    emit_annotation_gesture(view.surface, phase, view.annotationDragTargetKind,
                            view.annotationDragIndex, view.annotationDragHandle, point);
}

/// Arms a press that carries the chosen group once it travels. `index` is the
/// member pressed, or `UINT32_MAX` for a press inside the group's box.
static BOOL annotation_press_group(ScreenwidePreviewInteractionView *view, uint32_t index,
                                   int32_t layer) {
  view.annotationPressKind = AnnotationPressGroup;
  view.annotationDragLayer = layer;
  view.annotationDragTargetKind = ScreenwideAnnotationTargetGroup;
  view.annotationDragIndex = index;
  view.annotationDragHandle = ScreenwideAnnotationHandleBody;
  view.annotationDragPending = YES;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_mouse_down(
    ScreenwidePreviewInteractionView *view, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwideAnnotationMode mode = annotation_active_mode(surface);
  if (mode == ScreenwideAnnotationModeNone) return NO;
  // A press is never a hover: the halo goes out before anything moves.
  annotation_update_hover(surface, point, YES);
  NSInteger handle = annotation_handle_at_point(surface, point);
  NSInteger shaft = handle >= 0 ? -1 : annotation_shaft_at_point(surface, point);
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  BOOL toggles = annotation_press_toggles();
  BOOL marquee = mode == ScreenwideAnnotationModeMarquee;
  // Empty picture under the marquee draws a band over the layer beneath it.
  int32_t marqueeLayer = marquee && handle < 0 && shaft < 0
                             ? annotation_marquee_layer_at_point(surface, point)
                             : INT32_MIN;
  // Inside the box round a group, with only the select tool in hand, a press
  // takes hold of the group: it carries every member, shown or not.
  int32_t groupLayer =
      handle < 0 && shaft < 0 && !toggles && !marquee && !annotation_drawing_mode(mode)
          ? annotation_group_layer_at_point(surface, point)
          : INT32_MIN;
  if (handle < 0 && shaft < 0 && !annotation_drawing_mode(mode) && groupLayer == INT32_MIN &&
      marqueeLayer == INT32_MIN) {
    // Empty picture with only the select tool in hand: the arrow chrome lets
    // go, and the press carries on to the layer underneath. Recording
    // selection clears the annotation together with the new layer; screenshots
    // still publish their selected-image annotation document separately.
    NSInteger selected = surface.annotationSelected;
    BOOL still = selected >= 0 && (NSUInteger)selected < count
                     ? items[selected].layer_id < 0
                     : annotation_has_group(surface) && count > 0 && items[0].layer_id < 0;
    if (still) {
      surface.annotationSelected = -1;
      emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetNone, 0,
                              ScreenwideAnnotationHandleBody, point);
    }
    return NO;
  }
  view.panning = NO;
  view.selectionDragActive = NO;
  view.annotationDragActive = YES;
  view.annotationDragBegun = NO;
  view.annotationDragPending = NO;
  view.annotationDragOrigin = point;
  view.annotationPressKind = AnnotationPressOrdinary;
  if (marqueeLayer != INT32_MIN) {
    // The band waits for the press to travel; a click lets the choice go,
    // unless the toggle modifier is held at either end. `index` keeps whether
    // it was held at the press, for the release.
    view.annotationPressKind = AnnotationPressMarquee;
    view.annotationDragLayer = marqueeLayer;
    view.annotationDragTargetKind = ScreenwideAnnotationTargetMarquee;
    view.annotationDragIndex = toggles ? 1 : 0;
    view.annotationDragPending = YES;
    return YES;
  }
  if (groupLayer != INT32_MIN) return annotation_press_group(view, UINT32_MAX, groupLayer);
  if (handle >= 0) {
    // A grip waits for the press to travel before it begins, exactly as a new
    // arrow does: a click that lands on a handle must not nudge the arrow by
    // the few points between the handle's centre and the pointer, nor leave an
    // edit in the history for it.
    view.annotationDragTargetKind = ScreenwideAnnotationTargetExisting;
    view.annotationDragIndex = (uint32_t)surface.annotationSelected;
    view.annotationDragHandle = (uint32_t)handle;
    view.annotationDragPending = YES;
    return YES;
  }
  if (shaft >= 0) {
    // One of several chosen together carries them all; a click on it alone
    // chooses it on its own, once the press has proved not to be a drag.
    if (!toggles && (items[shaft].flags & ScreenwideAnnotationFlagGrouped) != 0)
      return annotation_press_group(view, (uint32_t)shaft, items[shaft].layer_id);
    // Choosing an arrow is complete on the press: Rust commits the choice and
    // the OSC moves to it. The press may still turn into a move of the whole
    // arrow, which begins as its own gesture once it has travelled - exactly
    // as a grip does, so a click never leaves an edit in the history. With the
    // toggle modifier the choice waits for the release: a click adds or
    // removes the arrow, and a drag moves it as any other.
    view.annotationDragTargetKind = ScreenwideAnnotationTargetExisting;
    view.annotationDragIndex = (uint32_t)shaft;
    view.annotationDragHandle = annotation_body_handle(surface, shaft, point);
    view.annotationDragPending = YES;
    if (toggles)
      view.annotationPressKind = AnnotationPressToggle;
    else
      annotation_choose(surface, shaft, point);
    return YES;
  }
  // Empty picture: a new annotation. An arrow is drawn out, so it waits for the
  // press to prove a drag and a click leaves no stub behind. A counter and a
  // text box are dropped whole where the press lands, so they begin at once
  // and a click alone makes them; the drag that may follow carries them.
  view.annotationDragTargetKind = ScreenwideAnnotationTargetNew;
  view.annotationDragIndex = 0;
  if (mode == ScreenwideAnnotationModeCounter || mode == ScreenwideAnnotationModeText) {
    view.annotationDragHandle = ScreenwideAnnotationHandleBody;
    view.annotationDragBegun = YES;
    emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetNew, 0,
                            ScreenwideAnnotationHandleBody, point);
    return YES;
  }
  view.annotationDragHandle = ScreenwideAnnotationHandleEnd;
  view.annotationDragPending = YES;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_mouse_dragged(
    ScreenwidePreviewInteractionView *view, NSPoint point) {
  if (view.annotationPressIgnored) return YES;
  if (!view.annotationDragActive) return NO;
  if (view.annotationDragPending) {
    if (hypot(point.x - view.annotationDragOrigin.x,
              point.y - view.annotationDragOrigin.y) < kAnnotationDragSlop)
      return YES;
    // A toggle press that travels is an ordinary press: one of a group
    // carries the group, with Command snapping it as it goes, and any other
    // arrow is chosen as the press would have chosen it, then carried.
    if (view.annotationPressKind == AnnotationPressToggle) {
      NSUInteger count = 0;
      const ScreenwidePreviewAnnotation *items = annotation_items(view.surface, &count);
      uint32_t index = view.annotationDragIndex;
      if (index < count && (items[index].flags & ScreenwideAnnotationFlagGrouped) != 0) {
        annotation_press_group(view, index, items[index].layer_id);
      } else {
        view.annotationPressKind = AnnotationPressOrdinary;
        annotation_choose(view.surface, index, view.annotationDragOrigin);
      }
    }
    view.annotationDragPending = NO;
    view.annotationDragBegun = YES;
    annotation_emit_drag(view, 0, view.annotationDragOrigin);
  }
  annotation_emit_drag(view, 1, point);
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_mouse_up(
    ScreenwidePreviewInteractionView *view, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  if (view.annotationPressIgnored) {
    view.annotationPressIgnored = NO;
    return YES;
  }
  if (!view.annotationDragActive) return NO;
  if (view.annotationDragBegun) {
    annotation_emit_drag(view, 2, point);
  } else if (view.annotationPressKind == AnnotationPressToggle) {
    emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetToggle,
                            view.annotationDragIndex, ScreenwideAnnotationHandleBody, point);
  } else if (view.annotationPressKind == AnnotationPressGroup) {
    // A click on one of the group chooses it alone; a click inside the box
    // but on no member lets the group go.
    if (view.annotationDragIndex != UINT32_MAX)
      annotation_choose(surface, view.annotationDragIndex, point);
    else
      emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetNone, 0,
                              ScreenwideAnnotationHandleBody, point);
  } else if (view.annotationPressKind == AnnotationPressMarquee && view.annotationDragIndex == 0 &&
             !annotation_press_toggles()) {
    // A click with the marquee on empty picture lets the choice go.
    emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetNone, 0,
                            ScreenwideAnnotationHandleBody, point);
  }
  view.annotationDragActive = NO;
  view.annotationDragPending = NO;
  view.annotationDragBegun = NO;
  view.annotationPressKind = AnnotationPressOrdinary;
  return YES;
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_add_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  NSUInteger items = 0;
  const ScreenwidePreviewAnnotation *list = annotation_items(surface, &items);
  // Several chosen together wear boxes round each and round them all, and a
  // marquee band shows the ground it is covering.
  annotation_group_add_osc(vertices, count, size, surface, scale);
  annotation_marquee_add_osc(vertices, count, size, surface, scale);
  // A chosen redaction or shape wears the selection's box rather than discs.
  if (annotation_redact_add_osc(vertices, count, size, surface, scale)) return;
  if (list == NULL || surface.annotationSelected < 0 ||
      (NSUInteger)surface.annotationSelected >= items)
    return;
  NSPoint handles[SCREENWIDE_ANNOTATION_MAX_GRIPS];
  uint32_t kinds[SCREENWIDE_ANNOTATION_MAX_GRIPS];
  NSUInteger grips = annotation_grips(annotation_image_frame(surface),
                                      list[surface.annotationSelected], handles, kinds);
  // The same disc the selection OSC draws its corner grips with: a 4pt fill
  // with a one-device-pixel ring, snapped to a whole device pixel.
  CGFloat extent = 4.0 + 2.0 / scale;
  for (NSUInteger index = 0; index < grips; index++) {
    CGFloat x = round(handles[index].x * scale) / scale;
    CGFloat y = round(handles[index].y * scale) / scale;
    screenwide_region_osc_add_texture_quad(
        vertices, count, size,
        NSMakeRect(x - extent, y - extent, extent * 2.0, extent * 2.0),
        NSMakeRect(0.0, 0.0, 1.0, 1.0), 3);
  }
}
