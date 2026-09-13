// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn blend_state(
  device: &ID3D11Device,
  source_alpha: windows::Win32::Graphics::Direct3D11::D3D11_BLEND,
) -> Result<ID3D11BlendState, String> {
  let target = D3D11_RENDER_TARGET_BLEND_DESC {
    BlendEnable: true.into(),
    SrcBlend: D3D11_BLEND_SRC_ALPHA,
    DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
    BlendOp: D3D11_BLEND_OP_ADD,
    SrcBlendAlpha: source_alpha,
    DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
    BlendOpAlpha: D3D11_BLEND_OP_ADD,
    RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
  };
  let mut blend = None;
  unsafe {
    device.CreateBlendState(
      &D3D11_BLEND_DESC {
        RenderTarget: [target; 8],
        ..Default::default()
      },
      Some(&mut blend),
    )
  }
  .map_err(|error| error.to_string())?;
  blend.ok_or_else(|| "D3D11 created no region OSC blend state".to_owned())
}

pub(super) fn sampler(
  device: &ID3D11Device,
  filter: windows::Win32::Graphics::Direct3D11::D3D11_FILTER,
) -> Result<ID3D11SamplerState, String> {
  let mut sampler = None;
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
      Some(&mut sampler),
    )
  }
  .map_err(|error| error.to_string())?;
  sampler.ok_or_else(|| "D3D11 created no region OSC sampler".to_owned())
}

pub(super) fn input_elements() -> [D3D11_INPUT_ELEMENT_DESC; 4] {
  [
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("POSITION"),
      SemanticIndex: 0,
      Format: DXGI_FORMAT_R32G32_FLOAT,
      InputSlot: 0,
      AlignedByteOffset: 0,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 0,
      Format: DXGI_FORMAT_R32G32_FLOAT,
      InputSlot: 0,
      AlignedByteOffset: 8,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 1,
      Format: DXGI_FORMAT_R32G32_FLOAT,
      InputSlot: 0,
      AlignedByteOffset: 16,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
    D3D11_INPUT_ELEMENT_DESC {
      SemanticName: s!("TEXCOORD"),
      SemanticIndex: 2,
      Format: DXGI_FORMAT_R32_UINT,
      InputSlot: 0,
      AlignedByteOffset: 24,
      InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
      InstanceDataStepRate: 0,
    },
  ]
}
