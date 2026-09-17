// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#include <math.h>

#import "gpu_compositor_macos_presenter.h"
#import "gpu_compositor_macos_presenter_private.h"

/// Moves everything attached to a layer's clip by (dx, dy) canvas pixels when
/// a Frame gesture moves the canvas origin: the crop and image placement, the
/// visible source region the shader gates clip coverage on, and both cursor
/// representations. The workspace kernel draws `cursor`, not
/// `overlay.cursor_*`, so the recorded pointer must travel with the clip too.
static void shift_layer_content(ScreenwideWorkspaceLayer *layer, double dx,
                                double dy) {
  layer->canvas.crop_x = (int32_t)llround(layer->canvas.crop_x + dx);
  layer->canvas.crop_y = (int32_t)llround(layer->canvas.crop_y + dy);
  layer->canvas.image_x += (float)dx;
  layer->canvas.image_y += (float)dy;
  layer->canvas.source_crop_x += (int32_t)llround(dx);
  layer->canvas.source_crop_y += (int32_t)llround(dy);
  if (layer->overlay.cursor_width > 0) {
    layer->overlay.cursor_x += (int32_t)llround(dx);
    layer->overlay.cursor_y += (int32_t)llround(dy);
  }
  layer->cursor.x += (float)dx;
  layer->cursor.y += (float)dy;
}

int screenwide_gpu_still_presenter_begin_workspace_resize(void *handle) {
  if (handle == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  presenter.workspaceResizeLayers = [presenter.workspaceLayers copy];
  presenter.workspaceResizeApplied = NO;
  return presenter.workspaceResizeLayers.count > 0;
}

static int update_workspace_resize(
    void *handle, uint32_t selected_layer, double move_x_ratio,
    double move_y_ratio, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  if (handle == NULL || !isfinite(origin_x_ratio) ||
      !isfinite(origin_y_ratio) || !isfinite(width_ratio) ||
      !isfinite(height_ratio) || width_ratio <= 0.0 ||
      height_ratio <= 0.0) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  if (presenter.workspaceResizeLayers.count == 0) return 0;
  NSMutableArray<NSValue *> *resized =
      [NSMutableArray arrayWithCapacity:presenter.workspaceResizeLayers.count];
  NSUInteger index = 0;
  for (NSValue *value in presenter.workspaceResizeLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    double old_width = MAX(layer.canvas_width, 1u);
    double old_height = MAX(layer.canvas_height, 1u);
    uint32_t next_width = (uint32_t)MAX(llround(old_width * width_ratio), 1);
    uint32_t next_height = (uint32_t)MAX(llround(old_height * height_ratio), 1);
    double origin_x = old_width * origin_x_ratio;
    double origin_y = old_height * origin_y_ratio;
    double move_x = index == selected_layer ? old_width * move_x_ratio : 0.0;
    double move_y = index == selected_layer ? old_height * move_y_ratio : 0.0;
    double old_shortest = MAX(MIN(old_width, old_height), 1.0);
    double next_shortest = MIN(next_width, next_height);
    layer.canvas_width = next_width;
    layer.canvas_height = next_height;
    shift_layer_content(&layer, move_x - origin_x, move_y - origin_y);
    layer.canvas.background_radius = (uint32_t)MAX(
        llround(layer.canvas.background_radius * next_shortest / old_shortest), 0);
    [resized addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
    index += 1;
  }
  presenter.workspaceResizeApplied = YES;
  presenter.workspaceLayers = resized;
  return 1;
}

int screenwide_gpu_still_presenter_update_workspace_resize(
    void *handle, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  return update_workspace_resize(
      handle, UINT32_MAX, 0.0, 0.0, origin_x_ratio, origin_y_ratio,
      width_ratio, height_ratio);
}

int screenwide_gpu_still_presenter_update_workspace_auto_fit_move(
    void *handle, uint32_t selected_layer, double move_x_ratio,
    double move_y_ratio, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  return update_workspace_resize(
      handle, selected_layer, move_x_ratio, move_y_ratio,
      origin_x_ratio, origin_y_ratio, width_ratio, height_ratio);
}

int screenwide_gpu_still_presenter_update_recording_auto_fit_move(
    void *handle, uint32_t selected_pane, double move_x_ratio,
    double move_y_ratio, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio) {
  if (handle == NULL || !isfinite(origin_x_ratio) ||
      !isfinite(origin_y_ratio) || !isfinite(width_ratio) ||
      !isfinite(height_ratio) || width_ratio <= 0.0 || height_ratio <= 0.0)
    return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  if (presenter.workspaceResizeLayers.count == 0) return 0;
  BOOL bakedCamera = selected_pane == 1 && presenter.workspaceResizeLayers.count == 1;
  NSMutableArray<NSValue *> *resized =
      [NSMutableArray arrayWithCapacity:presenter.workspaceResizeLayers.count];
  BOOL found = NO;
  for (NSValue *value in presenter.workspaceResizeLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == selected_pane || bakedCamera) {
      found = YES;
      double oldWidth = MAX(layer.canvas_width, 1u);
      double oldHeight = MAX(layer.canvas_height, 1u);
      double originX = oldWidth * origin_x_ratio;
      double originY = oldHeight * origin_y_ratio;
      double moveX = oldWidth * move_x_ratio;
      double moveY = oldHeight * move_y_ratio;
      layer.canvas_width = (uint32_t)MAX(llround(oldWidth * width_ratio), 1);
      layer.canvas_height = (uint32_t)MAX(llround(oldHeight * height_ratio), 1);
      if (bakedCamera) {
        // The selected camera moves independently, while the screen content
        // (and the cursor attached to it) only follows the canvas origin.
        shift_layer_content(&layer, -originX, -originY);
        layer.overlay.camera_frame_x += (int32_t)llround(moveX - originX);
        layer.overlay.camera_frame_y += (int32_t)llround(moveY - originY);
      } else {
        shift_layer_content(&layer, moveX - originX, moveY - originY);
      }
      layer.placement.width =
          (uint32_t)MAX(llround(layer.placement.width * width_ratio), 1);
      layer.placement.height =
          (uint32_t)MAX(llround(layer.placement.height * height_ratio), 1);
    }
    [resized addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
  }
  if (!found) return 0;
  presenter.workspaceResizeApplied = YES;
  presenter.workspaceLayers = resized;
  return 1;
}

int screenwide_gpu_still_presenter_update_workspace_selected_resize(
    void *handle, uint32_t selected_layer, double origin_x_ratio,
    double origin_y_ratio, double width_ratio, double height_ratio) {
  if (handle == NULL || !isfinite(origin_x_ratio) ||
      !isfinite(origin_y_ratio) || !isfinite(width_ratio) ||
      !isfinite(height_ratio) || width_ratio <= 0.0 || height_ratio <= 0.0)
    return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  NSArray<NSValue *> *base = presenter.workspaceResizeLayers.count > 0
      ? presenter.workspaceResizeLayers
      : presenter.workspaceLayers;
  NSMutableArray<NSValue *> *resized =
      [NSMutableArray arrayWithCapacity:base.count];
  BOOL found = NO;
  for (NSValue *value in base) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == selected_layer) {
      found = YES;
      double old_width = MAX(layer.canvas_width, 1u);
      double old_height = MAX(layer.canvas_height, 1u);
      uint32_t next_width = (uint32_t)MAX(llround(old_width * width_ratio), 1);
      uint32_t next_height = (uint32_t)MAX(llround(old_height * height_ratio), 1);
      double old_shortest = MAX(MIN(old_width, old_height), 1.0);
      double next_shortest = MIN(next_width, next_height);
      double origin_x = old_width * origin_x_ratio;
      double origin_y = old_height * origin_y_ratio;
      shift_layer_content(&layer, -origin_x, -origin_y);
      // A baked camera is another layer in this canvas. Keep its absolute
      // canvas position stable when a Frame gesture moves the canvas origin,
      // matching the shared semantic rebase used at gesture commit.
      if (layer.overlay.camera_frame_width > 0) {
        layer.overlay.camera_frame_x -= (int32_t)llround(origin_x);
        layer.overlay.camera_frame_y -= (int32_t)llround(origin_y);
      }
      layer.canvas_width = next_width;
      layer.canvas_height = next_height;
      layer.canvas.background_radius = (uint32_t)MAX(
          llround(layer.canvas.background_radius * next_shortest / old_shortest), 0);
      layer.placement.width = (uint32_t)MAX(llround(layer.placement.width * width_ratio), 1);
      layer.placement.height = (uint32_t)MAX(llround(layer.placement.height * height_ratio), 1);
    }
    [resized addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
  }
  if (!found) return 0;
  presenter.workspaceResizeApplied = YES;
  presenter.workspaceLayers = resized;
  return 1;
}

int screenwide_gpu_still_presenter_update_workspace_selected_radius(
    void *handle, uint32_t selected_layer, double radius_percent, int frame) {
  if (handle == NULL || !isfinite(radius_percent)) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  NSMutableArray<NSValue *> *updated =
      [NSMutableArray arrayWithCapacity:presenter.workspaceLayers.count];
  BOOL found = NO;
  double percent = fmin(50.0, fmax(0.0, radius_percent));
  for (NSValue *value in presenter.workspaceLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == selected_layer) {
      found = YES;
      // Same pixel derivation as the export path (platform_macos.rs): the
      // frame radius is a share of the canvas's shorter side, the clip radius
      // a share of the crop's shorter side.
      if (frame != 0) {
        double shortest = MIN(layer.canvas_width, layer.canvas_height);
        layer.canvas.background_radius =
            (uint32_t)MAX(llround(shortest * percent / 100.0), 0);
      } else {
        double shortest = MIN(layer.canvas.crop_width, layer.canvas.crop_height);
        layer.canvas.radius =
            (uint32_t)MAX(llround(shortest * percent / 100.0), 0);
      }
    }
    [updated addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
  }
  if (!found) return 0;
  presenter.workspaceLayers = updated;
  return 1;
}

void screenwide_gpu_still_presenter_end_workspace_resize(
    void *handle, int commit) {
  if (handle == NULL) return;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  if (commit == 0 && presenter.workspaceResizeLayers.count > 0)
    presenter.workspaceLayers = [presenter.workspaceResizeLayers mutableCopy];
  presenter.workspaceResizeLayers = nil;
  presenter.workspaceResizeApplied = NO;
}
