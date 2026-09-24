// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One-pass D3D11 preview compositor. Decoded frames remain on the shared GPU;
//! the CPU only updates this pass's small constant buffer.

#[path = "compositor/cursor_artwork.rs"]
mod cursor_artwork;
#[path = "compositor/draw.rs"]
mod draw;
#[path = "compositor/pipeline.rs"]
mod pipeline;
#[path = "compositor/source.rs"]
mod source;
#[path = "compositor/submit.rs"]
mod submit;
use cursor_artwork::native_cursor_pixels;

use std::{ffi::c_void, path::PathBuf};

use windows::{
  core::{w, Interface, PCWSTR},
  Win32::Foundation::ERROR_SUCCESS,
  Win32::Graphics::{
    Direct3D::{D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D_SRV_DIMENSION_BUFFER},
    Direct3D11::{
      ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11PixelShader,
      ID3D11RenderTargetView, ID3D11Resource, ID3D11SamplerState, ID3D11ShaderResourceView,
      ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE,
      D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
      D3D11_BUFFER_DESC, D3D11_BUFFER_SRV, D3D11_BUFFER_SRV_0, D3D11_BUFFER_SRV_1,
      D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_CPU_ACCESS_WRITE, D3D11_FILTER_MIN_MAG_MIP_LINEAR,
      D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE_DISCARD, D3D11_RENDER_TARGET_BLEND_DESC,
      D3D11_RESOURCE_MISC_BUFFER_STRUCTURED, D3D11_SAMPLER_DESC, D3D11_SHADER_RESOURCE_VIEW_DESC,
      D3D11_SHADER_RESOURCE_VIEW_DESC_0, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
      D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_USAGE_IMMUTABLE,
      D3D11_VIEWPORT,
    },
    Dxgi::Common::{
      DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC,
    },
    Gdi::{
      CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
      BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    },
  },
  Win32::System::{
    Environment::ExpandEnvironmentStringsW,
    Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ, RRF_ZEROONFAILURE},
  },
  Win32::UI::WindowsAndMessaging::{
    DestroyCursor, DrawIconEx, GetIconInfo, LoadImageW, DI_NORMAL, HCURSOR, IDC_ARROW, IDC_CROSS,
    IDC_HAND, IDC_IBEAM, IDC_NO, IDC_SIZENS, IDC_SIZEWE, IMAGE_CURSOR, LR_LOADFROMFILE, LR_SHARED,
  },
};

use super::background_image::BackgroundImageCache;
use super::counter_artwork::CounterAtlas;
use super::keyboard_artwork::{KeyboardArtworkCache, KeyboardConstants};
use crate::editor::annotations::geometry::{ArrowGeometry, ArrowTriangle};
use crate::editor::keyboard_effects::KeyboardOverlay;
use crate::editor::media_preview::BakeGeometry;
use crate::screenshots::{
  colour_f32, foreground_bounds_f32, generator_palette, mesh_generator, optional_colour_f32,
  output_placement, validate_mesh, ScreenshotOutputSettings,
};

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/recording_preview_vs.cso"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/recording_preview_ps.cso"));
#[repr(C)]
#[derive(Clone, Copy)]
struct Constants {
  output_source: [f32; 4],
  image_rect: [f32; 4],
  crop_rect: [f32; 4],
  source_crop_rect: [f32; 4],
  /// The crop tool's result layer: its output-pixel rectangle, then its
  /// radius, an enabled flag, the shadow sigma and one spare word.
  crop_preview_rect: [f32; 4],
  crop_preview_effects: [f32; 4],
  solid_color: [f32; 4],
  base_color: [f32; 4],
  recenter_inset_color: [f32; 4],
  mesh_points: [[f32; 4]; 8],
  mesh_colors: [[f32; 4]; 4],
  effects: [f32; 4],
  /// Timeline seconds, the generator speed the ported generators scale them
  /// by, how many canvas pixels one drawn pixel covers, and how many atlas
  /// pixels the annotations' type holds per canvas pixel.
  motion: [f32; 4],
  cursor_geometry: [f32; 4],
  cursor_effects: [f32; 4],
  cursor_blur: [f32; 4],
  camera_frame: [f32; 4],
  camera_crop: [f32; 4],
  camera_effects: [f32; 4],
  magnifier: [f32; 4],
  magnifier_options: [f32; 4],
  magnifier_bounds: [f32; 4],
  native_cursor_hotspots: [[f32; 4]; 8],
  options: [u32; 4],
  cursor_options: [u32; 4],
  background_options: [u32; 4],
  /// Annotations below the camera are sorted ahead of those above it, so the
  /// first word is both the below-camera count and where the above-camera run
  /// starts; the second is the total. The rest are spare.
  annotation_options: [u32; 4],
}

/// The structured-buffer elements the arrow shader reads, and the buffers
/// that carry them.
#[path = "compositor/arrows.rs"]
mod arrows;
pub(crate) use arrows::{
  PreparedArrows, PreparedType, PreviewArrow, PreviewSample, StructuredBuffer, MAX_EXPOSURE_SAMPLES,
};

pub(super) struct Compositor {
  background_cache: BackgroundImageCache,
  /// Prepared arrows, written per draw.
  annotations: StructuredBuffer,
  /// Exposure samples for moving annotations, written per draw beside the
  /// arrows.
  samples: StructuredBuffer,
  annotation_points: StructuredBuffer,
  annotation_text: StructuredBuffer,
  constants: ID3D11Buffer,
  cursor_hotspots: [[f32; 4]; 8],
  cursor_view: ID3D11ShaderResourceView,
  counter_atlas: CounterAtlas,
  keyboard_cache: KeyboardArtworkCache,
  keyboard_constants: ID3D11Buffer,
  /// Bound at t3 when no shortcut is on screen and at t4 when the canvas has
  /// no background picture, mirroring the four-byte fallback buffer the Metal
  /// compositor binds.
  fallback_view: ID3D11ShaderResourceView,
  layer_blend: ID3D11BlendState,
  pixel_shader: ID3D11PixelShader,
  sampler: ID3D11SamplerState,
  point_sampler: ID3D11SamplerState,
  vertex_shader: ID3D11VertexShader,
}

#[derive(Clone)]
pub(super) struct SourceTexture {
  pub(super) size: (u32, u32),
  texture: ID3D11Texture2D,
  view: ID3D11ShaderResourceView,
}

impl Compositor {
  pub(super) fn keyboard_visible_bounds(
    &self,
    device: &ID3D11Device,
    overlay: &KeyboardOverlay,
    output: (u32, u32),
  ) -> Result<Option<[f64; 4]>, String> {
    self.keyboard_cache.visible_bounds(device, overlay, output)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn preview_shader_is_embedded_as_compiled_bytecode() {
    assert_eq!(&VERTEX_SHADER[..4], b"DXBC");
    assert_eq!(&PIXEL_SHADER[..4], b"DXBC");
  }
}

#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/crop_preview_tests.rs"]
mod crop_preview_tests;
#[cfg(all(test, target_os = "windows", target_arch = "x86_64"))]
#[path = "compositor/fpu_tests.rs"]
mod fpu_tests;
#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/render_test_helpers.rs"]
mod render_test_helpers;
#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/render_tests.rs"]
mod render_tests;
#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/selection_state_tests.rs"]
mod selection_state_tests;
#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/swapchain_tests.rs"]
mod swapchain_tests;
