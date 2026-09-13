// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn input_elements() -> [D3D11_INPUT_ELEMENT_DESC; 4] {
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

pub(super) fn blend_state(device: &ID3D11Device) -> Result<ID3D11BlendState, String> {
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

pub(super) fn rasterizer_state(device: &ID3D11Device) -> Result<ID3D11RasterizerState, String> {
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

pub(super) fn sampler(
  device: &ID3D11Device,
  filter: D3D11_FILTER,
) -> Result<ID3D11SamplerState, String> {
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

pub(super) fn create_vertex_buffer(
  device: &ID3D11Device,
  capacity: usize,
) -> Result<ID3D11Buffer, String> {
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
