// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CANVAS_KERNELS @R"METAL(
kernel void compose_canvas_rgba(
    const device uchar4 *source [[buffer(0)]],
    device uchar4 *output [[buffer(1)]],
    constant CanvasUniforms &u [[buffer(2)]],
    constant uint2 &source_dimensions [[buffer(3)]],
    constant float &seconds [[buffer(4)]],
    constant OverlayUniforms &cursor [[buffer(5)]],
    const device uchar4 *camera [[buffer(6)]],
    constant StillOverlayUniforms &overlay [[buffer(7)]],
    const device uchar4 *keyboard_pixels [[buffer(10)]],
    constant KeyboardUniforms &keyboard [[buffer(11)]],
    const device AnnotationUniforms *annotations [[buffer(12)]],
    constant uint &annotation_count [[buffer(13)]],
    const device AnnotationSample *annotation_samples [[buffer(15)]],
    const device uchar4 *annotation_numbers [[buffer(16)]],
    constant uint2 &annotation_atlas [[buffer(17)]],
    texture2d_array<float, access::read> cursor_images [[texture(0)]],
    texture2d<float, access::sample> background_picture [[texture(1)]],
    uint2 gid [[thread_position_in_grid]],
    uint2 dimensions [[threads_per_grid]]) {
  if (any(gid >= dimensions)) return;
  float4 rgba = canvas_rgba_pixel(
    source, source_dimensions.x, source_dimensions.y, float2(gid) + 0.5,
    float2(dimensions), u, seconds, background_picture);
  float4 cursor_rgba = canvas_cursor_pixel(
    cursor_images, cursor, u, float2(gid) + 0.5);
  rgba = mix(rgba, cursor_rgba, cursor_rgba.a);
  // The still export dispatches one thread per output pixel over a canvas
  // measured in those same pixels, so a drawn pixel is a canvas pixel.
  const float annotation_pixel_scale = 1.0;
  rgba = composite_annotations(rgba, annotations, annotation_count, 0u,
                               float2(gid) + 0.5, u, float2(source_dimensions),
                               annotation_pixel_scale, annotation_samples,
                               annotation_numbers, annotation_atlas);
  float2 camera_point = float2(gid) -
    float2(overlay.camera_frame_x, overlay.camera_frame_y);
  float2 camera_size = float2(
    overlay.camera_frame_width, overlay.camera_frame_height);
  float camera_coverage = overlay.camera_frame_width > 0
    ? rounded_coverage(camera_point, camera_size, float(overlay.camera_radius))
    : 0.0;
  if (overlay.camera_frame_width > 0 && overlay.camera_drop_shadow != 0) {
    float camera_sigma = margin_capped_sigma(
      float2(overlay.camera_frame_x, overlay.camera_frame_y), camera_size,
      float2(dimensions), shadow_sigma(camera_size));
    if (camera_sigma > 1.0) {
      float camera_shadow = soft_shadow(
        camera_point, camera_size, float(overlay.camera_radius),
        camera_sigma, 0.14);
      // The shadow belongs to the area outside the camera frame. Applying it
      // to the accumulated canvas before compositing the camera tints the
      // camera itself as well, especially when the overlay is large.
      rgba.rgb *= 1.0 - camera_shadow * (1.0 - camera_coverage);
    }
  }
  if (camera_coverage > 0.0) {
    float2 source_point = float2(overlay.camera_crop_x, overlay.camera_crop_y) +
      clamp(camera_point, float2(0.0), camera_size) / camera_size *
      float2(overlay.camera_crop_width, overlay.camera_crop_height);
    uint2 camera_pixel = min(uint2(source_point), uint2(
      overlay.camera_source_width - 1, overlay.camera_source_height - 1));
    float4 camera_rgba = float4(camera[
      camera_pixel.y * overlay.camera_source_width + camera_pixel.x]) / 255.0;
    rgba = mix(rgba, camera_rgba, camera_coverage * camera_rgba.a);
  }
  if (overlay.camera_frame_width > 0 && overlay.camera_on_top == 0) {
    rgba = overlay_canvas_foreground_rgba(
      rgba, source, source_dimensions.x, source_dimensions.y,
      float2(gid) + 0.5, float2(dimensions), u);
    rgba = mix(rgba, cursor_rgba, cursor_rgba.a);
    // The redrawn foreground covers the pass above, so the marks under the
    // camera go back over it exactly as the cursor does.
    rgba = composite_annotations(rgba, annotations, annotation_count, 0u,
                                 float2(gid) + 0.5, u, float2(source_dimensions),
                                 annotation_pixel_scale, annotation_samples,
                               annotation_numbers, annotation_atlas);
  }
  rgba = composite_keyboard(rgba, keyboard_pixels, keyboard,
                            float2(gid) + 0.5, float2(dimensions));
  rgba = composite_annotations(rgba, annotations, annotation_count, 1u,
                               float2(gid) + 0.5, u, float2(source_dimensions),
                               annotation_pixel_scale, annotation_samples,
                               annotation_numbers, annotation_atlas);
  float canvas_coverage = rounded_coverage(
    float2(gid) + 0.5, float2(dimensions), float(u.background_radius));
  if (u.foreground_only == 0) rgba.rgb = output_dither(rgba.rgb, float2(gid));
  rgba.rgb *= canvas_coverage;
  rgba.a = u.foreground_only != 0 || u.transparent_background != 0
    ? rgba.a * canvas_coverage : 1.0;
  output[gid.y * dimensions.x + gid.x] = uchar4(
    clamp(rgba, 0.0, 1.0) * 255.0 + 0.5);
}

kernel void present_canvas_rgba(
    const device uchar4 *source [[buffer(0)]],
    constant CanvasUniforms &u [[buffer(1)]],
    constant uint2 &source_dimensions [[buffer(2)]],
    constant float &seconds [[buffer(3)]],
    const device uchar4 *cursor [[buffer(4)]],
    const device uchar4 *camera [[buffer(5)]],
    constant StillOverlayUniforms &overlay [[buffer(6)]],
    texture2d<float, access::write> output [[texture(0)]],
    texture2d<float, access::sample> background_picture [[texture(1)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 dimensions(output.get_width(), output.get_height());
  if (any(gid >= dimensions)) return;
  float2 canvas_dimensions = float2(dimensions);
  float2 point = float2(gid) + 0.5;
  float4 rgba = canvas_rgba_pixel(source, source_dimensions.x,
                                  source_dimensions.y, point,
                                  canvas_dimensions, u, seconds,
                                  background_picture);
  int2 cursor_point = int2(floor(point)) - int2(overlay.cursor_x, overlay.cursor_y);
  if (overlay.cursor_width > 0 && cursor_point.x >= 0 && cursor_point.y >= 0 &&
      cursor_point.x < int(overlay.cursor_width) &&
      cursor_point.y < int(overlay.cursor_height)) {
    bool cursor_visible = true;
    if (u.clip_cursor_at_video_edge != 0) {
      float2 crop_point = point - float2(u.crop_x, u.crop_y);
      float2 crop_size = float2(u.crop_width, u.crop_height);
      cursor_visible = all(crop_point >= 0.0) && all(crop_point < crop_size) &&
        rounded_pixel_visible(crop_point, crop_size, float(u.radius));
    }
    if (cursor_visible) {
      float4 cursor_pixel = still_cursor_pixel(
        cursor, float2(cursor_point),
        uint2(overlay.cursor_width, overlay.cursor_height),
        uint2(overlay.cursor_source_width, overlay.cursor_source_height));
      rgba = mix(rgba, cursor_pixel, cursor_pixel.a);
    }
  }
  float2 camera_point = point -
    float2(overlay.camera_frame_x, overlay.camera_frame_y);
  float2 camera_size = float2(
    overlay.camera_frame_width, overlay.camera_frame_height);
  float camera_coverage = overlay.camera_frame_width > 0
    ? rounded_coverage(camera_point, camera_size, float(overlay.camera_radius))
    : 0.0;
  if (overlay.camera_frame_width > 0 && overlay.camera_drop_shadow != 0) {
    float camera_sigma = margin_capped_sigma(
      float2(overlay.camera_frame_x, overlay.camera_frame_y), camera_size,
      float2(dimensions), shadow_sigma(camera_size));
    if (camera_sigma > 1.0) {
      float camera_shadow = soft_shadow(
        camera_point, camera_size, float(overlay.camera_radius),
        camera_sigma, 0.14);
      rgba.rgb *= 1.0 - camera_shadow * (1.0 - camera_coverage);
    }
  }
  if (camera_coverage > 0.0) {
    float2 source_point = float2(overlay.camera_crop_x, overlay.camera_crop_y) +
      clamp(camera_point, float2(0.0), camera_size) / camera_size *
      float2(overlay.camera_crop_width, overlay.camera_crop_height);
    uint2 camera_pixel = min(uint2(source_point), uint2(
      overlay.camera_source_width - 1, overlay.camera_source_height - 1));
    float4 camera_rgba = float4(camera[
      camera_pixel.y * overlay.camera_source_width + camera_pixel.x]) / 255.0;
    rgba = mix(rgba, camera_rgba, camera_coverage * camera_rgba.a);
  }
  if (overlay.camera_frame_width > 0 && overlay.camera_on_top == 0) {
    rgba = overlay_canvas_foreground_rgba(
      rgba, source, source_dimensions.x, source_dimensions.y, point,
      canvas_dimensions, u);
    if (overlay.cursor_width > 0 && cursor_point.x >= 0 && cursor_point.y >= 0 &&
        cursor_point.x < int(overlay.cursor_width) &&
        cursor_point.y < int(overlay.cursor_height)) {
      bool cursor_visible = true;
      if (u.clip_cursor_at_video_edge != 0) {
        float2 crop_point = point - float2(u.crop_x, u.crop_y);
        float2 crop_size = float2(u.crop_width, u.crop_height);
        cursor_visible = all(crop_point >= 0.0) && all(crop_point < crop_size) &&
          rounded_pixel_visible(crop_point, crop_size, float(u.radius));
      }
      if (cursor_visible) {
        float4 cursor_pixel = still_cursor_pixel(
          cursor, float2(cursor_point),
          uint2(overlay.cursor_width, overlay.cursor_height),
          uint2(overlay.cursor_source_width, overlay.cursor_source_height));
        rgba = mix(rgba, cursor_pixel, cursor_pixel.a);
      }
    }
  }
  float canvas_coverage = rounded_coverage(
    point, canvas_dimensions, float(u.background_radius));
  if (u.foreground_only == 0) rgba.rgb = output_dither(rgba.rgb, float2(gid));
  rgba.rgb *= canvas_coverage;
  rgba.a = u.foreground_only != 0 || u.transparent_background != 0
    ? rgba.a * canvas_coverage : 1.0;
  output.write(clamp(rgba, 0.0, 1.0), gid);
}

)METAL"
