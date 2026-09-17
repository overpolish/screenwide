// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  /// Releases session-sized presentation buffers while retaining captured
  /// textures and interaction state for the next visible session.
  pub(super) fn release_drawables(&mut self) {
    if self.drawables_released {
      return;
    }
    self.drawables_released = true;
    self.vertex_buffer = None;
    self.vertex_capacity = 0;
    self.vertices.clear();
    self.vertices.shrink_to_fit();

    let context = self.gpu.context();
    unsafe {
      context.IASetVertexBuffers(0, 1, Some(&None), Some(&0), Some(&0));
      context.PSSetShaderResources(0, Some(&[None, None, None, None, None]));
      context.OMSetRenderTargets(None, None);
    }
    if let Err(error) = self.chain.resize((2, 2)) {
      eprintln!("The Windows region OSC could not release swap-chain buffers: {error}");
    }
  }

  pub(super) fn submit(
    &mut self,
    vertices: &[Vertex],
    segments: &[Segment],
    constants: &RenderConstants,
    size: (u32, u32),
  ) -> Result<(), String> {
    self.chain.resize(size)?;
    self.write_vertices(vertices)?;
    let gpu = Arc::clone(&self.gpu);
    let constants_resource: ID3D11Resource = gpu.constants.cast().map_err(|e| e.to_string())?;
    let target = self.chain.back_buffer_view(gpu.device())?;
    let magnifier = self
      .magnifier_source
      .as_ref()
      .map_or_else(|| gpu.placeholder.clone(), |source| source.view.clone());
    let snapshot = self
      .snapshot
      .as_ref()
      .map_or_else(|| gpu.placeholder.clone(), |source| source.view.clone());
    // macOS puts a non-composited OCR snapshot in an opaque CALayer beneath
    // its transparent Metal layer. Windows folds both into this target, so
    // every presented snapshot-not only Ruler's composited one-must retain
    // opaque destination alpha as translucent shading is drawn over it.
    let blend = if opaque_snapshot_target(self.snapshot_presented, self.snapshot.is_some()) {
      &gpu.opaque_blend
    } else {
      &gpu.blend
    };
    let context = gpu.context();
    unsafe {
      // Flip-discard back buffers are undefined after a present.
      context.ClearRenderTargetView(&target, &[0.0; 4]);
      context.OMSetRenderTargets(Some(&[Some(target)]), None);
      context.OMSetBlendState(blend, Some(&[0.0; 4]), 0xffff_ffff);
      context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: size.0 as f32,
        Height: size.1 as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      context.RSSetState(&gpu.rasterizer);
      context.IASetInputLayout(&gpu.layout);
      context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      let stride = size_of::<Vertex>() as u32;
      let offset = 0_u32;
      context.IASetVertexBuffers(
        0,
        1,
        Some(&self.vertex_buffer.clone()),
        Some(&stride),
        Some(&offset),
      );
      context.VSSetShader(&gpu.vertex_shader, None);
      context.PSSetShader(&gpu.pixel_shader, None);
      context.PSSetConstantBuffers(0, Some(&[Some(gpu.constants.clone())]));
      context.PSSetSamplers(
        0,
        Some(&[
          Some(gpu.linear_sampler.clone()),
          Some(gpu.point_sampler.clone()),
        ]),
      );
      // One draw call per constant-buffer state. The base scene is a single
      // segment; each folded-in control adds one because its fill, foreground
      // and label texture are its own.
      for segment in segments {
        if segment.count == 0 {
          continue;
        }
        let mut frame = *constants;
        frame.action_fills = segment.action_fills;
        frame.chrome = segment.chrome;
        frame.chrome_outline = segment.chrome_outline;
        context.UpdateSubresource(
          &constants_resource,
          0,
          None,
          (&raw const frame).cast::<c_void>(),
          0,
          0,
        );
        let label = segment
          .label
          .clone()
          .unwrap_or_else(|| gpu.placeholder.clone());
        let secondary = segment
          .secondary
          .clone()
          .unwrap_or_else(|| gpu.placeholder.clone());
        context.PSSetShaderResources(
          0,
          Some(&[
            Some(label),
            Some(secondary),
            Some(gpu.icons.clone()),
            Some(snapshot.clone()),
            Some(magnifier.clone()),
          ]),
        );
        context.Draw(segment.count, segment.start);
      }
      context.PSSetShaderResources(0, Some(&[None, None, None, None, None]));
      context.OMSetRenderTargets(None, None);
    }
    self.chain.present()?;
    Ok(())
  }

  pub(super) fn write_vertices(&mut self, vertices: &[Vertex]) -> Result<(), String> {
    if vertices.is_empty() {
      return Ok(());
    }
    if self.vertex_buffer.is_none() || self.vertex_capacity < vertices.len() {
      let capacity = vertices.len().next_power_of_two().max(512);
      let mut buffer = None;
      unsafe {
        self.gpu.device().CreateBuffer(
          &D3D11_BUFFER_DESC {
            ByteWidth: (capacity * size_of::<Vertex>()) as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
            ..Default::default()
          },
          None,
          Some(&mut buffer),
        )
      }
      .map_err(|error| format!("The Windows region OSC vertex buffer failed: {error}"))?;
      self.vertex_buffer = buffer;
      self.vertex_capacity = capacity;
    }
    let buffer = self
      .vertex_buffer
      .clone()
      .ok_or_else(|| "D3D11 created no region OSC vertex buffer".to_owned())?;
    let resource: ID3D11Resource = buffer.cast().map_err(|error| error.to_string())?;
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    let context = self.gpu.context();
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
    Ok(())
  }
}
