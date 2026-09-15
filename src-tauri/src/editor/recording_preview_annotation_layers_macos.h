// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

// A mark belongs to its image, independently of the currently selected layer.
static inline BOOL annotation_layer_selection(ScreenwidePreviewSurface *surface,
    int32_t layer, ScreenwidePreviewSelection *selection) {
  if (surface.hasSelection && (layer < 0 || surface.selection.layer_id == (uint32_t)layer)) {
    *selection = surface.selection;
    return YES;
  }
  for (NSValue *value in surface.selectionTargets) {
    ScreenwidePreviewSelection target;
    [value getValue:&target size:sizeof(target)];
    if (target.layer_id == (uint32_t)layer) {
      *selection = target;
      return YES;
    }
  }
  return NO;
}

static inline NSRect annotation_layer_image(ScreenwidePreviewSurface *surface, int32_t layer) {
  ScreenwidePreviewSelection target;
  return annotation_layer_selection(surface, layer, &target)
    ? selection_image_frame_for(surface, target) : NSZeroRect;
}
