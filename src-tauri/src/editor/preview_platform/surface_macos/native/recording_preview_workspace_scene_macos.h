// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

#import "../../../../../crates/osc/src/gpu/macos/osc_gpu_macos.h"

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
/// Draws a caret and selection into the text box at `index` in layer
/// `layer`'s list, from byte `start` to byte `end` of its UTF-8 text, the
/// caret shown where `caret` is non-zero; with `typing` zero, into no box.
/// Non-zero where that changes what is drawn.
int screenwide_workspace_scene_set_typing(const void *scene, int typing, uint32_t layer,
                                          uint32_t index, uint32_t start, uint32_t end,
                                          int caret);
/// Draws the scene into `texture`, a BGRA texture the drawable's size, on the
/// shared queue: one placement per layer, over a checkerboard sized for
/// `scale` drawable pixels a point and coloured for the light appearance
/// where `light` is non-zero, and the crop magnifier if active. Non-zero
/// `reuse` shows the picture last composed again where the scene has not
/// changed since, for a redraw of the selection chrome alone.
int screenwide_workspace_scene_draw(const void *scene, void *texture,
                                    const ScreenwideWorkspacePlacement *placements,
                                    uint32_t count,
                                    const ScreenwideRegionMagnifier *magnifier, double scale,
                                    int light, int reuse);

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
/// Where the pane's shortcut strip shows, as shares of its canvas.
int screenwide_workspace_scene_keyboard_bounds(const void *scene, uint32_t pane_index,
                                               double *x, double *y, double *width,
                                               double *height);
/// Where the pane's caption shows, as shares of its canvas.
int screenwide_workspace_scene_caption_bounds(const void *scene, uint32_t pane_index,
                                              double *x, double *y, double *width,
                                              double *height);
/// Where the caption being typed into on the pane sets its words: its block
/// as x, y, width and height shares of its canvas in `block`, its type size
/// as a share of the block's height, and how its lines line up.
int screenwide_workspace_scene_caption_editing_block(const void *scene, uint32_t pane_index,
                                                     double *block, double *size, double *align);
/// The word of the pane's caption under the point, as shares of its canvas,
/// counted across its lines; -1 over none.
int32_t screenwide_workspace_scene_caption_word_at(const void *scene, uint32_t pane_index,
                                                   double x, double y);

/// Copies straight RGBA into a BGRA drawable's `id<MTLTexture>` of its size,
/// on the shared queue.
int screenwide_preview_upload_rgba(void *texture, const uint8_t *rgba, uint32_t width,
                                   uint32_t height);
