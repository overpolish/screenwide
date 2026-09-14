// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl AudioRibbon {
  pub(super) fn draw(&mut self) -> Result<(), String> {
    let Some(samples) = self.pixels.clone() else {
      return Ok(());
    };
    let [red, green, blue] = crate::system_accent::accent_rgb();
    let constants = Constants {
      color: [red, green, blue, 1.0],
      flat: [self.neutral, self.neutral, self.neutral, 0.25],
      geometry: [
        self.viewport.0 as f32,
        self.viewport.1 as f32,
        5.0 * self.scale,
        2.0 * self.scale,
      ],
      style: [
        self.viewport.1 as f32 * 0.30,
        self.scale,
        self.playhead,
        self.points as f32,
      ],
    };
    // A paused waveform needs no repeated presents; recoloring or replacing
    // envelopes still invalidates it even if its playhead has not moved.
    if !self.dirty && self.last_constants == Some(constants) {
      return Ok(());
    }
    let buffer = unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(0) }
      .map_err(|error| error.to_string())?;
    let mut target = None;
    unsafe {
      self
        .device
        .CreateRenderTargetView(&buffer, None, Some(&mut target))
    }
    .map_err(|error| error.to_string())?;
    let target = target.ok_or("The audio ribbon render target was not created")?;
    let resource: ID3D11Resource = self.constants.cast().map_err(|error| error.to_string())?;
    unsafe {
      self.context.ClearRenderTargetView(&target, &[0.0; 4]);
      self.context.UpdateSubresource(
        &resource,
        0,
        None,
        (&raw const constants).cast::<c_void>(),
        0,
        0,
      );
      self.context.OMSetRenderTargets(Some(&[Some(target)]), None);
      self
        .context
        .OMSetBlendState(Some(&self.blend), None, u32::MAX);
      self
        .context
        .RSSetState(None::<&windows::Win32::Graphics::Direct3D11::ID3D11RasterizerState>);
      self.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: self.viewport.0 as f32,
        Height: self.viewport.1 as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      self
        .context
        .IASetInputLayout(None::<&windows::Win32::Graphics::Direct3D11::ID3D11InputLayout>);
      self
        .context
        .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      self.context.VSSetShader(&self.vertex, None);
      self.context.PSSetShader(&self.pixel, None);
      self
        .context
        .PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
      self.context.PSSetShaderResources(0, Some(&[Some(samples)]));
      self.context.Draw(3, 0);
      self.context.PSSetShaderResources(0, Some(&[None]));
      self.context.OMSetRenderTargets(None, None);
      self
        .swap_chain
        .Present(0, windows::Win32::Graphics::Dxgi::DXGI_PRESENT(0))
        .ok()
    }
    .map_err(|error| error.to_string())?;
    self.dirty = false;
    self.last_constants = Some(constants);
    Ok(())
  }
}
