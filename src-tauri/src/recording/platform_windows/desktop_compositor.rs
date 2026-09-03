// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GPU composition for Windows regions that cross display boundaries.

use std::{ffi::c_void, mem::size_of, time::Instant};

use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    Direct3D11::{
      ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11PixelShader, ID3D11RenderTargetView,
      ID3D11Resource, ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D,
      ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_RENDER_TARGET,
      D3D11_BIND_SHADER_RESOURCE, D3D11_BUFFER_DESC, D3D11_FILTER_MIN_MAG_MIP_LINEAR,
      D3D11_SAMPLER_DESC, D3D11_TEXTURE2D_DESC, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT,
      D3D11_VIEWPORT,
    },
    Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
  },
};

use crate::desktop_capture::{CapturePiece, CapturePlan, FrameSynchronizer};

use super::writer::{snapshot_frame, Frame};

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/desktop_compositor_vs.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/desktop_compositor_ps.cso"));

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PieceConstants {
  output_size: [u32; 2],
  source_size: [u32; 2],
  source_origin: [u32; 2],
  source_extent: [u32; 2],
  destination_origin: [u32; 2],
  destination_extent: [u32; 2],
}

const _: () = assert!(size_of::<PieceConstants>().is_multiple_of(16));

impl PieceConstants {
  fn new(output: [u32; 2], source: [u32; 2], piece: CapturePiece) -> Self {
    Self {
      output_size: output,
      source_size: source,
      source_origin: [piece.source_pixels.x, piece.source_pixels.y],
      source_extent: [piece.source_pixels.width, piece.source_pixels.height],
      destination_origin: [piece.destination.x, piece.destination.y],
      destination_extent: [piece.destination.width, piece.destination.height],
    }
  }
}

pub(super) struct DesktopFrameCoordinator {
  compositor: DesktopCompositor,
  device: ID3D11Device,
  latest: Vec<Option<Frame>>,
  pieces: Vec<CapturePiece>,
  synchronizer: FrameSynchronizer,
}

impl DesktopFrameCoordinator {
  pub fn new(device: ID3D11Device, plan: &CapturePlan) -> Result<Self, String> {
    Ok(Self {
      compositor: DesktopCompositor::new(&device, plan.width, plan.height)?,
      device,
      latest: vec![None; plan.pieces.len()],
      pieces: plan.pieces.clone(),
      synchronizer: FrameSynchronizer::new(plan.pieces.len())?,
    })
  }

  pub fn update(&mut self, source_index: usize, frame: Frame) -> Result<Option<Frame>, String> {
    let slot = self
      .latest
      .get_mut(source_index)
      .ok_or_else(|| "A frame arrived from an unknown desktop source".to_owned())?;
    let source_100ns = frame.source_100ns;
    let frame = snapshot_frame(&self.device, frame)?;
    *slot = Some(frame);
    let Some(tick) = self.synchronizer.update(source_index, source_100ns)? else {
      return Ok(None);
    };
    let frames = self
      .latest
      .iter()
      .map(|frame| {
        frame
          .as_ref()
          .expect("the synchronizer waits for every desktop source")
      })
      .collect::<Vec<_>>();
    let wall = frames
      .iter()
      .map(|frame| frame.wall)
      .max()
      .unwrap_or_else(Instant::now);
    Ok(Some(Frame {
      source_100ns: tick.output_ns,
      texture: self.compositor.compose(&frames, &self.pieces)?,
      wall,
    }))
  }
}

struct DesktopCompositor {
  constants: ID3D11Buffer,
  context: ID3D11DeviceContext,
  device: ID3D11Device,
  height: u32,
  pixel_shader: ID3D11PixelShader,
  sampler: ID3D11SamplerState,
  vertex_shader: ID3D11VertexShader,
  width: u32,
}

impl DesktopCompositor {
  fn new(device: &ID3D11Device, width: u32, height: u32) -> Result<Self, String> {
    let context = unsafe { device.GetImmediateContext() }.map_err(|error| error.to_string())?;
    let mut vertex_shader = None;
    let mut pixel_shader = None;
    let mut constants = None;
    let mut sampler = None;
    unsafe {
      device
        .CreateVertexShader(VERTEX_SHADER, None, Some(&mut vertex_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreatePixelShader(PIXEL_SHADER, None, Some(&mut pixel_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreateBuffer(
          &D3D11_BUFFER_DESC {
            ByteWidth: size_of::<PieceConstants>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            ..Default::default()
          },
          None,
          Some(&mut constants),
        )
        .map_err(|error| error.to_string())?;
      device
        .CreateSamplerState(
          &D3D11_SAMPLER_DESC {
            Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
            AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
            AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
            MaxLOD: f32::MAX,
            ..Default::default()
          },
          Some(&mut sampler),
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(Self {
      constants: constants.ok_or_else(|| "Direct3D created no desktop constants".to_owned())?,
      context,
      device: device.clone(),
      height,
      pixel_shader: pixel_shader
        .ok_or_else(|| "Direct3D created no desktop pixel shader".to_owned())?,
      sampler: sampler.ok_or_else(|| "Direct3D created no desktop sampler".to_owned())?,
      vertex_shader: vertex_shader
        .ok_or_else(|| "Direct3D created no desktop vertex shader".to_owned())?,
      width,
    })
  }

  fn compose(&self, frames: &[&Frame], pieces: &[CapturePiece]) -> Result<ID3D11Texture2D, String> {
    if frames.len() != pieces.len() {
      return Err("Desktop frames no longer match the capture plan".to_owned());
    }
    let description = D3D11_TEXTURE2D_DESC {
      Width: self.width,
      Height: self.height,
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
      ..Default::default()
    };
    let mut texture = None;
    unsafe {
      self
        .device
        .CreateTexture2D(&description, None, Some(&mut texture))
    }
    .map_err(|error| error.to_string())?;
    let texture = texture.ok_or_else(|| "Direct3D created no desktop canvas".to_owned())?;
    let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let mut target: Option<ID3D11RenderTargetView> = None;
    unsafe {
      self
        .device
        .CreateRenderTargetView(&resource, None, Some(&mut target))
    }
    .map_err(|error| error.to_string())?;
    let target = target.ok_or_else(|| "Direct3D created no desktop target".to_owned())?;
    let constants_resource: ID3D11Resource =
      self.constants.cast().map_err(|error| error.to_string())?;
    unsafe {
      self
        .context
        .ClearRenderTargetView(&target, &[0.0, 0.0, 0.0, 1.0]);
      self.context.OMSetRenderTargets(Some(&[Some(target)]), None);
      self.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: self.width as f32,
        Height: self.height as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      self.context.IASetInputLayout(None);
      self
        .context
        .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      self.context.VSSetShader(&self.vertex_shader, None);
      self.context.PSSetShader(&self.pixel_shader, None);
      self
        .context
        .VSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
      self
        .context
        .PSSetSamplers(0, Some(&[Some(self.sampler.clone())]));
      for (frame, piece) in frames.iter().zip(pieces) {
        let mut source_description = D3D11_TEXTURE2D_DESC::default();
        frame.texture.GetDesc(&mut source_description);
        let source_resource: ID3D11Resource =
          frame.texture.cast().map_err(|error| error.to_string())?;
        let mut source: Option<ID3D11ShaderResourceView> = None;
        self
          .device
          .CreateShaderResourceView(&source_resource, None, Some(&mut source))
          .map_err(|error| error.to_string())?;
        let source = source.ok_or_else(|| "Direct3D created no desktop source view".to_owned())?;
        let constants = PieceConstants::new(
          [self.width, self.height],
          [source_description.Width, source_description.Height],
          *piece,
        );
        self.context.UpdateSubresource(
          &constants_resource,
          0,
          None,
          (&raw const constants).cast::<c_void>(),
          0,
          0,
        );
        self.context.PSSetShaderResources(0, Some(&[Some(source)]));
        self.context.Draw(6, 0);
      }
      self.context.PSSetShaderResources(0, Some(&[None]));
      self.context.OMSetRenderTargets(None, None);
    }
    Ok(texture)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::desktop_capture::PixelRect;

  #[test]
  fn constants_keep_monitor_crop_and_canvas_placement_separate() {
    let constants = PieceConstants::new(
      [1600, 900],
      [3840, 2160],
      CapturePiece {
        display_id: 7,
        source_pixels: PixelRect {
          x: 120,
          y: 80,
          width: 1920,
          height: 1080,
        },
        destination: PixelRect {
          x: 400,
          y: 0,
          width: 1200,
          height: 900,
        },
      },
    );
    assert_eq!(constants.output_size, [1600, 900]);
    assert_eq!(constants.source_size, [3840, 2160]);
    assert_eq!(constants.source_origin, [120, 80]);
    assert_eq!(constants.destination_origin, [400, 0]);
  }
}
