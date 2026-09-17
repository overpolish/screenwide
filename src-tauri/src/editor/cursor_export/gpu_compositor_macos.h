// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stddef.h>
#include <stdint.h>

#import <CoreMedia/CoreMedia.h>

#import "gpu_compositor_macos_annotation_types.h"
#import "gpu_compositor_macos_keyboard_types.h"
#import "../osc_gpu_macos.h"

typedef struct {
  float background_color[4];
  float recenter_inset_color[4];
  uint32_t background_radius;
  int32_t crop_x;
  int32_t crop_y;
  uint32_t crop_width;
  uint32_t crop_height;
  float image_x;
  float image_y;
  uint32_t image_width;
  uint32_t image_height;
  int32_t source_crop_x;
  int32_t source_crop_y;
  uint32_t source_crop_width;
  uint32_t source_crop_height;
  uint32_t radius;
  uint32_t drop_shadow;
  uint32_t mesh_enabled;
  uint32_t mesh_seed;
  float mesh_warp_percent;
  uint32_t mesh_point_count;
  /// Which picture the mesh background paints. Zero is the app's own blob
  /// mesh, which alone reads the points and the warp; every other id names a
  /// generator that reads only `mesh_colors` and `mesh_seed`.
  uint32_t mesh_generator;
  uint32_t mesh_generator_color_count;
  float mesh_points[4][8];
  float mesh_colors[5][4];
  uint32_t clip_cursor_at_video_edge;
  uint32_t transparent_background;
  uint32_t foreground_only;
  /// Non-zero when `background_image_id` names a picture registered through
  /// `screenwide_gpu_register_background_image`. Zero paints the mesh or the
  /// solid colour, which is also what an unregistered id falls back to.
  uint32_t has_background_image;
  uint32_t background_image_id;
  /// What the canvas seconds are multiplied by before a ported generator
  /// reads them, from the table in `screenshots/mesh_generator.rs`. The
  /// classic mesh is 1.0 and drifts with the seconds as they come.
  float mesh_generator_speed;
  /// Non-zero while the crop tool previews the whole uncropped source. The
  /// fields below are the cropped layer, drawn a second time over that ghost
  /// with the rounding and shadow the ghost gives up.
  uint32_t crop_preview;
  float crop_preview_x;
  float crop_preview_y;
  float crop_preview_width;
  float crop_preview_height;
  float crop_preview_radius;
  uint32_t crop_preview_drop_shadow;
} ScreenwideCanvas;
// The same fields in the same order as Rust's `NativeCanvas` and Metal's
// `CanvasUniforms`. Every one is four bytes wide, so the generator ids landing
// between `mesh_point_count` and `mesh_points` move all three or none.
_Static_assert(sizeof(ScreenwideCanvas) == 376,
               "ScreenwideCanvas ABI must match Rust");
_Static_assert(offsetof(ScreenwideCanvas, mesh_generator) == 108,
               "ScreenwideCanvas.mesh_generator ABI must match Rust");
_Static_assert(offsetof(ScreenwideCanvas, mesh_points) == 116,
               "ScreenwideCanvas.mesh_points ABI must match Rust");
_Static_assert(offsetof(ScreenwideCanvas, mesh_generator_speed) == 344,
               "ScreenwideCanvas.mesh_generator_speed ABI must match Rust");
_Static_assert(offsetof(ScreenwideCanvas, crop_preview) == 348,
               "ScreenwideCanvas.crop_preview ABI must match Rust");

/// One output frame's cursor, evaluated from the recorded event timeline.
/// Positions and sizes are canvas pixels; the compositor's shader owns every
/// pixel of the drawn cursor, including its motion blur and click animation.
typedef struct {
  float blur_delta_x;
  float blur_delta_y;
  float height;
  float hotspot_x;
  float hotspot_y;
  float rotation_radians;
  float scale;
  float width;
  float x;
  float y;
  uint32_t style;
  uint32_t clip_at_video_edge;
  uint32_t visible;
  float opacity;
} ScreenwideGpuCursor;

/// One cursor style's artwork. `pixels` is tightly packed RGBA owned by the
/// caller and is only read while the compositor uploads its textures.
/// System artwork stretches over the recorded cursor box (`use_design` is
/// zero); vector fallback artwork keeps its design aspect inside that box.
typedef struct {
  const uint8_t *pixels;
  uint32_t width;
  uint32_t height;
  float design_width;
  float design_height;
  float origin_x;
  float origin_y;
  uint32_t use_design;
  uint32_t clip_local_box;
  uint32_t supersample;
} ScreenwideCursorArtwork;

typedef struct {
  uint32_t width;
  uint32_t height;
  float design_width;
  float design_height;
  float origin_x;
  float origin_y;
  uint32_t use_design;
  uint32_t clip_local_box;
  uint32_t supersample;
} ScreenwideCursorArtworkUniforms;

typedef struct {
  int32_t x;
  int32_t y;
  uint32_t cursor_width;
  uint32_t cursor_height;
  uint32_t output_width;
  uint32_t output_height;
  int32_t crop_x;
  int32_t crop_y;
  uint32_t crop_width;
  uint32_t crop_height;
  uint32_t crop_radius;
  uint32_t clip_at_video_edge;
  ScreenwideGpuCursor cursor;
  ScreenwideCursorArtworkUniforms artwork;
} ScreenwideOverlayUniforms;

typedef struct {
  int32_t cursor_x;
  int32_t cursor_y;
  uint32_t cursor_width;
  uint32_t cursor_height;
  uint32_t cursor_source_width;
  uint32_t cursor_source_height;
  uint32_t camera_crop_x;
  uint32_t camera_crop_y;
  uint32_t camera_crop_width;
  uint32_t camera_crop_height;
  int32_t camera_frame_x;
  int32_t camera_frame_y;
  uint32_t camera_frame_width;
  uint32_t camera_frame_height;
  uint32_t camera_radius;
  uint32_t camera_source_width;
  uint32_t camera_source_height;
  uint32_t camera_drop_shadow;
  uint32_t camera_on_top;
} ScreenwideStillOverlay;
