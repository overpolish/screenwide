// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl SelectionOverlay {
  #[allow(clippy::too_many_arguments)]
  pub(in crate::editor::preview_platform::surface) fn draw(
    &mut self,
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
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
    // The element an arrow's tip has snapped to, outlined a device pixel
    // wide so the anchor it took reads as part of that element.
    annotation_bounds: Option<[f32; 4]>,
    scale: f64,
    light: bool,
  ) -> Result<(), String> {
    let size = (viewport_size.0.max(2), viewport_size.1.max(2));
    if size != self.buffer_size {
      unsafe {
        self.swap_chain.ResizeBuffers(
          2,
          size.0,
          size.1,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_SWAP_CHAIN_FLAG(0),
        )
      }
      .map_err(|error| format!("The Windows selection overlay could not resize: {error}"))?;
      self.buffer_size = size;
    }
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
    let mut segments = Vec::with_capacity(1);
    if !vertices.is_empty() {
      segments.push(Segment {
        constants,
        count: vertices.len() as u32,
        start: 0,
      });
    }
    if vertices.len() > self.vertex_capacity {
      self.vertex_capacity = vertices.len().next_power_of_two().max(256);
      self.vertex_buffer = create_vertex_buffer(device, self.vertex_capacity)?;
    }
    if !vertices.is_empty() {
      let resource: ID3D11Resource = self.vertex_buffer.cast().map_err(|e| e.to_string())?;
      let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
      unsafe { context.Map(&resource, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped)) }
        .map_err(|error| error.to_string())?;
      unsafe {
        std::ptr::copy_nonoverlapping(
          vertices.as_ptr(),
          mapped.pData.cast::<Vertex>(),
          vertices.len(),
        );
        context.Unmap(&resource, 0);
      }
    }
    let placeholder = self.placeholder.view.clone();
    let constant_resource: ID3D11Resource =
      self.constants.cast().map_err(|error| error.to_string())?;
    let texture = unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(0) }
      .map_err(|error| error.to_string())?;
    let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let mut target: Option<ID3D11RenderTargetView> = None;
    unsafe { device.CreateRenderTargetView(&resource, None, Some(&mut target)) }
      .map_err(|error| error.to_string())?;
    let target = target.ok_or_else(|| "D3D11 created no selection target".to_owned())?;
    unsafe {
      context.ClearRenderTargetView(&target, &[0.0; 4]);
      context.OMSetRenderTargets(Some(&[Some(target)]), None);
      context.OMSetBlendState(&self.blend, Some(&[0.0; 4]), 0xffff_ffff);
      context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: size.0 as f32,
        Height: size.1 as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      context.RSSetState(&self.rasterizer);
      context.IASetInputLayout(&self.layout);
      context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      let stride = size_of::<Vertex>() as u32;
      let offset = 0;
      let vertex_buffer = Some(self.vertex_buffer.clone());
      context.IASetVertexBuffers(
        0,
        1,
        Some(&raw const vertex_buffer),
        Some(&stride),
        Some(&offset),
      );
      context.VSSetShader(&self.vertex_shader, None);
      context.PSSetShader(&self.pixel_shader, None);
      context.PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
      context.PSSetSamplers(
        0,
        Some(&[
          Some(self.linear_sampler.clone()),
          Some(self.point_sampler.clone()),
        ]),
      );
      for segment in &segments {
        context.UpdateSubresource(
          &constant_resource,
          0,
          None,
          (&raw const segment.constants).cast::<c_void>(),
          0,
          0,
        );
        context.PSSetShaderResources(
          0,
          Some(&[
            Some(placeholder.clone()),
            Some(placeholder.clone()),
            Some(placeholder.clone()),
            Some(placeholder.clone()),
            Some(placeholder.clone()),
          ]),
        );
        context.Draw(segment.count, segment.start);
      }
      context.PSSetShaderResources(0, Some(&[None, None, None, None, None]));
      context.OMSetRenderTargets(None, None);
      self
        .swap_chain
        .Present(0, DXGI_PRESENT(0))
        .ok()
        .map_err(|error| error.to_string())?;
    }
    Ok(())
  }
}
