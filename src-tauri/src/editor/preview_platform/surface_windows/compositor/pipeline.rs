// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Compositor {
  pub(in crate::editor::preview_platform::surface) fn new(
    device: &ID3D11Device,
  ) -> Result<Self, String> {
    let mut vertex_shader = None;
    let mut pixel_shader = None;
    let mut blur_vertex_shader = None;
    let mut blur_pixel_shader = None;
    unsafe {
      device
        .CreateVertexShader(VERTEX_SHADER, None, Some(&mut vertex_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreatePixelShader(PIXEL_SHADER, None, Some(&mut pixel_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreateVertexShader(BLUR_VERTEX_SHADER, None, Some(&mut blur_vertex_shader))
        .map_err(|error| error.to_string())?;
      device
        .CreatePixelShader(BLUR_PIXEL_SHADER, None, Some(&mut blur_pixel_shader))
        .map_err(|error| error.to_string())?;
    }
    let mut blur_constants = None;
    unsafe {
      device.CreateBuffer(
        &D3D11_BUFFER_DESC {
          ByteWidth: size_of::<BlurConstants>() as u32,
          Usage: D3D11_USAGE_DEFAULT,
          BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
          ..Default::default()
        },
        None,
        Some(&mut blur_constants),
      )
    }
    .map_err(|error| error.to_string())?;
    let description = D3D11_BUFFER_DESC {
      ByteWidth: size_of::<Constants>() as u32,
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
      ..Default::default()
    };
    let mut constants = None;
    unsafe { device.CreateBuffer(&description, None, Some(&mut constants)) }
      .map_err(|error| error.to_string())?;
    let mut keyboard_constants = None;
    unsafe {
      device.CreateBuffer(
        &D3D11_BUFFER_DESC {
          ByteWidth: size_of::<KeyboardConstants>() as u32,
          Usage: D3D11_USAGE_DEFAULT,
          BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
          ..Default::default()
        },
        None,
        Some(&mut keyboard_constants),
      )
    }
    .map_err(|error| error.to_string())?;
    let mut keyboard_fallback_texture = None;
    unsafe {
      device.CreateTexture2D(
        &D3D11_TEXTURE2D_DESC {
          Width: 1,
          Height: 1,
          MipLevels: 1,
          ArraySize: 1,
          Format: DXGI_FORMAT_B8G8R8A8_UNORM,
          SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
          },
          Usage: D3D11_USAGE_IMMUTABLE,
          BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
          ..Default::default()
        },
        Some(&D3D11_SUBRESOURCE_DATA {
          pSysMem: [0_u8; 4].as_ptr().cast::<c_void>(),
          SysMemPitch: 4,
          SysMemSlicePitch: 0,
        }),
        Some(&mut keyboard_fallback_texture),
      )
    }
    .map_err(|error| error.to_string())?;
    let keyboard_fallback_texture = keyboard_fallback_texture
      .ok_or_else(|| "D3D11 created no keyboard fallback texture".to_owned())?;
    let keyboard_fallback_resource: ID3D11Resource = keyboard_fallback_texture
      .cast()
      .map_err(|error| error.to_string())?;
    let mut keyboard_fallback = None;
    unsafe {
      device.CreateShaderResourceView(
        &keyboard_fallback_resource,
        None,
        Some(&mut keyboard_fallback),
      )
    }
    .map_err(|error| error.to_string())?;
    let sampler_description = D3D11_SAMPLER_DESC {
      Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
      AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
      AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
      AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
      MaxLOD: f32::MAX,
      ..Default::default()
    };
    let mut sampler = None;
    unsafe { device.CreateSamplerState(&sampler_description, Some(&mut sampler)) }
      .map_err(|error| error.to_string())?;
    let layer_target = D3D11_RENDER_TARGET_BLEND_DESC {
      BlendEnable: true.into(),
      SrcBlend: D3D11_BLEND_ONE,
      DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
      BlendOp: D3D11_BLEND_OP_ADD,
      SrcBlendAlpha: D3D11_BLEND_ONE,
      DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
      BlendOpAlpha: D3D11_BLEND_OP_ADD,
      RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
    };
    let mut layer_blend = None;
    unsafe {
      device.CreateBlendState(
        &D3D11_BLEND_DESC {
          RenderTarget: [layer_target; 8],
          ..Default::default()
        },
        Some(&mut layer_blend),
      )
    }
    .map_err(|error| error.to_string())?;
    let point_description = D3D11_SAMPLER_DESC {
      Filter: windows::Win32::Graphics::Direct3D11::D3D11_FILTER_MIN_MAG_MIP_POINT,
      AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
      AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
      AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
      MaxLOD: f32::MAX,
      ..Default::default()
    };
    let mut point_sampler = None;
    unsafe { device.CreateSamplerState(&point_description, Some(&mut point_sampler)) }
      .map_err(|error| error.to_string())?;
    let cursors = [
      (IDC_ARROW, "Arrow", "aero_arrow.cur"),
      (IDC_IBEAM, "IBeam", "beam_r.cur"),
      (IDC_IBEAM, "IBeam", "beam_r.cur"),
      (IDC_SIZEWE, "SizeWE", "aero_ew.cur"),
      (IDC_SIZENS, "SizeNS", "aero_ns.cur"),
      (IDC_HAND, "Hand", "aero_link.cur"),
      (IDC_CROSS, "Crosshair", "cross_r.cur"),
      (IDC_NO, "No", "aero_unavail.cur"),
    ];
    let cursor_pixels = cursors
      .into_iter()
      .map(|(cursor, scheme_value, fallback_file)| {
        native_cursor_pixels(cursor, scheme_value, fallback_file)
      })
      .collect::<Result<Vec<_>, _>>()?;
    let (cursor_width, cursor_height, _, _) = cursor_pixels[0];
    if cursor_pixels
      .iter()
      .any(|(width, height, _, _)| (*width, *height) != (cursor_width, cursor_height))
    {
      return Err("Windows standard cursors do not share one bitmap size".to_owned());
    }
    let cursor_description = D3D11_TEXTURE2D_DESC {
      Width: cursor_width,
      Height: cursor_height,
      MipLevels: 1,
      ArraySize: cursor_pixels.len() as u32,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
      ..Default::default()
    };
    let initial_data = cursor_pixels
      .iter()
      .map(|(_, _, _, pixels)| D3D11_SUBRESOURCE_DATA {
        pSysMem: pixels.as_ptr().cast(),
        SysMemPitch: cursor_width * 4,
        ..Default::default()
      })
      .collect::<Vec<_>>();
    let mut cursor_texture = None;
    unsafe {
      device.CreateTexture2D(
        &cursor_description,
        Some(initial_data.as_ptr()),
        Some(&mut cursor_texture),
      )
    }
    .map_err(|error| error.to_string())?;
    let cursor_texture =
      cursor_texture.ok_or_else(|| "D3D11 created no native cursor texture atlas".to_owned())?;
    let cursor_resource: ID3D11Resource =
      cursor_texture.cast().map_err(|error| error.to_string())?;
    let mut cursor_view = None;
    unsafe { device.CreateShaderResourceView(&cursor_resource, None, Some(&mut cursor_view)) }
      .map_err(|error| error.to_string())?;
    let cursor_hotspots = std::array::from_fn(|index| {
      let hotspot = cursor_pixels[index].2;
      [hotspot[0], hotspot[1], 0.0, 0.0]
    });
    Ok(Self {
      background_cache: BackgroundImageCache::default(),
      blur_constants: blur_constants
        .ok_or_else(|| "D3D11 created no preview blur constant buffer".to_owned())?,
      blur_pixel_shader: blur_pixel_shader
        .ok_or_else(|| "D3D11 created no preview blur pixel shader".to_owned())?,
      blur_vertex_shader: blur_vertex_shader
        .ok_or_else(|| "D3D11 created no preview blur vertex shader".to_owned())?,
      constants: constants.ok_or_else(|| "D3D11 created no preview constant buffer".to_owned())?,
      cursor_hotspots,
      cursor_view: cursor_view
        .ok_or_else(|| "D3D11 created no native cursor atlas view".to_owned())?,
      keyboard_cache: KeyboardArtworkCache::default(),
      keyboard_constants: keyboard_constants
        .ok_or_else(|| "D3D11 created no keyboard constant buffer".to_owned())?,
      fallback_view: keyboard_fallback
        .ok_or_else(|| "D3D11 created no keyboard fallback view".to_owned())?,
      layer_blend: layer_blend
        .ok_or_else(|| "D3D11 created no screenshot layer blend state".to_owned())?,
      pixel_shader: pixel_shader
        .ok_or_else(|| "D3D11 created no preview pixel shader".to_owned())?,
      sampler: sampler.ok_or_else(|| "D3D11 created no preview sampler".to_owned())?,
      point_sampler: point_sampler
        .ok_or_else(|| "D3D11 created no preview point sampler".to_owned())?,
      vertex_shader: vertex_shader
        .ok_or_else(|| "D3D11 created no preview vertex shader".to_owned())?,
    })
  }
}
