// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Gpu {
  pub(crate) fn new() -> Result<Arc<Self>, String> {
    let shared = overlay_surface::Device::new()?;
    let device = shared.device();
    let mut vertex_shader = None;
    let mut pixel_shader = None;
    let mut layout = None;
    unsafe {
      device
        .CreateVertexShader(VERTEX_SHADER, None, Some(&mut vertex_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreatePixelShader(PIXEL_SHADER, None, Some(&mut pixel_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreateInputLayout(&input_elements(), VERTEX_SHADER, Some(&mut layout))
        .map_err(|error| error.to_string())?;
    }
    let mut constants = None;
    unsafe {
      device.CreateBuffer(
        &D3D11_BUFFER_DESC {
          ByteWidth: size_of::<RenderConstants>() as u32,
          Usage: D3D11_USAGE_DEFAULT,
          BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
          ..Default::default()
        },
        None,
        Some(&mut constants),
      )
    }
    .map_err(|error| error.to_string())?;

    // Metal's normal pipeline: straight source-over on colour and alpha.
    let blend = blend_state(device, D3D11_BLEND_SRC_ALPHA)?;
    let opaque_blend = blend_state(device, D3D11_BLEND_ONE)?;
    // Metal's default is no face culling. Several shared OSC primitives are
    // deliberately emitted in either winding (notably line quads), so D3D's
    // default back-face culling silently removed rulers and probes.
    let mut rasterizer = None;
    unsafe {
      device.CreateRasterizerState(
        &D3D11_RASTERIZER_DESC {
          FillMode: D3D11_FILL_SOLID,
          CullMode: D3D11_CULL_NONE,
          DepthClipEnable: true.into(),
          ..Default::default()
        },
        Some(&mut rasterizer),
      )
    }
    .map_err(|error| error.to_string())?;
    let linear_sampler = sampler(device, D3D11_FILTER_MIN_MAG_MIP_LINEAR)?;
    let point_sampler = sampler(device, D3D11_FILTER_MIN_MAG_MIP_POINT)?;
    // No SRV slot may ever be null; t0-t4 fall back to one transparent texel.
    let placeholder = upload_rgba(device, &[0_u8; 4], 1, 1)?;
    let icons = upload_icons(device).unwrap_or_else(|error| {
      eprintln!("The Windows region OSC could not upload the icon atlas: {error}");
      placeholder.clone()
    });

    Ok(Arc::new(Self {
      shared,
      vertex_shader: vertex_shader
        .ok_or_else(|| "D3D11 created no region OSC vertex shader".to_owned())?,
      pixel_shader: pixel_shader
        .ok_or_else(|| "D3D11 created no region OSC pixel shader".to_owned())?,
      layout: layout.ok_or_else(|| "D3D11 created no region OSC input layout".to_owned())?,
      constants: constants.ok_or_else(|| "D3D11 created no region OSC constants".to_owned())?,
      rasterizer: rasterizer.ok_or_else(|| "D3D11 created no region OSC rasterizer".to_owned())?,
      blend,
      opaque_blend,
      linear_sampler,
      point_sampler,
      placeholder,
      icons,
    }))
  }
}
