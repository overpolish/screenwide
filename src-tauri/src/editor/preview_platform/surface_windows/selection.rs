// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Transparent D3D11 selection overlay composed above the preview panes.

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

fn input_elements() -> [D3D11_INPUT_ELEMENT_DESC; 4] {
  [
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("POSITION"),
      Format: DXGI_FORMAT_R32G32_FLOAT,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      ..Default::default()
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      Format: DXGI_FORMAT_R32G32_FLOAT,
      AlignedByteOffset: 8,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      ..Default::default()
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 1,
      Format: DXGI_FORMAT_R32G32_FLOAT,
      AlignedByteOffset: 16,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      ..Default::default()
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 2,
      Format: DXGI_FORMAT_R32_UINT,
      AlignedByteOffset: 24,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      ..Default::default()
    },
  ]
}

fn blend_state(device: &ID3D11Device) -> Result<ID3D11BlendState, String> {
  let target = D3D11_RENDER_TARGET_BLEND_DESC {
    BlendEnable: true.into(),
    SrcBlend: D3D11_BLEND_SRC_ALPHA,
    DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
    BlendOp: D3D11_BLEND_OP_ADD,
    SrcBlendAlpha: D3D11_BLEND_SRC_ALPHA,
    DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
    BlendOpAlpha: D3D11_BLEND_OP_ADD,
    RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
  };
  let mut state = None;
  unsafe {
    device.CreateBlendState(
      &D3D11_BLEND_DESC {
        RenderTarget: [target; 8],
        ..Default::default()
      },
      Some(&mut state),
    )
  }
  .map_err(|error| error.to_string())?;
  state.ok_or_else(|| "D3D11 created no shared OSC blend state".to_owned())
}

fn rasterizer_state(device: &ID3D11Device) -> Result<ID3D11RasterizerState, String> {
  let mut state = None;
  unsafe {
    device.CreateRasterizerState(
      &D3D11_RASTERIZER_DESC {
        FillMode: D3D11_FILL_SOLID,
        CullMode: D3D11_CULL_NONE,
        DepthClipEnable: true.into(),
        ..Default::default()
      },
      Some(&mut state),
    )
  }
  .map_err(|error| error.to_string())?;
  state.ok_or_else(|| "D3D11 created no shared OSC rasterizer".to_owned())
}

fn sampler(device: &ID3D11Device, filter: D3D11_FILTER) -> Result<ID3D11SamplerState, String> {
  let mut state = None;
  unsafe {
    device.CreateSamplerState(
      &D3D11_SAMPLER_DESC {
        Filter: filter,
        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
        AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
        MaxLOD: f32::MAX,
        ..Default::default()
      },
      Some(&mut state),
    )
  }
  .map_err(|error| error.to_string())?;
  state.ok_or_else(|| "D3D11 created no shared OSC sampler".to_owned())
}

fn create_vertex_buffer(device: &ID3D11Device, capacity: usize) -> Result<ID3D11Buffer, String> {
  let mut buffer = None;
  unsafe {
    device.CreateBuffer(
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
  .map_err(|error| error.to_string())?;
  buffer.ok_or_else(|| "D3D11 created no shared OSC vertex buffer".to_owned())
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

  #[allow(clippy::too_many_arguments)]
  pub(super) fn draw(
    &mut self,
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    viewport_size: (u32, u32),
    frame: Option<[f32; 4]>,
    radius_point: Option<[f32; 2]>,
    crop_image: Option<[f32; 4]>,
    /// The selected layer's corner radius, as a percentage of the crop
    /// rectangle's shorter side, so the shade rounds with the result.
    crop_radius_percent: f64,
    guides: Option<(Option<f32>, Option<f32>, bool, bool)>,
    magnifier_box: Option<[f32; 4]>,
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
    if let Some(frame) = frame {
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
    let index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() };
    let texture = unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(index) }
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
