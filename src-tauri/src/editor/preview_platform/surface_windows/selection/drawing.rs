// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl SelectionOverlay {
  #[allow(clippy::too_many_arguments)]
  pub(in crate::editor::preview_platform::surface_windows) fn draw(
    &mut self,
    viewport_size: (u32, u32),
    frame: Option<[f32; 4]>,
    radius_point: Option<[f32; 2]>,
    crop_image: Option<[f32; 4]>,
    // The selected layer's corner radius, as a percentage of the crop
    // rectangle's shorter side, so the shade rounds with the result.
    crop_radius_percent: f64,
    guides: Option<(Option<f32>, Option<f32>, bool, bool)>,
    magnifier_box: Option<[f32; 4]>,
    // `Some` when the arrow chrome owns the screen, holding the selected
    // arrow's grips in device pixels - possibly none, with the arrow tool in
    // hand and nothing chosen. The layer's own chrome stands down for as long
    // as it does, matching `annotation_owns_chrome`; `None` leaves it up.
    annotation_handles: Option<&[[f32; 2]]>,
    // The chosen box annotation's box in device pixels and its radius
    // percentage, drawn as the layer selection draws its own; a stroke's has
    // no radius dot.
    annotation_box: Option<([f32; 4], Option<f64>)>,
    // The chosen image's own frame in device pixels: its four corners,
    // clockwise from the picture's own top-left, then its radius dot.
    annotation_frame: Option<[[f32; 2]; 5]>,
    // The element an arrow's tip has snapped to, outlined a device pixel
    // wide so the anchor it took reads as part of that element.
    annotation_bounds: Option<[f32; 4]>,
    // The equal-gap bars a snapped counter lined up with, already device
    // pixel rectangles: a hairline across each gap with a tick at each end.
    annotation_gaps: &[[f32; 4]],
    // The boxes round annotations chosen together, whole groups first, and
    // the marquee band being drawn, all in device pixels.
    annotation_group: &[[f32; 4]],
    annotation_marquee: Option<[f32; 4]>,
    scale: f64,
    light: bool,
  ) -> Result<(), String> {
    let size = (viewport_size.0.max(2), viewport_size.1.max(2));
    let gpu = self.shared;
    self.surface.resize(gpu, size);
    let scale = scale.max(0.1);
    let view = Size {
      width: f64::from(size.0) / scale,
      height: f64::from(size.1) / scale,
    };
    let mut constants = RenderConstants::new(light);
    if let Some(box_rect) = magnifier_box.filter(|rect| rect[2] > 0.0) {
      constants.magnifier_box = box_rect;
      // The magnifier pixels are composed into the pane below this transparent
      // OSC swap chain. Shared chrome only owns the cutout here.
      constants.magnifier_flags[1] = 1;
    }
    let mut vertices = Vec::with_capacity(96);
    // The arrow chrome draws only its own grips: the layer's selection and
    // crop chrome stand down for as long as it has the pointer.
    if let Some(annotation_handles) = annotation_handles {
      // The element first: it is the context for the anchor that landed on
      // it, so the disc sits over its outline rather than under it. The
      // object colour is the one the guides use too.
      if let Some(bounds) = annotation_bounds.filter(|rect| rect[2] > 0.0 && rect[3] > 0.0) {
        let rect = logical_rect(bounds, scale);
        let pixel = 1.0 / scale;
        for edge in [
          Rect::from_xywh(rect.origin.x, rect.origin.y, rect.size.width, pixel),
          Rect::from_xywh(
            rect.origin.x,
            rect.origin.y + rect.size.height - pixel,
            rect.size.width,
            pixel,
          ),
          Rect::from_xywh(rect.origin.x, rect.origin.y, pixel, rect.size.height),
          Rect::from_xywh(
            rect.origin.x + rect.size.width - pixel,
            rect.origin.y,
            pixel,
            rect.size.height,
          ),
        ] {
          osc_gpu::add_pixel_aligned_quad(&mut vertices, view, edge, scale, 5);
        }
      }
      // The equal gaps the counter lined up with, in the object colour the
      // guides and the element outline use.
      for bar in annotation_gaps {
        osc_gpu::add_pixel_aligned_quad(&mut vertices, view, logical_rect(*bar, scale), scale, 5);
      }
      if let Some((frame, radius_percent)) = annotation_box {
        osc_gpu::add_selection(
          &mut vertices,
          view,
          logical_rect(frame, scale),
          scale,
          radius_percent.unwrap_or(0.0),
          radius_percent.is_some(),
        );
      }
      if let Some(frame) = annotation_frame {
        let [a, b, c, d, dot] = frame.map(|[x, y]| Point {
          x: f64::from(x) / scale,
          y: f64::from(y) / scale,
        });
        osc_gpu::add_turned_selection(&mut vertices, view, [a, b, c, d], dot, scale);
      }
      for frame in annotation_group {
        osc_gpu::add_group_frame(&mut vertices, view, logical_rect(*frame, scale), scale);
      }
      // The region selector's marching ants with no shade and no handles:
      // the band is its own image, so nothing round it is dimmed.
      if let Some(band) = annotation_marquee.filter(|rect| rect[2] > 0.0 || rect[3] > 0.0) {
        let band = logical_rect(band, scale);
        osc_gpu::add_crop_with_handles(&mut vertices, view, band, band, scale, 0.0, true, false);
      }
      let points = annotation_handles
        .iter()
        .map(|point| Point {
          x: f64::from(point[0]) / scale,
          y: f64::from(point[1]) / scale,
        })
        .collect::<Vec<_>>();
      osc_gpu::add_annotation_handles(&mut vertices, view, &points, scale);
    } else if let Some(frame) = frame {
      let logical_frame = logical_rect(frame, scale);
      if let Some(image) = crop_image.filter(|rect| rect[2] >= 0.0) {
        osc_gpu::add_crop(
          &mut vertices,
          view,
          logical_frame,
          logical_rect(image, scale),
          scale,
          crop_radius_percent,
        );
      } else {
        let radius_percent = radius_point
          .filter(|point| point[0].is_finite() && point[1].is_finite())
          .map(|point| {
            let shortest = logical_frame
              .size
              .width
              .min(logical_frame.size.height)
              .max(1.0);
            (((f64::from(point[0] - frame[0]) / scale - 10.0) / (shortest * 0.55)) * 100.0)
              .clamp(0.0, 50.0)
          });
        osc_gpu::add_selection(
          &mut vertices,
          view,
          logical_frame,
          scale,
          radius_percent.unwrap_or_default(),
          radius_percent.is_some(),
        );
      }
    }
    if let Some((x, y, x_object, y_object)) = guides {
      let half = 0.5 / scale;
      if let Some(x) = x {
        let x = f64::from(x) / scale;
        osc_gpu::add_pixel_aligned_quad(
          &mut vertices,
          view,
          Rect::from_xywh(x - half, 0.0, half * 2.0, view.height),
          scale,
          if x_object { 5 } else { 4 },
        );
      }
      if let Some(y) = y {
        let y = f64::from(y) / scale;
        osc_gpu::add_pixel_aligned_quad(
          &mut vertices,
          view,
          Rect::from_xywh(0.0, y - half, view.width, half * 2.0),
          scale,
          if y_object { 5 } else { 4 },
        );
      }
    }
    let Frame::Ready(frame) = self.surface.acquire(gpu)? else {
      return Ok(());
    };
    // Every vertex shares the frame's constants, so the chrome is one draw.
    if !vertices.is_empty() {
      if vertices.len() > self.vertex_capacity {
        self.vertex_capacity = vertices.len().next_power_of_two().max(256);
        self.vertex_buffer = vertex_buffer(gpu, self.vertex_capacity);
      }
      gpu
        .queue
        .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
      gpu
        .queue
        .write_buffer(&self.constants, 0, bytemuck::bytes_of(&constants));
    }
    let target = frame.texture.create_view(&Default::default());
    let mut encoder = gpu
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenwide selection overlay"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Screenwide selection overlay"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
      if !vertices.is_empty() {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.bindings, &[0]);
        pass.draw(0..vertices.len() as u32, 0..1);
      }
    }
    gpu.queue.submit([encoder.finish()]);
    gpu.queue.present(frame);
    Ok(())
  }
}
