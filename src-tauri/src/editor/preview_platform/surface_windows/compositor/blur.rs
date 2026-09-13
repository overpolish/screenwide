// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  pub(in crate::editor::preview_platform::surface) fn blur_targets(
    &self,
    device: &ID3D11Device,
    size: (u32, u32),
  ) -> Result<BlurTargets, String> {
    let description = D3D11_TEXTURE2D_DESC {
      Width: size.0.max(1),
      Height: size.1.max(1),
      MipLevels: 1,
      ArraySize: 1,
      // The back buffer's format, so the vertical pass writes it unconverted.
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
      ..Default::default()
    };
    let mut targets = Vec::with_capacity(2);
    for _ in 0..2 {
      let mut texture = None;
      unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
        .map_err(|error| format!("The suspended preview blur has no target: {error}"))?;
      let texture = texture.ok_or_else(|| "D3D11 created no preview blur target".to_owned())?;
      let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
      let mut view = None;
      unsafe { device.CreateShaderResourceView(&resource, None, Some(&mut view)) }
        .map_err(|error| error.to_string())?;
      let view = view.ok_or_else(|| "D3D11 created no preview blur view".to_owned())?;
      targets.push((texture, view));
    }
    let (scratch, scratch_view) = targets.pop().expect("two blur targets were created");
    let (composed, composed_view) = targets.pop().expect("two blur targets were created");
    Ok(BlurTargets {
      size,
      composed,
      composed_view,
      scratch,
      scratch_view,
    })
  }

  /// Blurs the frame already drawn into `targets.composed()` into `target`
  /// with a separable Gaussian of `sigma` pane pixels.
  pub(in crate::editor::preview_platform::surface) fn blur(
    &self,
    context: &ID3D11DeviceContext,
    targets: &BlurTargets,
    target: &ID3D11Texture2D,
    size: (u32, u32),
    sigma: f32,
  ) -> Result<(), String> {
    // Three sigma covers the Gaussian to well under a 8-bit quantum, which is
    // also where a CSS `filter: blur()` truncates its kernel.
    let radius = (sigma * 3.0).ceil().max(1.0);
    let pairs = ((radius / 2.0).ceil() as u32).clamp(1, BLUR_TAP_PAIRS);
    let spacing = (radius / (pairs * 2) as f32).max(1.0);
    let texel = [
      1.0 / size.0.max(1) as f32,
      1.0 / size.1.max(1) as f32,
      sigma,
      pairs as f32,
    ];
    self.blur_pass(
      context,
      &targets.composed_view,
      &targets.scratch,
      size,
      texel,
      [1.0, 0.0, spacing, 0.0],
    )?;
    self.blur_pass(
      context,
      &targets.scratch_view,
      target,
      size,
      texel,
      [0.0, 1.0, spacing, 0.0],
    )
  }

  fn blur_pass(
    &self,
    context: &ID3D11DeviceContext,
    source: &ID3D11ShaderResourceView,
    target: &ID3D11Texture2D,
    size: (u32, u32),
    texel: [f32; 4],
    axis: [f32; 4],
  ) -> Result<(), String> {
    let values = BlurConstants { texel, axis };
    unsafe {
      self
        .blur_constants
        .cast::<ID3D11Resource>()
        .map_err(|error| error.to_string())
        .map(|resource| {
          context.UpdateSubresource(
            &resource,
            0,
            None,
            (&raw const values).cast::<c_void>(),
            0,
            0,
          );
        })?;
    }
    let resource: ID3D11Resource = target.cast().map_err(|error| error.to_string())?;
    let device = unsafe { target.GetDevice() }.map_err(|error| error.to_string())?;
    let mut render_target: Option<ID3D11RenderTargetView> = None;
    unsafe { device.CreateRenderTargetView(&resource, None, Some(&mut render_target)) }
      .map_err(|error| format!("The suspended preview blur has no render target: {error}"))?;
    let render_target =
      render_target.ok_or_else(|| "D3D11 created no preview blur render target".to_owned())?;
    let viewport = D3D11_VIEWPORT {
      Width: size.0 as f32,
      Height: size.1 as f32,
      MaxDepth: 1.0,
      ..Default::default()
    };
    unsafe {
      context.OMSetRenderTargets(Some(&[Some(render_target)]), None);
      // The pass replaces the target outright; a layer pane's premultiplied
      // transparency is carried in the blurred pixels themselves.
      context.OMSetBlendState(None::<&ID3D11BlendState>, None, u32::MAX);
      context.RSSetViewports(Some(&[viewport]));
      context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(&self.blur_vertex_shader, None);
      context.PSSetShader(&self.blur_pixel_shader, None);
      context.PSSetConstantBuffers(0, Some(&[Some(self.blur_constants.clone())]));
      context.PSSetShaderResources(0, Some(&[Some(source.clone())]));
      context.PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));
      context.Draw(3, 0);
      context.PSSetShaderResources(0, Some(&[None]));
      context.OMSetRenderTargets(None, None);
    }
    Ok(())
  }
}
