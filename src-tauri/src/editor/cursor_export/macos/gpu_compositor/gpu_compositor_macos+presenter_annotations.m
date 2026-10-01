// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "gpu_compositor_macos_presenter.h"
#import "gpu_compositor_macos_presenter_private.h"

// The retained workspace keeps its own copy of every layer's annotations.
// Layers arrive as views of lists Rust owns only for the call, and the
// presenter redraws on pan and zoom without Rust, so each list is copied into
// a store the presenter holds for as long as a retained layer points into it.

ScreenwideAnnotations screenwide_presenter_retain_annotations(
    ScreenwideStillPresenter *presenter, const ScreenwideAnnotations *source) {
  ScreenwideAnnotations retained = {0};
  if (source == NULL || source->count == 0 || source->items == NULL) return retained;
  size_t items = (size_t)source->count * sizeof(ScreenwideAnnotation);
  size_t points = source->data.points == NULL ? 0 : (size_t)source->data.point_count * sizeof(float[2]);
  size_t text = source->data.text == NULL ? 0 : (size_t)source->data.text_len;
  NSMutableData *store = [NSMutableData dataWithLength:items + points + text];
  uint8_t *bytes = store.mutableBytes;
  memcpy(bytes, source->items, items);
  if (points > 0) memcpy(bytes + items, source->data.points, points);
  if (text > 0) memcpy(bytes + items + points, source->data.text, text);
  retained.items = (const ScreenwideAnnotation *)bytes;
  retained.count = source->count;
  if (points > 0) {
    retained.data.points = (const float(*)[2])(bytes + items);
    retained.data.point_count = source->data.point_count;
  }
  if (text > 0) {
    retained.data.text = bytes + items + points;
    retained.data.text_len = source->data.text_len;
  }
  [presenter.workspaceAnnotationStores addObject:store];
  return retained;
}

void screenwide_presenter_prune_annotation_stores(ScreenwideStillPresenter *presenter) {
  // A store is live while the scene or the resize snapshot still points into
  // it: cancelling a resize puts the snapshot's layers back.
  NSMutableSet<NSValue *> *live = [NSMutableSet set];
  for (NSArray<NSValue *> *layers in @[
         presenter.workspaceLayers ?: @[], presenter.workspaceResizeLayers ?: @[]
       ]) {
    for (NSValue *value in layers) {
      ScreenwideWorkspaceLayer layer;
      [value getValue:&layer size:sizeof(layer)];
      if (layer.annotations.items != NULL)
        [live addObject:[NSValue valueWithPointer:layer.annotations.items]];
    }
  }
  NSMutableArray<NSMutableData *> *kept = [NSMutableArray array];
  for (NSMutableData *store in presenter.workspaceAnnotationStores)
    if ([live containsObject:[NSValue valueWithPointer:store.bytes]]) [kept addObject:store];
  presenter.workspaceAnnotationStores = kept;
}

/// Moves the hover halo on the retained workspace: the annotation at `index` in
/// `pane_index`'s own list wears it, and every other annotation in the scene
/// puts it down. A negative index only clears.
///
/// The halo is the one piece of annotation state the pointer changes without
/// the document changing, so it is set here and redrawn rather than sent back
/// round through a fresh composition.
int screenwide_gpu_still_presenter_set_workspace_annotation_hover(
    void *handle, uint32_t pane_index, int32_t index, float width) {
  if (handle == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  int found = 0;
  for (NSValue *value in presenter.workspaceLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == pane_index) found = 1;
    // The list is the presenter's own store, so the halo is set on it in place.
    ScreenwideAnnotation *items = (ScreenwideAnnotation *)layer.annotations.items;
    for (uint32_t annotation = 0; annotation < layer.annotations.count; ++annotation)
      items[annotation].hover = layer.pane_index == pane_index && index >= 0 &&
                                        (uint32_t)index == annotation
                                    ? width
                                    : 0;
  }
  return found;
}
