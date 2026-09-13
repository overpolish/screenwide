// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl AudioRibbon {
  pub(in crate::editor::preview_platform::surface) fn new(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    factory: &IDXGIFactory2,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let desc = DXGI_SWAP_CHAIN_DESC1 {
      Width: 2,
      Height: 2,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
      BufferCount: 2,
      Scaling: DXGI_SCALING_STRETCH,
      SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
      AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
      ..Default::default()
    };
    let swap_chain = unsafe { factory.CreateSwapChainForComposition(device, &desc, None) }
      .and_then(|chain| chain.cast::<IDXGISwapChain3>())
      .map_err(|e| e.to_string())?;
    let visual = unsafe { composition.CreateVisual() }.map_err(|e| e.to_string())?;
    unsafe {
      visual
        .SetOffsetX2(-100000.0)
        .and_then(|_| visual.SetContent(&swap_chain))
        .and_then(|_| root.AddVisual(&visual, true, None::<&IDCompositionVisual>))
    }
    .map_err(|e| e.to_string())?;
    let mut vertex = None;
    let mut pixel = None;
    unsafe {
      device
        .CreateVertexShader(VS, None, Some(&mut vertex))
        .and_then(|_| device.CreatePixelShader(PS, None, Some(&mut pixel)))
    }
    .map_err(|e| e.to_string())?;
    let mut constants = None;
    unsafe {
      device.CreateBuffer(
        &D3D11_BUFFER_DESC {
          ByteWidth: size_of::<Constants>() as u32,
          Usage: D3D11_USAGE_DEFAULT,
          BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
          ..Default::default()
        },
        None,
        Some(&mut constants),
      )
    }
    .map_err(|e| e.to_string())?;
    let mut blend = None;
    unsafe {
      device.CreateBlendState(
        &D3D11_BLEND_DESC {
          RenderTarget: [
            D3D11_RENDER_TARGET_BLEND_DESC {
              BlendEnable: true.into(),
              SrcBlend: D3D11_BLEND_ONE,
              DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
              BlendOp: D3D11_BLEND_OP_ADD,
              SrcBlendAlpha: D3D11_BLEND_ONE,
              DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
              BlendOpAlpha: D3D11_BLEND_OP_ADD,
              RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
            },
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
            D3D11_RENDER_TARGET_BLEND_DESC::default(),
          ],
          ..Default::default()
        },
        Some(&mut blend),
      )
    }
    .map_err(|e| e.to_string())?;
    Ok(Self {
      blend: blend.ok_or("no ribbon blend")?,
      constants: constants.ok_or("no ribbon constants")?,
      context: context.clone(),
      device: device.clone(),
      pixels: None,
      pixel: pixel.ok_or("no ribbon pixel shader")?,
      swap_chain,
      visual,
      vertex: vertex.ok_or("no ribbon vertex shader")?,
      viewport: (2, 2),
      composition: composition.clone(),
      offset: (0.0, 0.0),
      scale: 1.0,
      neutral: 1.0,
      dirty: true,
      last_constants: None,
      points: 0,
      playhead: 0.0,
    })
  }
}
