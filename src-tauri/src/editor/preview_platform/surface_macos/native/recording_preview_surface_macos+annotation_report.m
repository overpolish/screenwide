// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include "recording_preview_annotation_layers_macos.h"

/// A point on screen, back in `image`'s normalised space. This is exactly
/// what Rust turns into source pixels.
static BOOL annotation_normalised_point(NSRect image, NSPoint point, double *x, double *y) {
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return NO;
  *x = (point.x - NSMinX(image)) / image.size.width;
  *y = (point.y - NSMinY(image)) / image.size.height;
  return YES;
}

/// Which snapping modifiers this sample was taken with: bit 0 Shift, bit 1
/// Command. Both are read at the moment the sample is reported rather than
/// latched at the press, so either can be taken and let go part way through a
/// drag.
static uint32_t annotation_snapped(void) {
  NSEventModifierFlags flags = [NSEvent modifierFlags];
  return ((flags & NSEventModifierFlagShift) != 0 ? 1u : 0u) |
         ((flags & NSEventModifierFlagCommand) != 0 ? 2u : 0u);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_press_toggles(void) {
  return ([NSEvent modifierFlags] & NSEventModifierFlagCommand) != 0;
}

SCREENWIDE_PREVIEW_PRIVATE void emit_annotation_gesture(
    ScreenwidePreviewSurface *surface, uint32_t phase, uint32_t targetKind,
    uint32_t index, uint32_t handle, NSPoint point) {
  if (surface.annotationGestureCallback == NULL) return;
  // The macOS workspace draws every layer into one pane, so the selection's
  // `pane_index` is always 0 and `layer_id` carries the workspace order the
  // gesture addresses - the same identity `emit_selection_gesture` reports.
  uint32_t layer = surface.selection.layer_id;
  NSRect image = annotation_image_frame(surface);
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if ((targetKind == ScreenwideAnnotationTargetExisting ||
       targetKind == ScreenwideAnnotationTargetSelect ||
       targetKind == ScreenwideAnnotationTargetToggle) &&
      index < count) {
    // An annotation is measured on its own layer, whichever one is chosen.
    image = annotation_layer_image(surface, items[index].layer_id);
    if (items[index].layer_id >= 0) layer = (uint32_t)items[index].layer_id;
    index = items[index].index;
  }
  double x = 0.0;
  double y = 0.0;
  if (!annotation_normalised_point(image, point, &x, &y)) return;
  surface.annotationGestureCallback(phase, layer, targetKind, index, handle, x, y,
                                    annotation_snapped(), image.size.width,
                                    surface.annotationGestureContext);
}

SCREENWIDE_PREVIEW_PRIVATE void emit_annotation_layer_gesture(
    ScreenwidePreviewSurface *surface, uint32_t phase, uint32_t targetKind, int32_t layer,
    NSPoint point) {
  if (surface.annotationGestureCallback == NULL) return;
  NSRect image = annotation_layer_image(surface, layer);
  double x = 0.0;
  double y = 0.0;
  if (!annotation_normalised_point(image, point, &x, &y)) return;
  // A still's annotations carry no layer of their own: the selection's is
  // theirs, as it is for every other gesture on a still.
  uint32_t pane = layer >= 0 ? (uint32_t)layer : surface.selection.layer_id;
  surface.annotationGestureCallback(phase, pane, targetKind, 0, ScreenwideAnnotationHandleBody, x,
                                    y, annotation_snapped(), image.size.width,
                                    surface.annotationGestureContext);
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_choose(ScreenwidePreviewSurface *surface,
                                                  NSInteger index, NSPoint point) {
  surface.annotationSelected = index;
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  ScreenwidePreviewSelection target;
  if (annotation_layer_selection(surface, items[index].layer_id, &target)) {
    surface.selection = target;
    surface.hasSelection = YES;
  }
  emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetSelect,
                          (uint32_t)index, ScreenwideAnnotationHandleBody,
                          point);
}

// The rule `drawing_layer` keeps in `src-tauri/src/editor/annotations/gesture.rs`:
// the topmost picture, the canvas frame and the keyboard overlay being none.
SCREENWIDE_PREVIEW_PRIVATE void annotation_take_layer_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  for (NSValue *value in surface.selectionTargets.reverseObjectEnumerator) {
    ScreenwidePreviewSelection target;
    [value getValue:&target size:sizeof(target)];
    if (target.layer_id == ScreenwideFrameLayerId || selection_is_keyboard(target)) continue;
    if (!NSPointInRect(point, selection_image_frame_for(surface, target))) continue;
    if (surface.hasSelection && surface.selection.pane_index == target.pane_index &&
        surface.selection.layer_id == target.layer_id)
      return;
    surface.hasSelection = YES;
    surface.selection = target;
    surface.annotationSelected = -1;
    clear_selection_snap_guides(surface);
    if (surface.selectionCallback != NULL)
      surface.selectionCallback((int32_t)target.layer_id, surface.selectionContext);
    redraw_selection(surface);
    invalidate_selection_cursor_rects(surface);
    return;
  }
}

// Whether `point` lands on a picture a fresh annotation could join. With no
// pictures laid out there is nothing to miss, so every press is on one. The
// twin of `over_a_picture` in `src-tauri/src/editor/annotations/gesture.rs`.
SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_picture_at_point(ScreenwidePreviewSurface *surface,
                                                           NSPoint point) {
  BOOL any = NO;
  for (NSValue *value in surface.selectionTargets) {
    ScreenwidePreviewSelection target;
    [value getValue:&target size:sizeof(target)];
    if (target.layer_id == ScreenwideFrameLayerId || selection_is_keyboard(target)) continue;
    any = YES;
    if (NSPointInRect(point, selection_image_frame_for(surface, target))) return YES;
  }
  return !any;
}

// A press on no annotation under the sticker tool places nothing while
// something is in hand, or off every picture: it lets the choice go, so the
// next sticker's picture can be chosen, and goes no further, leaving the
// frame and the layers as they were. The next press on the picture places.
// `NO` leaves the press to place a sticker.
SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_sticker_lets_go(ScreenwidePreviewInteractionView *view,
                                                          NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  BOOL held = surface.annotationSelected >= 0 || annotation_has_group(surface);
  if (!held && annotation_picture_at_point(surface, point)) return NO;
  if (held) {
    surface.annotationSelected = -1;
    emit_annotation_gesture(surface, 0, ScreenwideAnnotationTargetNone, 0,
                            ScreenwideAnnotationHandleBody, point);
  }
  view.annotationPressIgnored = YES;
  return YES;
}
