// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shared OSC shader's wgpu pipeline, which every OSC surface draws its
//! vertices through on both platforms: the region OSC, its ruler and OCR
//! chrome, and the editor's selection.

use crate::style::{
  control_palette, ocr_palette, overlay_palette, picked_palette, ruler_palette, turn_palette,
};
use screenwide_gpu::surface::FORMAT;
use screenwide_gpu::{RenderStages, Shader};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/osc.wgsl"));

/// One triangle-list vertex. `position` is already in NDC: the pixel-to-clip
/// mapping happens on the CPU so the vertex shader stays a pass-through. The
/// twin of `ScreenwideRegionOscVertex`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pub position: [f32; 2],
  pub uv: [f32; 2],
  pub kind: u32,
  pub padding: u32,
}

const _: () = assert!(std::mem::size_of::<Vertex>() == 24);
const _: () = assert!(
  crate::shader_layouts::OSC.vertex_buffers[0].stride == std::mem::size_of::<Vertex>() as u64
);

/// The twin of `OscGpu` in `osc.wgsl`: every member is a 16-byte row, so the
/// uniform layout needs no padding.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderConstants {
  pub light_mode: [u32; 4],
  pub magnifier_box: [f32; 4],
  pub action_fills: [[f32; 4]; 2],
  pub control_colors: [[f32; 4]; 2],
  pub ocr_colors: [[f32; 4]; 8],
  pub overlay_shade: [f32; 4],
  pub ruler_colors: [[f32; 4]; 2],
  pub ruler_sample: [f32; 4],
  pub ruler_animation: [f32; 4],
  pub magnifier_source: [f32; 4],
  pub magnifier_sample: [f32; 4],
  pub magnifier_source_range: [f32; 4],
  pub magnifier_flags: [u32; 4],
  /// The rounded chrome plate Windows draws itself: `.x` is its corner
  /// radius. macOS hands that radius to the material surface instead.
  pub chrome: [f32; 4],
  pub chrome_outline: [f32; 4],
  /// Physical viewport size and source texel size for the Windows material
  /// blur sampled from the already-resident frozen desktop texture.
  pub chrome_backdrop: [f32; 4],
  /// The snapshot UV window after Ruler pan/zoom.
  pub chrome_source: [f32; 4],
  /// An object picked in the camera: one kept, one removed.
  pub picked_colors: [[f32; 4]; 2],
  /// The halo round the line painted over the chosen object picked in the
  /// camera: `.x` how far it reaches and `.y` how many points it runs
  /// through, in the target's pixels, two to a row of `picked_points`.
  pub picked_halo: [f32; 4],
  pub picked_points: [[f32; 4]; PICKED_POINT_PAIRS],
  /// The rotation gizmo's X, Y and Z ring colours.
  pub turn_colors: [[f32; 4]; 3],
  /// The rotation gizmo, `TURN_RING_ROWS` rows: each ring's `u` direction,
  /// then each ring's `v`, in the target's pixels with the depth toward the
  /// viewer third, so a ring's point at angle `t` is `cos t * u + sin t * v`
  /// from its middle; then the middle in pixels, the core's half width and
  /// the outline's width; then the ring the pointer holds or `-1`, how far it
  /// is lit toward white, and how strongly far halves show.
  pub turn_rings: [[f32; 4]; TURN_RING_ROWS],
  /// The window being picked, in the target's pixels: x, y, width, height.
  pub window_cutout: [f32; 4],
  /// The window being picked: `.x` its corner radius, `.y` how far the
  /// screen round it has faded in, `.z` how wide the backdrop's blur is, zero
  /// with no backdrop, and `.w` how wide its edge feathers, every length in
  /// the target's pixels.
  pub window_shape: [f32; 4],
  /// A reference's picture and shadow: `.x` how many of the target's pixels
  /// a point spans, which their lengths are given in, and `.y` 1 where the
  /// picture has see-through pixels and casts its shadow from them.
  pub reference: [f32; 4],
}

/// The rows of `RenderConstants::turn_rings`.
pub const TURN_RING_ROWS: usize = 8;

/// How many rows of two points a painted line's halo carries: up to 64.
pub const PICKED_POINT_PAIRS: usize = 32;

const _: () = assert!(std::mem::size_of::<RenderConstants>().is_multiple_of(16));

impl RenderConstants {
  /// The palettes come from the platform-neutral tokens; action fills stay
  /// zero because the OCR controls push their own pair per draw.
  pub fn new(light_mode: bool) -> Self {
    let controls = control_palette(light_mode);
    let ocr = ocr_palette(light_mode);
    let ruler = ruler_palette(light_mode);
    Self {
      light_mode: [u32::from(light_mode), 0, 0, 0],
      magnifier_box: [0.0; 4],
      action_fills: [[0.0; 4]; 2],
      control_colors: [controls.fill, controls.outline],
      ocr_colors: [
        ocr.primary_fill,
        ocr.primary_outline,
        ocr.qr_fill,
        ocr.qr_outline,
        ocr.error_fill,
        ocr.error_outline,
        ocr.selection_fill,
        ocr.selection_outline,
      ],
      overlay_shade: overlay_palette().shade,
      ruler_colors: [ruler.primary, ruler.info],
      ruler_sample: [0.0; 4],
      ruler_animation: [0.0; 4],
      magnifier_source: [0.0; 4],
      magnifier_sample: [0.0; 4],
      magnifier_source_range: [0.0, 0.0, 1.0, 1.0],
      magnifier_flags: [0; 4],
      chrome: [0.0; 4],
      chrome_outline: [0.0; 4],
      chrome_backdrop: [0.0; 4],
      chrome_source: [0.0, 0.0, 1.0, 1.0],
      picked_colors: picked_palette(light_mode),
      picked_halo: [0.0; 4],
      picked_points: [[0.0; 4]; PICKED_POINT_PAIRS],
      turn_colors: turn_palette(light_mode),
      turn_rings: [[0.0; 4]; TURN_RING_ROWS],
      window_cutout: [0.0; 4],
      window_shape: [0.0; 4],
      reference: [0.0; 4],
    }
  }
}

pub fn shader(device: &wgpu::Device) -> Shader {
  Shader::new(
    device,
    "Screenwide OSC",
    &crate::shader_layouts::OSC,
    SHADER,
    crate::precompiled_shaders::OSC,
  )
}

/// Straight source-over: colour is premultiplied through its own alpha, and
/// the alpha channel is laid over the destination's as it is, so the target
/// holds premultiplied colour for the compositor to blend.
pub fn pipeline(shader: &Shader) -> wgpu::RenderPipeline {
  shader.render_pipeline(&RenderStages {
    label: "Screenwide OSC pipeline",
    vertex: "vs_main",
    fragment: "fs_main",
    targets: &[Some(wgpu::ColorTargetState {
      format: FORMAT,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::SrcAlpha,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::One,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
      }),
      write_mask: wgpu::ColorWrites::ALL,
    })],
    // Several OSC primitives are deliberately emitted in either winding
    // (notably line quads), so nothing is culled.
    primitive: wgpu::PrimitiveState {
      cull_mode: None,
      ..Default::default()
    },
  })
}

pub fn sampler(device: &wgpu::Device, filter: wgpu::FilterMode) -> wgpu::Sampler {
  device.create_sampler(&wgpu::SamplerDescriptor {
    label: Some("Screenwide OSC sampler"),
    address_mode_u: wgpu::AddressMode::ClampToEdge,
    address_mode_v: wgpu::AddressMode::ClampToEdge,
    address_mode_w: wgpu::AddressMode::ClampToEdge,
    mag_filter: filter,
    min_filter: filter,
    mipmap_filter: wgpu::MipmapFilterMode::Nearest,
    ..Default::default()
  })
}
