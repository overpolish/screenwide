// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Transparent D3D11 selection overlay composed above the preview panes.

#[path = "selection/drawing.rs"]
mod drawing;
#[path = "selection/pipeline.rs"]
mod pipeline;

use pipeline::blend_state;
use pipeline::create_vertex_buffer;
use pipeline::input_elements;
use pipeline::rasterizer_state;
use pipeline::sampler;

use std::ffi::c_void;

use windows::{
  core::{s, Interface},
  Win32::Graphics::{
    Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    Direct3D11::{
      ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
      ID3D11PixelShader, ID3D11RasterizerState, ID3D11RenderTargetView, ID3D11Resource,
      ID3D11SamplerState, ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER,
      D3D11_BIND_VERTEX_BUFFER, D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_OP_ADD,
      D3D11_BLEND_SRC_ALPHA, D3D11_BUFFER_DESC, D3D11_COLOR_WRITE_ENABLE_ALL,
      D3D11_CPU_ACCESS_WRITE, D3D11_CULL_NONE, D3D11_FILL_SOLID, D3D11_FILTER,
      D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_FILTER_MIN_MAG_MIP_POINT, D3D11_INPUT_ELEMENT_DESC,
      D3D11_INPUT_PER_VERTEX_DATA, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE_DISCARD,
      D3D11_RASTERIZER_DESC, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_SAMPLER_DESC,
      D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT,
    },
    DirectComposition::{IDCompositionDevice, IDCompositionVisual},
    Dxgi::{
      Common::{
        DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R32G32_FLOAT,
        DXGI_FORMAT_R32_UINT, DXGI_SAMPLE_DESC,
      },
      IDXGIFactory2, IDXGISwapChain3, DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
      DXGI_SWAP_CHAIN_FLAG, DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
    },
  },
};

#[path = "selection/placeholder.rs"]
mod placeholder;

use crate::osc::{
  geometry::{Rect, Size},
  gpu::windows::{self as osc_gpu, RenderConstants, Vertex, PIXEL_SHADER, VERTEX_SHADER},
};
use placeholder::{placeholder_texture, PlaceholderTexture};

#[derive(Clone)]
struct Segment {
  constants: RenderConstants,
  count: u32,
  start: u32,
}

fn logical_rect(rect: [f32; 4], scale: f64) -> Rect {
  Rect::from_xywh(
    f64::from(rect[0]) / scale,
    f64::from(rect[1]) / scale,
    f64::from(rect[2]) / scale,
    f64::from(rect[3]) / scale,
  )
}

pub(super) struct SelectionOverlay {
  blend: ID3D11BlendState,
  buffer_size: (u32, u32),
  constants: ID3D11Buffer,
  layout: ID3D11InputLayout,
  /// Bound to every pixel-shader texture slot, which the overlay's own quads
  /// never sample.
  placeholder: PlaceholderTexture,
  linear_sampler: ID3D11SamplerState,
  pixel_shader: ID3D11PixelShader,
  point_sampler: ID3D11SamplerState,
  rasterizer: ID3D11RasterizerState,
  swap_chain: IDXGISwapChain3,
  vertex_buffer: ID3D11Buffer,
  vertex_capacity: usize,
  vertex_shader: ID3D11VertexShader,
  /// Held only to keep the composition visual alive: the swap chain is
  /// attached once and no property is mutated after construction.
  _visual: IDCompositionVisual,
}

impl SelectionOverlay {
  pub(super) fn new(
    device: &ID3D11Device,
    factory: &IDXGIFactory2,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let description = DXGI_SWAP_CHAIN_DESC1 {
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
    let swap_chain = unsafe { factory.CreateSwapChainForComposition(device, &description, None) }
      .and_then(|chain| chain.cast::<IDXGISwapChain3>())
      .map_err(|error| format!("The Windows selection swap chain could not be created: {error}"))?;
    let visual = unsafe { composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe {
      visual
        .SetContent(&swap_chain)
        .map_err(|error| error.to_string())?;
      root
        .AddVisual(&visual, true, None::<&IDCompositionVisual>)
        .map_err(|error| error.to_string())?;
    }
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
      device
        .CreateBuffer(
          &D3D11_BUFFER_DESC {
            ByteWidth: size_of::<RenderConstants>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            ..Default::default()
          },
          None,
          Some(&mut constants),
        )
        .map_err(|error| error.to_string())?;
    }
    let blend = blend_state(device)?;
    let rasterizer = rasterizer_state(device)?;
    let linear_sampler = sampler(device, D3D11_FILTER_MIN_MAG_MIP_LINEAR)?;
    let point_sampler = sampler(device, D3D11_FILTER_MIN_MAG_MIP_POINT)?;
    let vertex_capacity = 256;
    let vertex_buffer = create_vertex_buffer(device, vertex_capacity)?;
    let placeholder = placeholder_texture(device)?;
    Ok(Self {
      blend,
      buffer_size: (2, 2),
      constants: constants.ok_or_else(|| "D3D11 created no selection constants".to_owned())?,
      layout: layout.ok_or_else(|| "D3D11 created no shared OSC input layout".to_owned())?,
      placeholder,
      linear_sampler,
      pixel_shader: pixel_shader
        .ok_or_else(|| "D3D11 created no selection pixel shader".to_owned())?,
      point_sampler,
      rasterizer,
      swap_chain,
      vertex_buffer,
      vertex_capacity,
      vertex_shader: vertex_shader
        .ok_or_else(|| "D3D11 created no selection vertex shader".to_owned())?,
      _visual: visual,
    })
  }
}
