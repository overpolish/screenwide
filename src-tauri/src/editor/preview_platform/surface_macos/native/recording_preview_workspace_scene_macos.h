// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

#import "../../../../osc/gpu/macos/osc_gpu_macos.h"

// The workspace's retained scene lives in Rust (`workspace_scene`), which
// draws it with the shared wgpu compositor. The surface keeps the layer, the
// input and the present, and reaches the scene through these entries.

/// A canvas' display rectangle in drawable pixels, from the top left.
typedef struct {
  int32_t x;
  int32_t y;
  uint32_t width;
  uint32_t height;
} ScreenwideWorkspacePlacement;

typedef struct {
  uint32_t index;
  double x;
  double y;
  double width;
  double height;
} ScreenwideWorkspacePaneRect;

/// Releases the scene reference the surface was created with.
void screenwide_workspace_scene_release(const void *scene);
/// Whether the scene holds `count` layers, one for each placement a redraw
/// would pass. A redraw that cannot draw must not take a drawable.
int screenwide_workspace_scene_ready(const void *scene, uint32_t count);
/// Draws the scene into `texture`, the drawable's `id<MTLTexture>`, on the
/// shared queue: one placement per layer, and the crop magnifier if active.
int screenwide_workspace_scene_draw(const void *scene, void *texture,
                                    const ScreenwideWorkspacePlacement *placements,
                                    uint32_t count,
                                    const ScreenwideRegionMagnifier *magnifier);

/// Native Frame gestures transform the retained scene directly so the media
/// and the OSC show the same pointer sample before React mirrors the update.
/// Each update is measured from the scene as `begin_resize` found it.
int screenwide_workspace_scene_begin_resize(const void *scene);
void screenwide_workspace_scene_end_resize(const void *scene, int commit);
int screenwide_workspace_scene_update_resize(const void *scene, double origin_x_ratio,
                                             double origin_y_ratio, double width_ratio,
                                             double height_ratio);
int screenwide_workspace_scene_update_auto_fit_move(
    const void *scene, uint32_t selected_layer, double move_x_ratio, double move_y_ratio,
    double origin_x_ratio, double origin_y_ratio, double width_ratio, double height_ratio);
int screenwide_workspace_scene_update_recording_auto_fit_move(
    const void *scene, uint32_t selected_pane, double move_x_ratio, double move_y_ratio,
    double origin_x_ratio, double origin_y_ratio, double width_ratio, double height_ratio);
/// Resizes only the selected recording pane's canvas.
int screenwide_workspace_scene_update_selected_resize(
    const void *scene, uint32_t selected_pane, double origin_x_ratio, double origin_y_ratio,
    double width_ratio, double height_ratio);
/// Live corner radius of the selected pane: `frame` non-zero rounds its
/// canvas, zero the clip inside it.
int screenwide_workspace_scene_update_selected_radius(const void *scene,
                                                      uint32_t selected_pane,
                                                      double radius_percent, int frame);
/// Places the pane's shortcut strip at a normalised centre and scales it from
/// where the gesture found it.
int screenwide_workspace_scene_update_keyboard(const void *scene, uint32_t pane_index,
                                               double center_x, double center_y,
                                               double scale_ratio);
/// Where the pane's shortcut strip shows, as shares of its canvas.
int screenwide_workspace_scene_keyboard_bounds(const void *scene, uint32_t pane_index,
                                               double *x, double *y, double *width,
                                               double *height);

/// Copies straight RGBA into a BGRA drawable's `id<MTLTexture>` of its size,
/// on the shared queue.
int screenwide_preview_upload_rgba(void *texture, const uint8_t *rgba, uint32_t width,
                                   uint32_t height);
