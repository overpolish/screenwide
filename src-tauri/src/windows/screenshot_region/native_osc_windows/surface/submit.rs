// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(super) fn submit(
    &mut self,
    vertices: &[Vertex],
    segments: &[Segment],
    constants: &RenderConstants,
    size: (u32, u32),
  ) -> Result<(), String> {
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
      .map_err(|error| format!("The Windows region OSC could not resize: {error}"))?;
      self.buffer_size = size;
    }
    self.write_vertices(vertices)?;
    let gpu = Arc::clone(&self.gpu);
    let constants_resource: ID3D11Resource = gpu.constants.cast().map_err(|e| e.to_string())?;
    let index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() };
    let texture = unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(index) }
      .map_err(|error| error.to_string())?;
    let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let mut target: Option<ID3D11RenderTargetView> = None;
    unsafe {
      gpu
        .device
        .CreateRenderTargetView(&resource, None, Some(&mut target))
    }
    .map_err(|error| error.to_string())?;
    let target = target.ok_or_else(|| "D3D11 created no region OSC target".to_owned())?;
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
    unsafe {
      // Flip-discard back buffers are undefined after a present.
      gpu.context.ClearRenderTargetView(&target, &[0.0; 4]);
      gpu.context.OMSetRenderTargets(Some(&[Some(target)]), None);
      gpu
        .context
        .OMSetBlendState(blend, Some(&[0.0; 4]), 0xffff_ffff);
      gpu.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: size.0 as f32,
        Height: size.1 as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      gpu.context.RSSetState(&gpu.rasterizer);
      gpu.context.IASetInputLayout(&gpu.layout);
      gpu
        .context
        .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      let stride = size_of::<Vertex>() as u32;
      let offset = 0_u32;
      gpu.context.IASetVertexBuffers(
        0,
        1,
        Some(&self.vertex_buffer.clone()),
        Some(&stride),
        Some(&offset),
      );
      gpu.context.VSSetShader(&gpu.vertex_shader, None);
      gpu.context.PSSetShader(&gpu.pixel_shader, None);
      gpu
        .context
        .PSSetConstantBuffers(0, Some(&[Some(gpu.constants.clone())]));
      gpu.context.PSSetSamplers(
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
        gpu.context.UpdateSubresource(
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
        gpu.context.PSSetShaderResources(
          0,
          Some(&[
            Some(label),
            Some(secondary),
            Some(gpu.icons.clone()),
            Some(snapshot.clone()),
            Some(magnifier.clone()),
          ]),
        );
        gpu.context.Draw(segment.count, segment.start);
      }
      gpu
        .context
        .PSSetShaderResources(0, Some(&[None, None, None, None, None]));
      gpu.context.OMSetRenderTargets(None, None);
      // Never block the pointer thread on the compositor.
      self
        .swap_chain
        .Present(0, DXGI_PRESENT(0))
        .ok()
        .map_err(|error| error.to_string())?;
    }
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
        self.gpu.device.CreateBuffer(
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
    unsafe {
      self
        .gpu
        .context
        .Map(&resource, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped))
    }
    .map_err(|error| error.to_string())?;
    unsafe {
      std::ptr::copy_nonoverlapping(
        vertices.as_ptr(),
        mapped.pData.cast::<Vertex>(),
        vertices.len(),
      );
      self.gpu.context.Unmap(&resource, 0);
    }
    Ok(())
  }
}
