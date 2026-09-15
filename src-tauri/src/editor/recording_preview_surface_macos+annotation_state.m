// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include "recording_preview_annotation_layers_macos.h"

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

void screenwide_preview_surface_set_annotations(
    void *handle, const ScreenwidePreviewAnnotation *items, size_t count,
    int32_t selected_index, int32_t mode, int32_t active_layer) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  // The block outlives this call, so the caller's array is copied here while
  // it is still alive and only the copy is captured.
  NSUInteger copied = MIN((NSUInteger)count, ScreenwideMaxAnnotations);
  NSMutableData *data = [NSMutableData
      dataWithBytes:(copied == 0 ? NULL : items)
             length:copied * sizeof(ScreenwidePreviewAnnotation)];
  NSInteger selected = selected_index;
  ScreenwideAnnotationMode next = (ScreenwideAnnotationMode)mode;
  on_main_async(^{
    // Selection and its annotation chrome change in the same main-queue turn.
    ScreenwidePreviewSelection target;
    if (next != ScreenwideAnnotationModeNone && active_layer >= 0 && annotation_layer_selection(surface, active_layer, &target)) {
      surface.selection = target;
      surface.hasSelection = YES;
    }
    surface.annotations = data;
    surface.annotationSelected = selected;
    BOOL changed = surface.annotationMode != next;
    surface.annotationMode = next;
    if (next == ScreenwideAnnotationModeNone) {
      // No tool, no halo: the pointer may never move again to retire it.
      surface.annotationHovered = -1;
      surface.annotationHoverRevision += 1;
    }
    if (changed) invalidate_selection_cursor_rects(surface);
    // Nothing is drawn here. A layout draws once every base rect belongs to
    // the same scene, in `finish_layout`; a gesture sample is followed by the
    // manager's own present, which encodes the pixels and this OSC into one
    // command buffer.
  });
}

void screenwide_preview_surface_set_annotation_hover_callback(
    void *handle, screenwide_preview_annotation_hover_callback callback,
    void *context) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.annotationHoverCallback = callback;
    surface.annotationHoverContext = context;
  });
}

void screenwide_preview_surface_set_annotation_gesture_callback(
    void *handle, screenwide_preview_annotation_gesture_callback callback,
    void *context) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.annotationGestureCallback = callback;
    surface.annotationGestureContext = context;
  });
}
