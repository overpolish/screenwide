// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_WORKSPACE_KERNELS @R"METAL(struct WorkspacePlacement {
  int x;
  int y;
  uint width;
  uint height;
};

struct WorkspaceMagnifier {
  uint active;
  uint pane_index;
  uint layer_id;
  uint sample_camera;
  uint edges;
  uint light_mode;
  float sample_u;
  float sample_v;
  float source_min_u, source_min_v, source_max_u, source_max_v;
  int box_x;
  int box_y;
  uint box_width;
  uint box_height;
};
// Separate kernels preserve foreground-over ordering in one command buffer.
kernel void workspace_clear(
    texture2d<float, access::write> output [[texture(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (any(gid >= uint2(output.get_width(), output.get_height()))) return;
  output.write(float4(0.0), gid);
}

kernel void workspace_layer(
    const device uchar4 *source [[buffer(0)]],
    texture2d<float, access::read_write> output [[texture(0)]],
    constant CanvasUniforms &u [[buffer(1)]],
    constant uint2 &source_dimensions [[buffer(2)]],
    constant WorkspacePlacement &placement [[buffer(3)]],
    constant uint &first_layer [[buffer(4)]],
    constant uint2 &logical_dimensions [[buffer(5)]],
    constant OverlayUniforms &cursor_uniforms [[buffer(6)]],
    const device uchar4 *camera [[buffer(7)]],
    constant StillOverlayUniforms &overlay [[buffer(8)]],
    constant float &seconds [[buffer(9)]],
    const device uchar4 *keyboard_pixels [[buffer(10)]],
    constant KeyboardUniforms &keyboard [[buffer(11)]],
    const device AnnotationUniforms *annotations [[buffer(12)]],
    constant uint &annotation_count [[buffer(13)]],
    const device AnnotationSample *annotation_samples [[buffer(15)]],
    const device uchar4 *annotation_numbers [[buffer(16)]],
    constant uint2 &annotation_atlas [[buffer(17)]],
    const device packed_float2 *annotation_points [[buffer(18)]],
    const device uchar *annotation_text [[buffer(19)]],
    texture2d_array<float, access::read> cursor_images [[texture(1)]],
    texture2d<float, access::sample> background_picture [[texture(2)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 dimensions(output.get_width(), output.get_height());
  if (any(gid >= dimensions) || placement.width == 0 || placement.height == 0)
    return;
  float2 global_point = float2(gid) + 0.5;
  float2 local = global_point - float2(placement.x, placement.y);
  if (any(local < 0.0) || local.x >= float(placement.width) ||
      local.y >= float(placement.height)) return;
  float2 canvas_dimensions = float2(logical_dimensions);
  float2 canvas_point = local / float2(placement.width, placement.height) *
                        canvas_dimensions;
  float canvas_coverage = rounded_coverage(
      canvas_point, canvas_dimensions, float(u.background_radius));
  float4 existing = output.read(gid);
  float4 rgba;
  if (first_layer != 0 || u.foreground_only == 0) {
    rgba = canvas_rgba_pixel(source, source_dimensions.x, source_dimensions.y,
                             canvas_point, canvas_dimensions, u, seconds,
                             background_picture);
  } else {
    rgba = overlay_canvas_foreground_rgba(
        existing, source, source_dimensions.x, source_dimensions.y,
        canvas_point, canvas_dimensions, u);
  }
  if (cursor_uniforms.cursor.visible != 0) {
    float blur = min(length(float2(cursor_uniforms.cursor.blur_delta_x, cursor_uniforms.cursor.blur_delta_y)), 80.0);
    float radius = length(float2(cursor_uniforms.cursor.width, cursor_uniforms.cursor.height)) * cursor_uniforms.cursor.scale + blur + 4.0;
    bool visible = all(abs(canvas_point - float2(cursor_uniforms.cursor.x,
                                                  cursor_uniforms.cursor.y)) <= radius);
    if (u.clip_cursor_at_video_edge != 0) {
      float2 crop_point = canvas_point - float2(u.crop_x, u.crop_y);
      float2 crop_size = float2(u.crop_width, u.crop_height);
      visible = visible && all(crop_point >= 0.0) && all(crop_point < crop_size) &&
        rounded_pixel_visible(crop_point, crop_size, float(u.radius));
    }
    if (visible) {
      float4 pixel = cursor_pixel(cursor_images, cursor_uniforms, canvas_point);
      rgba = mix(rgba, pixel, pixel.a);
    }
  }
  // The layer is drawn into `placement`, which is rarely its canvas' own size:
  // zoomed out, one drawn pixel covers several canvas pixels, and an edge
  // feathered over one canvas pixel would land inside a single drawn one.
  float2 annotation_pixel_steps =
      canvas_dimensions / float2(placement.width, placement.height);
  float annotation_pixel_scale =
      max(annotation_pixel_steps.x, annotation_pixel_steps.y);
  rgba = composite_annotations(rgba, annotations, annotation_count, 0u,
                               canvas_point, u, float2(source_dimensions),
                               annotation_pixel_scale, annotation_samples,
                               annotation_numbers, annotation_atlas);
  float2 camera_point = canvas_point - float2(overlay.camera_frame_x,
                                                overlay.camera_frame_y);
  float2 camera_size = float2(overlay.camera_frame_width,
                              overlay.camera_frame_height);
  float camera_coverage = overlay.camera_frame_width > 0
    ? rounded_coverage(camera_point, camera_size, float(overlay.camera_radius)) : 0.0;
  if (camera_coverage > 0.0) {
    float2 source_point = float2(overlay.camera_crop_x, overlay.camera_crop_y) +
      clamp(camera_point, float2(0.0), camera_size) / camera_size *
      float2(overlay.camera_crop_width, overlay.camera_crop_height);
    uint2 camera_source = min(uint2(source_point),
      uint2(overlay.camera_source_width - 1, overlay.camera_source_height - 1));
    float4 pixel = float4(camera[camera_source.y * overlay.camera_source_width +
                                 camera_source.x]) / 255.0;
    rgba = mix(rgba, pixel, camera_coverage * pixel.a);
  }
  if (overlay.camera_frame_width > 0 && overlay.camera_drop_shadow != 0) {
    float sigma = margin_capped_sigma(
      float2(overlay.camera_frame_x, overlay.camera_frame_y), camera_size,
      canvas_dimensions, shadow_sigma(camera_size));
    if (sigma > 1.0) {
      float shadow = soft_shadow(camera_point, camera_size,
        float(overlay.camera_radius), sigma, 0.14);
      rgba.rgb *= 1.0 - shadow * (1.0 - camera_coverage);
    }
  }
  if (overlay.camera_frame_width > 0 && overlay.camera_on_top == 0) {
    rgba = overlay_canvas_foreground_rgba(
      rgba, source, source_dimensions.x, source_dimensions.y,
      canvas_point, canvas_dimensions, u);
    // The redrawn foreground covers the pass above, so the annotations under
    // the camera go back over it exactly as the cursor does.
    rgba = composite_annotations(rgba, annotations, annotation_count, 0u,
                                 canvas_point, u, float2(source_dimensions),
                                 annotation_pixel_scale, annotation_samples,
                               annotation_numbers, annotation_atlas);
  }
  rgba = composite_keyboard(rgba, keyboard_pixels, keyboard, canvas_point,
                            canvas_dimensions);
  rgba = composite_annotations(rgba, annotations, annotation_count, 1u,
                               canvas_point, u, float2(source_dimensions),
                               annotation_pixel_scale, annotation_samples,
                               annotation_numbers, annotation_atlas);
  if (u.foreground_only == 0) rgba.rgb = output_dither(rgba.rgb, global_point);
  rgba.rgb *= canvas_coverage;
  rgba.a = u.foreground_only != 0 || u.transparent_background != 0
    ? rgba.a * canvas_coverage : 1.0;
  output.write(rgba, gid);
}


)METAL"
