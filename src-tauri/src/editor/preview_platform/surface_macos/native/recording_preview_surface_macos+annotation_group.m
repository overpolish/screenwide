// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Several annotations chosen together: the boxes drawn round them, and where
//! a press lands on the group. Rust decides which annotations are chosen and
//! where their boxes fall; this side only draws the boxes and hit-tests them.

#import "../../../../osc/gpu/macos/osc_gpu_macos.h"
#import "recording_preview_surface_macos_private.h"
#include "recording_preview_annotation_layers_macos.h"

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

/// The most vertices the group chrome takes from the OSC's buffer, leaving
/// the snap chrome drawn after it room for its guides, bars and anchor.
static const NSUInteger kAnnotationGroupVertexBudget = 1792;
/// One selection frame: a halo and a core, four quads of six vertices each.
static const NSUInteger kAnnotationGroupFrameVertices = 48;

static const ScreenwideAnnotationGroupBox *annotation_group_boxes(
    ScreenwidePreviewSurface *surface, NSUInteger *count) {
  *count = surface.annotationGroupBoxes.length / sizeof(ScreenwideAnnotationGroupBox);
  return *count == 0 ? NULL : surface.annotationGroupBoxes.bytes;
}

/// A box on screen, in this flipped view's coordinates, or the zero rect
/// where its layer is not laid out.
static NSRect annotation_group_rect(ScreenwidePreviewSurface *surface,
                                    ScreenwideAnnotationGroupBox box) {
  NSRect image = annotation_layer_image(surface, box.layer_id);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return NSZeroRect;
  return NSMakeRect(NSMinX(image) + image.size.width * box.left,
                    NSMinY(image) + image.size.height * box.top,
                    image.size.width * (box.right - box.left),
                    image.size.height * (box.bottom - box.top));
}

void screenwide_preview_surface_set_annotation_group(
    void *handle, const ScreenwideAnnotationGroupBox *boxes, size_t count) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  // The block outlives this call, so the boxes are copied while they live.
  NSData *data = [NSData dataWithBytes:(count == 0 ? NULL : boxes)
                                length:(NSUInteger)count * sizeof(ScreenwideAnnotationGroupBox)];
  on_main_async(^{
    surface.annotationGroupBoxes = data;
  });
}

SCREENWIDE_PREVIEW_PRIVATE BOOL annotation_has_group(ScreenwidePreviewSurface *surface) {
  return surface.annotationGroupBoxes.length > 0 &&
         annotation_active_mode(surface) != ScreenwideAnnotationModeNone;
}

SCREENWIDE_PREVIEW_PRIVATE int32_t annotation_group_layer_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  if (!annotation_has_group(surface)) return INT32_MIN;
  NSUInteger count = 0;
  const ScreenwideAnnotationGroupBox *boxes = annotation_group_boxes(surface, &count);
  for (NSUInteger index = 0; index < count; index++) {
    if (boxes[index].kind != ScreenwideAnnotationGroupBoxWhole) continue;
    NSRect rect = annotation_group_rect(surface, boxes[index]);
    if (point.x >= NSMinX(rect) && point.x <= NSMaxX(rect) && point.y >= NSMinY(rect) &&
        point.y <= NSMaxY(rect) && rect.size.width + rect.size.height > 0.0)
      return boxes[index].layer_id;
  }
  return INT32_MIN;
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_group_add_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  if (!annotation_has_group(surface)) return;
  NSUInteger total = 0;
  const ScreenwideAnnotationGroupBox *boxes = annotation_group_boxes(surface, &total);
  // The whole group's box first, so a crowd of members can never crowd it
  // out of the vertex budget.
  for (uint32_t pass = 0; pass < 2; pass++) {
    uint32_t kind = pass == 0 ? ScreenwideAnnotationGroupBoxWhole : ScreenwideAnnotationGroupBoxMember;
    for (NSUInteger index = 0; index < total; index++) {
      if (boxes[index].kind != kind) continue;
      if (*count + kAnnotationGroupFrameVertices > kAnnotationGroupVertexBudget) return;
      NSRect rect = annotation_group_rect(surface, boxes[index]);
      if (rect.size.width <= 0.0 && rect.size.height <= 0.0) continue;
      // The layer selection's own frame without its handles: it reads over
      // any picture, and says "chosen" the way every other OSC does.
      screenwide_region_osc_add_selection_frame(vertices, count, size, rect, scale);
    }
  }
}

SCREENWIDE_PREVIEW_PRIVATE int32_t annotation_marquee_layer_at_point(
    ScreenwidePreviewSurface *surface, NSPoint point) {
  NSMutableArray<NSValue *> *layers = [NSMutableArray array];
  if (surface.hasSelection) {
    ScreenwidePreviewSelection selected = surface.selection;
    [layers addObject:[NSValue valueWithBytes:&selected objCType:@encode(ScreenwidePreviewSelection)]];
  }
  if (surface.selectionTargets != nil) [layers addObjectsFromArray:surface.selectionTargets];
  for (NSValue *value in layers) {
    ScreenwidePreviewSelection layer;
    [value getValue:&layer size:sizeof(layer)];
    NSRect image = selection_image_frame_for(surface, layer);
    if (point.x >= NSMinX(image) && point.x <= NSMaxX(image) && point.y >= NSMinY(image) &&
        point.y <= NSMaxY(image))
      return (int32_t)layer.layer_id;
  }
  // A band pressed beside every picture still belongs to the one in hand.
  return surface.hasSelection ? (int32_t)surface.selection.layer_id : INT32_MIN;
}

SCREENWIDE_PREVIEW_PRIVATE void annotation_marquee_add_osc(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    ScreenwidePreviewSurface *surface, CGFloat scale) {
  NSRect band = surface.annotationMarquee;
  if (band.size.width <= 0.0 && band.size.height <= 0.0) return;
  // The region selector's own marching ants, unfilled. The band is its own
  // image, so no shade is drawn round it, and it has no handles to offer.
  screenwide_region_osc_add_crop_with_handles(vertices, count, size, band, band, scale, 0.0, YES,
                                              NO);
}
