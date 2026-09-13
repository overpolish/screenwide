// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::PreviewSurfaceRect;
use crate::editor::recording_preview_player::audio_visualizer::AudioRibbonEnvelopes;
use std::ffi::c_void;
use windows::core::Interface;
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D11::{
  ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11PixelShader,
  ID3D11Resource, ID3D11ShaderResourceView, ID3D11Texture2D, ID3D11VertexShader,
  D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BLEND_DESC,
  D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_BUFFER_DESC,
  D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_SUBRESOURCE_DATA,
  D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_IMMUTABLE, D3D11_VIEWPORT,
};
use windows::Win32::Graphics::DirectComposition::{IDCompositionDevice, IDCompositionVisual};
use windows::Win32::Graphics::Dxgi::Common::{
  DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R32_FLOAT,
  DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
  IDXGIFactory2, IDXGISwapChain3, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
  DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
};
const VS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/preview_audio_ribbon_vs.cso"));
const PS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/preview_audio_ribbon_ps.cso"));
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
struct Constants {
  color: [f32; 4],
  flat: [f32; 4],
  geometry: [f32; 4],
  style: [f32; 4],
}
pub(super) struct AudioRibbon {
  blend: ID3D11BlendState,
  constants: ID3D11Buffer,
  context: ID3D11DeviceContext,
  device: ID3D11Device,
  pixels: Option<ID3D11ShaderResourceView>,
  pixel: ID3D11PixelShader,
  swap_chain: IDXGISwapChain3,
  visual: IDCompositionVisual,
  vertex: ID3D11VertexShader,
  viewport: (u32, u32),
  points: u32,
  playhead: f32,
  composition: IDCompositionDevice,
  offset: (f32, f32),
  scale: f32,
  neutral: f32,
  dirty: bool,
  last_constants: Option<Constants>,
}

#[path = "audio_ribbon/draw.rs"]
mod draw;
#[path = "audio_ribbon/pipeline.rs"]
mod pipeline;

impl AudioRibbon {
  pub(super) fn set_viewport(
    &mut self,
    rect: PreviewSurfaceRect,
    scale: f64,
    backdrop: [f64; 4],
  ) -> Result<(), String> {
    let size = (
      (rect.width * scale).round().max(2.0) as u32,
      (rect.height * scale).round().max(2.0) as u32,
    );
    if size != self.viewport {
      unsafe {
        self.swap_chain.ResizeBuffers(
          2,
          size.0,
          size.1,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG(0),
        )
      }
      .map_err(|error| error.to_string())?;
      self.viewport = size;
      self.dirty = true;
    }
    self.offset = ((rect.x * scale) as f32, (rect.y * scale) as f32);
    self.scale = scale.max(0.1) as f32;
    let luminance = backdrop[0] * 0.2126 + backdrop[1] * 0.7152 + backdrop[2] * 0.0722;
    self.neutral = if luminance > 0.5 { 0.0 } else { 1.0 };
    self.draw()?;
    self.sync_visibility()
  }

  pub(super) fn set_envelopes(&mut self, values: &AudioRibbonEnvelopes) -> Result<(), String> {
    let samples = crate::editor::recording_preview_player::audio_visualizer::bucket_levels(values);
    self.points = values.points;
    self.dirty = true;
    if samples.is_empty() {
      self.pixels = None;
      return self.sync_visibility();
    }
    let desc = D3D11_TEXTURE2D_DESC {
      Width: samples.len() as u32,
      Height: 1,
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_R32_FLOAT,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_IMMUTABLE,
      BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
      ..Default::default()
    };
    let data = D3D11_SUBRESOURCE_DATA {
      pSysMem: samples.as_ptr().cast(),
      SysMemPitch: samples.len() as u32 * 4,
      ..Default::default()
    };
    let mut texture = None;
    unsafe {
      self
        .device
        .CreateTexture2D(&desc, Some(&data), Some(&mut texture))
    }
    .map_err(|error| error.to_string())?;
    let texture = texture.ok_or("The audio ribbon texture was not created")?;
    let mut view = None;
    unsafe {
      self
        .device
        .CreateShaderResourceView(&texture, None, Some(&mut view))
    }
    .map_err(|error| error.to_string())?;
    self.pixels = view;
    self.draw()?;
    self.sync_visibility()
  }

  pub(super) fn has_envelopes(&self) -> bool {
    self.pixels.is_some()
  }

  pub(super) fn set_playhead(&mut self, ratio: f64) -> Result<(), String> {
    if self.pixels.is_none() {
      return Ok(());
    }
    self.playhead = ratio.clamp(0.0, 1.0) as f32;
    self.draw()?;
    self.sync_visibility()
  }

  fn sync_visibility(&self) -> Result<(), String> {
    let x = if self.pixels.is_some() {
      self.offset.0
    } else {
      -100000.0
    };
    unsafe {
      self
        .visual
        .SetOffsetX2(x)
        .and_then(|_| self.visual.SetOffsetY2(self.offset.1))
        .and_then(|_| self.composition.Commit())
    }
    .map_err(|error| error.to_string())
  }
}
