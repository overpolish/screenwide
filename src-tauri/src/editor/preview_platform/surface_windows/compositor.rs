// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One-pass preview compositor on the shared wgpu device. Decoded frames stay
//! on the GPU: Media Foundation's textures are copied into a texture both
//! devices open, and the CPU only updates this pass's uniforms and lists.

#[path = "compositor/cursor_artwork.rs"]
mod cursor_artwork;
#[path = "compositor/draw.rs"]
mod draw;
#[path = "compositor/pipeline.rs"]
mod pipeline;
#[path = "compositor/redact.rs"]
mod redact;
#[path = "compositor/redact_pipeline.rs"]
mod redact_pipeline;
#[path = "compositor/redact_targets.rs"]
mod redact_targets;
#[path = "compositor/source.rs"]
mod source;
#[path = "compositor/submit.rs"]
mod submit;
use cursor_artwork::native_cursor_pixels;

use std::path::PathBuf;

use windows::{
  core::{w, PCWSTR},
  Win32::Foundation::ERROR_SUCCESS,
  Win32::Graphics::{
    Direct3D11::ID3D11Texture2D,
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
use crate::gpu::{D3d11Layer, Gpu};
use crate::screenshots::{
  colour_f32, foreground_bounds_f32, generator_palette, mesh_generator, optional_colour_f32,
  output_placement, validate_mesh, ScreenshotOutputSettings,
};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/preview.wgsl"));

/// Everything the canvas pass composes into and from, as one format.
pub(super) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// The twin of `Canvas` in `preview.wgsl`; every member is a 16-byte row.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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

/// The storage-buffer elements the annotation shader reads, and the buffers
/// that carry them.
#[path = "compositor/arrows.rs"]
mod arrows;
pub(crate) use arrows::{
  GpuBuffer, PreparedArrows, PreparedType, PreviewArrow, PreviewSample, MAX_EXPOSURE_SAMPLES,
};

pub(super) struct Compositor {
  gpu: &'static Gpu,
  background_cache: BackgroundImageCache,
  /// Prepared annotations, written per draw.
  annotations: GpuBuffer,
  /// Exposure samples for moving annotations, written per draw beside them.
  samples: GpuBuffer,
  annotation_points: GpuBuffer,
  annotation_text: GpuBuffer,
  constants: wgpu::Buffer,
  keyboard_constants: wgpu::Buffer,
  cursor_hotspots: [[f32; 4]; 8],
  cursor_view: wgpu::TextureView,
  counter_atlas: CounterAtlas,
  keyboard_cache: KeyboardArtworkCache,
  /// Bound where no shortcut is on screen, no background picture is chosen,
  /// no camera is composed or no type was rasterised: one transparent texel.
  fallback_view: wgpu::TextureView,
  layout: wgpu::BindGroupLayout,
  /// The canvas written whole.
  pipeline: wgpu::RenderPipeline,
  /// A screenshot layer blended, premultiplied, over the layers under it.
  layer_pipeline: wgpu::RenderPipeline,
  redactor: redact::Redactor,
  sampler: wgpu::Sampler,
  point_sampler: wgpu::Sampler,
}

/// A frame the canvas samples. A decoded frame lives in a texture the
/// Direct3D 11 device opens too, which the decoder's output is copied into; a
/// screenshot is uploaded once.
#[derive(Clone)]
pub(super) struct SourceTexture {
  pub(super) size: (u32, u32),
  texture: wgpu::Texture,
  view: wgpu::TextureView,
  shared: Option<std::sync::Arc<crate::gpu::SharedTexture>>,
  /// A screenshot's own pixels, which its redactions read their fills from.
  /// A video frame has none: its fills are read from its clip's frames.
  pub(super) picture: Option<std::sync::Arc<crate::screenshots::CapturedImage>>,
}

impl Compositor {
  pub(super) fn keyboard_visible_bounds(
    &self,
    overlay: &KeyboardOverlay,
    output: (u32, u32),
  ) -> Result<Option<[f64; 4]>, String> {
    self
      .keyboard_cache
      .visible_bounds(self.gpu, overlay, output)
  }
}

#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/crop_preview_tests.rs"]
mod crop_preview_tests;
#[cfg(all(test, target_os = "windows", target_arch = "x86_64"))]
#[path = "compositor/fpu_tests.rs"]
mod fpu_tests;
#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/redact_tests.rs"]
mod redact_tests;
#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/render_test_helpers.rs"]
mod render_test_helpers;
#[cfg(all(test, target_os = "windows"))]
#[path = "compositor/render_tests.rs"]
mod render_tests;
