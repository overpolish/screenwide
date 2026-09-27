// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include "recording_preview_annotation_layers_macos.h"

/// A point on screen, back in the layer's image-normalised space. This is
/// exactly what Rust turns into source pixels.
static BOOL annotation_normalised_point(ScreenwidePreviewSurface *surface,
                                        NSPoint point, double *x, double *y) {
  NSRect image = annotation_image_frame(surface);
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

SCREENWIDE_PREVIEW_PRIVATE void emit_annotation_gesture(
    ScreenwidePreviewSurface *surface, uint32_t phase, uint32_t targetKind,
    uint32_t index, uint32_t handle, NSPoint point) {
  if (surface.annotationGestureCallback == NULL) return;
  double x = 0.0;
  double y = 0.0;
  if (!annotation_normalised_point(surface, point, &x, &y)) return;
  // The macOS workspace draws every layer into one pane, so the selection's
  // `pane_index` is always 0 and `layer_id` carries the workspace order the
  // gesture addresses - the same identity `emit_selection_gesture` reports.
  uint32_t layer = surface.selection.layer_id;
  NSUInteger count = 0;
  const ScreenwidePreviewAnnotation *items = annotation_items(surface, &count);
  if ((targetKind == ScreenwideAnnotationTargetExisting || targetKind == ScreenwideAnnotationTargetSelect) && index < count) {
    if (items[index].layer_id >= 0) layer = (uint32_t)items[index].layer_id;
    index = items[index].index;
  }
  surface.annotationGestureCallback(phase, layer,
                                    targetKind, index, handle, x, y,
                                    annotation_snapped(),
                                    annotation_image_frame(surface).size.width,
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
