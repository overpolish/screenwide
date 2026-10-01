// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_TYPES @R"METAL(
#include <metal_stdlib>
using namespace metal;

struct GpuCursor {
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
  uint style;
  uint clip_at_video_edge;
  uint visible;
  float opacity;
};

struct CursorArtwork {
  uint width;
  uint height;
  float design_width;
  float design_height;
  float origin_x;
  float origin_y;
  uint use_design;
  uint clip_local_box;
  uint supersample;
};

struct OverlayUniforms {
  int x;
  int y;
  uint cursor_width;
  uint cursor_height;
  uint output_width;
  uint output_height;
  int crop_x;
  int crop_y;
  uint crop_width;
  uint crop_height;
  uint crop_radius;
  uint clip_at_video_edge;
  struct GpuCursor cursor;
  struct CursorArtwork artwork;
};

struct CameraUniforms {
  uint crop_x;
  uint crop_y;
  uint crop_width;
  uint crop_height;
  int frame_x;
  int frame_y;
  uint frame_width;
  uint frame_height;
  uint radius;
  uint source_width;
  uint source_height;
  uint drop_shadow;
};
struct CanvasUniforms {
  packed_float4 background_color, recenter_inset_color;
  uint background_radius;
  int crop_x, crop_y;
  uint crop_width, crop_height;
  float image_x, image_y;
  uint image_width, image_height;
  int source_crop_x, source_crop_y;
  uint source_crop_width, source_crop_height;
  uint radius;
  uint drop_shadow;
  uint mesh_enabled, mesh_seed;
  float mesh_warp_percent;
  uint mesh_point_count, mesh_generator, mesh_generator_color_count;
  packed_float4 mesh_points[8];
  packed_float4 mesh_colors[5];
  uint clip_cursor_at_video_edge;
  uint transparent_background;
  uint foreground_only;
  uint has_background_image;
  uint background_image_id;
  float mesh_generator_speed;
  uint crop_preview;
  float crop_preview_x, crop_preview_y;
  float crop_preview_width, crop_preview_height;
  float crop_preview_radius;
  uint crop_preview_drop_shadow;
};
struct StillOverlayUniforms {
  int cursor_x;
  int cursor_y;
  uint cursor_width;
  uint cursor_height;
  uint cursor_source_width;
  uint cursor_source_height;
  uint camera_crop_x;
  uint camera_crop_y;
  uint camera_crop_width;
  uint camera_crop_height;
  int camera_frame_x;
  int camera_frame_y;
  uint camera_frame_width;
  uint camera_frame_height;
  uint camera_radius;
  uint camera_source_width;
  uint camera_source_height;
  uint camera_drop_shadow;
  uint camera_on_top;
};
)METAL"
