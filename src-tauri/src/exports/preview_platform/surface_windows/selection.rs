// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Transparent D3D11 selection overlay composed above the preview panes.

use std::ffi::c_void;

use windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    Direct3D11::{
      ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11PixelShader, ID3D11RenderTargetView,
      ID3D11Resource, ID3D11SamplerState, ID3D11Texture2D, ID3D11VertexShader,
      D3D11_BIND_CONSTANT_BUFFER, D3D11_BUFFER_DESC, D3D11_FILTER_MIN_MAG_MIP_LINEAR,
      D3D11_SAMPLER_DESC, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT, D3D11_VIEWPORT,
    },
    DirectComposition::{IDCompositionDevice, IDCompositionVisual},
    Dxgi::{
      Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
      IDXGIFactory2, IDXGISwapChain3, DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
      DXGI_SWAP_CHAIN_FLAG, DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
    },
  },
};

#[path = "selection/label.rs"]
pub(super) mod label;
#[path = "selection/label_texture.rs"]
mod label_texture;

use label::{build_label_texture, label_scale_key, LABEL_STROKE};
use label_texture::{upload_label_texture, LabelTexture};

const VERTEX_SHADER: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/recording_selection_vs.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/recording_selection_ps.cso"));

#[repr(C)]
#[derive(Clone, Copy)]
struct Constants {
  frame: [f32; 4],
  viewport: [f32; 4],
  radius_control: [f32; 4],
  guides: [f32; 4],
  crop_image: [f32; 4],
  magnifier_box: [f32; 4],
  label: [f32; 4],
  secondary_label: [f32; 4],
  label_params: [f32; 4],
  action_shades: [f32; 4],
}

fn split_action_label_rects(
  frame: [f32; 4],
  primary: (u32, u32),
  secondary: (u32, u32),
  viewport: (f32, f32),
  scale: f32,
) -> ([f32; 4], [f32; 4]) {
  let padding_x = 6.0 * scale;
  let padding_y = 4.0 * scale;
  let gap = 4.0 * scale;
  let primary = (primary.0 as f32, primary.1 as f32);
  let secondary = (secondary.0 as f32, secondary.1 as f32);
  let primary_button = primary.0 + padding_x * 2.0;
  let secondary_button = secondary.0 + padding_x * 2.0;
  let total_width = primary_button + gap + secondary_button;
  let button_height = primary.1.max(secondary.1) + padding_y * 2.0;
  let x = (frame[0] + (frame[2] - total_width) * 0.5)
    .clamp(0.0, (viewport.0 - total_width).max(0.0))
    .floor();
  let mut y = frame[1] + frame[3] + 6.0 * scale;
  if y + button_height > viewport.1 {
    y = frame[1] - 6.0 * scale - button_height;
  }
  y = y.clamp(0.0, (viewport.1 - button_height).max(0.0)).floor();
  (
    [x + padding_x, y + padding_y, primary.0, primary.1],
    [
      x + primary_button + gap + padding_x,
      y + padding_y,
      secondary.0,
      secondary.1,
    ],
  )
}

pub(super) struct SelectionOverlay {
  pub(super) action: super::osc_action::OscAction,
  buffer_size: (u32, u32),
  constants: ID3D11Buffer,
  label: Option<LabelTexture>,
  secondary_label: Option<LabelTexture>,
  /// Bound whenever there is no label, so the pixel shader's texture slot is
  /// always filled with a real (transparent 1x1) view.
  label_placeholder: LabelTexture,
  label_sampler: ID3D11SamplerState,
  pixel_shader: ID3D11PixelShader,
  swap_chain: IDXGISwapChain3,
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
    unsafe {
      device
        .CreateVertexShader(VERTEX_SHADER, None, Some(&mut vertex_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreatePixelShader(PIXEL_SHADER, None, Some(&mut pixel_shader))
        .map_err(|error| error.to_string())?;
    }
    let mut constants = None;
    unsafe {
      device
        .CreateBuffer(
          &D3D11_BUFFER_DESC {
            ByteWidth: size_of::<Constants>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            ..Default::default()
          },
          None,
          Some(&mut constants),
        )
        .map_err(|error| error.to_string())?;
    }
    let sampler_description = D3D11_SAMPLER_DESC {
      Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
      AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
      AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
      AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
      MaxLOD: f32::MAX,
      ..Default::default()
    };
    let mut label_sampler = None;
    unsafe { device.CreateSamplerState(&sampler_description, Some(&mut label_sampler)) }
      .map_err(|error| error.to_string())?;
    let label_placeholder = upload_label_texture(device, &[0u8; 4], (1, 1), "", 0, false)?;
    Ok(Self {
      action: super::osc_action::OscAction::default(),
      buffer_size: (2, 2),
      constants: constants.ok_or_else(|| "D3D11 created no selection constants".to_owned())?,
      label: None,
      secondary_label: None,
      label_placeholder,
      label_sampler: label_sampler
        .ok_or_else(|| "D3D11 created no selection label sampler".to_owned())?,
      pixel_shader: pixel_shader
        .ok_or_else(|| "D3D11 created no selection pixel shader".to_owned())?,
      swap_chain,
      vertex_shader: vertex_shader
        .ok_or_else(|| "D3D11 created no selection vertex shader".to_owned())?,
      _visual: visual,
    })
  }

  /// Returns the label texture for `text` at `scale`, building it only when
  /// either changed since the last draw. `None` when GDI could not rasterise
  /// it, in which case no label is drawn.
  fn label_texture(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
    action: bool,
  ) -> Option<&LabelTexture> {
    let key = label_scale_key(scale);
    let stale = self
      .label
      .as_ref()
      .is_none_or(|label| label.scale_key != key || label.text != text || label.action != action);
    if stale {
      self.label = build_label_texture(device, text, scale, action)
        .inspect_err(|error| eprintln!("The selection size readout could not be drawn: {error}"))
        .ok();
    }
    self.label.as_ref()
  }

  fn secondary_label_texture(
    &mut self,
    device: &ID3D11Device,
    text: &str,
    scale: f64,
  ) -> Option<&LabelTexture> {
    let key = label_scale_key(scale);
    let stale = self
      .secondary_label
      .as_ref()
      .is_none_or(|label| label.scale_key != key || label.text != text || !label.action);
    if stale {
      self.secondary_label = build_label_texture(device, text, scale, true)
        .inspect_err(|error| eprintln!("The secondary OSC action could not be drawn: {error}"))
        .ok();
    }
    self.secondary_label.as_ref()
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
    guides: Option<(Option<f32>, Option<f32>, bool, bool)>,
    magnifier_box: Option<[f32; 4]>,
    label_text: Option<&str>,
    label_action: bool,
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
    // The readout hangs 4pt below the box, trailing edge flush with the box's
    // right edge (Keyframeless's placement), flips above the box when it would
    // run off the bottom, and is clamped into the viewport. Everything here is
    // already in physical pixels, so snapping to the pixel grid is a floor:
    // the glyphs then land on the grid they were rasterised on and stay crisp.
    let split_actions = label_action && label_text.is_some_and(|text| text.starts_with("Reset"));
    let split_label = if let Some(frame) = frame.filter(|_| split_actions) {
      let primary = self
        .label_texture(device, "Reset", scale, true)
        .map(|label| (label.view.clone(), label.size));
      let secondary = self
        .secondary_label_texture(device, "Apply to all", scale)
        .map(|label| (label.view.clone(), label.size));
      primary.zip(secondary).map(|(primary, secondary)| {
        let (primary_rect, secondary_rect) = split_action_label_rects(
          frame,
          primary.1,
          secondary.1,
          (size.0 as f32, size.1 as f32),
          scale as f32,
        );
        (primary.0, primary_rect, secondary.0, secondary_rect)
      })
    } else {
      None
    };
    let label = match (frame, label_text.filter(|_| !split_actions)) {
      (Some(frame), Some(text)) => self
        .label_texture(device, text, scale, label_action)
        .map(|label| (label.view.clone(), label.size))
        .map(|(view, (width, height))| {
          let width = width as f32;
          let height = height as f32;
          // Keep the visible button 6pt from the frame after its 4pt top
          // padding is included, matching the previous OSC placement.
          let gap = ((if label_action { 10.0 } else { 4.0 }) * scale) as f32;
          let (x, y) = super::recenter::label_origin(
            frame,
            (width, height),
            (size.0 as f32, size.1 as f32),
            gap,
            label_action,
          );
          (view, [x.floor(), y.floor(), width, height])
        }),
      _ => None,
    };
    let (label_view, label_rect, secondary_label_view, secondary_label_rect) = match split_label {
      Some((primary, primary_rect, secondary, secondary_rect)) => {
        (primary, primary_rect, secondary, secondary_rect)
      }
      None => {
        let (view, rect) = label.map_or(
          (self.label_placeholder.view.clone(), [0.0; 4]),
          |(view, rect)| (view, rect),
        );
        (view, rect, self.label_placeholder.view.clone(), [0.0; 4])
      }
    };
    let action_shades = self.action.layout(
      label_rect,
      (secondary_label_rect[2] > 0.0).then_some(secondary_label_rect),
      scale as f32,
      label_action,
    );
    let values = Constants {
      frame: frame.unwrap_or_default(),
      viewport: [
        size.0 as f32,
        size.1 as f32,
        u32::from(light) as f32,
        f32::from(frame.is_some()),
      ],
      radius_control: radius_point.map_or([0.0; 4], |point| [point[0], point[1], 1.0, 0.0]),
      guides: guides.map_or([-1.0, -1.0, 0.0, 0.0], |(x, y, x_object, y_object)| {
        [
          x.unwrap_or(-1.0),
          y.unwrap_or(-1.0),
          if x_object { 1.0 } else { 0.0 },
          if y_object { 1.0 } else { 0.0 },
        ]
      }),
      crop_image: crop_image.unwrap_or([-1.0; 4]),
      magnifier_box: magnifier_box.unwrap_or_default(),
      label: label_rect,
      secondary_label: secondary_label_rect,
      // The stroke is centred on the glyph outline, so half of it spills out;
      // GDI glyphs are lighter than CoreText's, and the ring-mean halo reaches
      // a touch past its radius, so it is trimmed to keep the same weight.
      label_params: [
        (LABEL_STROKE * 0.5 * 0.75 * scale) as f32,
        scale as f32,
        f32::from(label_action),
        f32::from(split_actions),
      ],
      action_shades,
    };
    let constants: ID3D11Resource = self.constants.cast().map_err(|error| error.to_string())?;
    unsafe {
      context.UpdateSubresource(
        &constants,
        0,
        None,
        (&raw const values).cast::<c_void>(),
        0,
        0,
      )
    };
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
      context.RSSetViewports(Some(&[D3D11_VIEWPORT {
        Width: size.0 as f32,
        Height: size.1 as f32,
        MaxDepth: 1.0,
        ..Default::default()
      }]));
      context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
      context.VSSetShader(&self.vertex_shader, None);
      context.PSSetShader(&self.pixel_shader, None);
      context.PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
      context.PSSetShaderResources(0, Some(&[Some(label_view), Some(secondary_label_view)]));
      context.PSSetSamplers(0, Some(&[Some(self.label_sampler.clone())]));
      context.Draw(3, 0);
      context.PSSetShaderResources(0, Some(&[None, None]));
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
