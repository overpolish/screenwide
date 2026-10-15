// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_SELECTION_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_SELECTION_MACOS_H

#include <stddef.h>
#include <stdint.h>

// The layer selection as Rust hands it over, and the shared display-space
// hit test and fit it is measured with (`workspace_editor/ffi.rs`).

typedef struct {
  double x;
  double y;
  double width;
  double height;
} ScreenwideDisplayRect;

typedef struct {
  uint64_t id;
  ScreenwideDisplayRect rect;
  uint8_t radius_enabled;
  double radius_percent;
  int32_t z_order;
  uint8_t selected;
  uint8_t visible;
} ScreenwideDisplayTarget;

typedef struct {
  uint8_t found;
  uint64_t target_id;
  uint8_t handle;
} ScreenwideDisplayHit;

typedef struct {
  ScreenwideDisplayRect fit;
  double zoom;
  double pan_x;
  double pan_y;
} ScreenwideDisplayFitRebase;

typedef struct {
  double pan_x;
  double pan_y;
  double zoom;
} ScreenwideViewTransform;
extern ScreenwideViewTransform screenwide_workspace_panel_fit(
    double viewport_width, double viewport_height, double base_x, double base_y,
    double base_width, double base_height, double fit_width);
_Static_assert(sizeof(ScreenwideViewTransform) == 24,
               "Rust/C view transform layout mismatch");

extern ScreenwideDisplayHit screenwide_workspace_hit_test(
    const ScreenwideDisplayTarget *targets, size_t count, double x, double y,
    double handle_size);
extern ScreenwideDisplayFitRebase screenwide_workspace_rebase_display_fit(
    double viewport_width, double viewport_height, ScreenwideDisplayRect displayed, double natural_width,
    double natural_height, double gutter, uint8_t allow_upscale);
_Static_assert(sizeof(ScreenwideDisplayTarget) == 64,
               "Rust/C display target layout mismatch");
_Static_assert(sizeof(ScreenwideDisplayHit) == 24,
               "Rust/C display hit layout mismatch");
_Static_assert(sizeof(ScreenwideDisplayFitRebase) == 56,
               "Rust/C display fit rebase layout mismatch");

/// The twin of Rust's `PreviewSelection`.
typedef struct {
  uint32_t pane_index;
  uint32_t layer_id;
  uint32_t crop_mode, radius_disabled, framed;
  double x, y, width, height;
  double radius_percent;
  double image_x, image_y, image_width, image_height;
  double recenter_x, recenter_y, recenter_width, recenter_height;
  /// Non-zero where a scene turns the layer and the select tool turns it
  /// with the rotation gizmo; how it is turned now, in degrees.
  uint32_t turnable;
  double rotation_x, rotation_y, rotation_z;
} ScreenwidePreviewSelection;
_Static_assert(sizeof(ScreenwidePreviewSelection) == 160,
               "Rust/C preview selection layout mismatch");

#endif
