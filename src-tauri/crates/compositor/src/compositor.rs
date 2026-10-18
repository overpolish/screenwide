// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One-pass canvas compositor on the shared wgpu device: the background, the
//! picture, the camera, the annotations, the keyboard overlay and the cursor,
//! drawn by `preview.wgsl` into whatever target the caller hands it. The
//! platform brings the frames in and takes the canvas out.

mod background;
mod camera;
mod canvas_features;
#[cfg(test)]
mod canvas_features_tests;
mod canvas_modules;
mod canvas_pipelines;
mod canvas_precompiled;
mod canvas_variants;
/// A caption style's preview, drawn as the export draws it.
pub mod caption_preview;
mod constants;
mod cursor_artwork;
pub mod cursor_placement;
/// Sources fed from Direct3D 11 textures: capture, the camera and Media
/// Foundation hand frames over through the shared D3D11 layer.
#[cfg(target_os = "windows")]
mod d3d11_source;
mod draw;
mod focus;
mod layer;
mod mark_blur;
pub mod mips;
mod pipeline;
mod preview_bindings;
mod redact;
mod redact_pipeline;
mod redact_targets;
mod scene_motion;
mod scene_turn;
mod source;
pub mod still;
pub mod still_overlay;
mod submit;
mod tiles;
#[cfg(target_os = "macos")]
pub mod video_planes;
mod whole_blur;

pub use camera::CameraComposition;
pub use canvas_pipelines::EveryAnnotationKind;
pub use layer::{CanvasGeometry, LayerDraw, LayerPlacement};

use self::constants::Constants;

use super::background_image::BackgroundImageCache;
use super::keyboard_artwork::{KeyboardArtworkCache, KeyboardConstants};
use crate::annotation_gpu::{
  numbered_arrows, place_images, CounterAtlas, GpuBuffer, ImageAtlas, PreparedArrows,
};
use screenwide_gpu::Gpu;
use screenwide_keyboard_effects::KeyboardOverlay;
use screenwide_output::{
  colour_f32, foreground_bounds_f32, generator_palette, mesh_generator, optional_colour_f32,
  output_placement, validate_mesh, ScreenshotOutputSettings,
};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/preview.wgsl"));

/// Everything the canvas pass composes into and from, as one format.
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// What one composition draws besides the picture and the annotations.
#[derive(Clone, Copy)]
pub struct ComposedFrame {
  pub cursor: Option<screenwide_cursor_effects::GpuCursor>,
  pub keyboard: Option<screenwide_keyboard_effects::KeyboardOverlay>,
  /// A layer laid over the layers under it: no background, and blended
  /// over what the target already holds.
  pub foreground_only: bool,
  pub seconds: f64,
}

/// The crop tool's loupe: where it sits, what it shows and how.
#[derive(Clone, Copy)]
pub struct CropMagnifier {
  pub bounds: [f32; 4],
  /// Where the loupe is drawn in the window, which only the Windows preview
  /// places an overlay at.
  #[cfg_attr(target_os = "macos", allow(dead_code))]
  pub display_box: [f32; 4],
  pub geometry: [f32; 4],
  pub options: [f32; 4],
}

/// What the shader needs of one uploaded style, by style index.
#[derive(Clone, Copy)]
struct ArtworkStyle {
  size: (u32, u32),
  design: (f32, f32),
  origin: (f32, f32),
  use_design: bool,
  clip_local_box: bool,
  supersample: bool,
}

pub struct Compositor {
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
  /// Every cursor style's artwork, one layer each, and how each is drawn.
  cursor_view: wgpu::TextureView,
  cursor_artworks: Vec<ArtworkStyle>,
  counter_atlas: CounterAtlas,
  image_atlas: ImageAtlas,
  keyboard_cache: KeyboardArtworkCache,
  /// Bound where no shortcut is on screen, no background picture is chosen,
  /// no camera is composed, no type was rasterised or no image drawn: one
  /// transparent texel.
  fallback_view: wgpu::TextureView,
  layout: wgpu::BindGroupLayout,
  tiles: tiles::AnnotationTiles,
  /// Shared by every compositor on the device; see `canvas_pipelines`.
  canvas: std::sync::Arc<canvas_pipelines::CanvasPipelines>,
  /// Set while an export holds it: annotated draws take the canvas variant
  /// with every kind; see `canvas_pipelines`.
  every_annotation_kind: std::sync::atomic::AtomicBool,
  redactor: redact::Redactor,
  /// The pictures baked cameras are composed into, one for each slot.
  camera_canvases: std::sync::Mutex<std::collections::HashMap<usize, SourceTexture>>,
  mark_blur: mark_blur::MarkBlur,
  mips: mips::Mips,
  focus: focus::FocusBlur,
  sampler: wgpu::Sampler,
  point_sampler: wgpu::Sampler,
}

/// A frame the canvas samples: a screenshot uploaded once, or a decoded frame
/// in a texture the platform's decoder writes into.
#[derive(Clone)]
pub struct SourceTexture {
  pub size: (u32, u32),
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
  /// The Direct3D 11 side of a texture a Windows decoder copies into.
  #[cfg(target_os = "windows")]
  pub shared: Option<std::sync::Arc<screenwide_gpu::SharedTexture>>,
  /// A screenshot's own pixels, which its redactions read their fills from.
  /// A video frame has none: its fills are read from its clip's frames.
  pub picture: Option<std::sync::Arc<screenwide_imaging::CapturedImage>>,
  /// A camera with Enhance on, which the canvas enlarges along its edges
  /// (FSR 1's EASU) wherever it draws it larger than it is, rather than
  /// stretching it.
  pub enhanced: bool,
  /// A camera whose background is taken away to nothing, whose drop shadow
  /// the canvas casts from the person's outline rather than its frame.
  pub cut_out: bool,
}

impl Compositor {
  pub fn gpu(&self) -> &'static Gpu {
    self.gpu
  }

  /// Where the shortcut strip shows, for the live preview's hit-testing.
  pub fn keyboard_visible_bounds(
    &self,
    overlay: &KeyboardOverlay,
    output: (u32, u32),
  ) -> Result<Option<[f64; 4]>, String> {
    self
      .keyboard_cache
      .visible_bounds(self.gpu, overlay, output)
  }
}

/// A ported generator's seed domain shift as the canvas carries it. Windows
/// resolves the shift on the CPU once per seed (`generator_seed_shift`), so
/// every draw gets the same one. macOS hashes on the GPU, as its canvases
/// always have, so a seed keeps the picture it had: the last word asks the
/// shader for its own hash.
fn generator_shift(seed: u32) -> [f32; 4] {
  #[cfg(target_os = "windows")]
  {
    let shift = screenwide_output::generator_seed_shift(seed);
    [shift[0], shift[1], shift[2], 0.0]
  }
  #[cfg(not(target_os = "windows"))]
  {
    let _ = seed;
    [0.0, 0.0, 0.0, 1.0]
  }
}

/// A cursor's artwork words in `Canvas`: `cursor_artwork`, `cursor_frame`
/// and `cursor_model`.
type ArtworkConstants = ([f32; 4], [f32; 4], [u32; 4]);

#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod caption_render_tests;
#[cfg(all(test, target_os = "windows"))]
mod crop_preview_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod cut_out_shadow_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod focus_tests;
#[cfg(all(test, target_os = "windows", target_arch = "x86_64"))]
mod fpu_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod precompiled_tests;
#[cfg(all(test, target_os = "windows"))]
mod redact_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod render_test_helpers;
#[cfg(all(test, target_os = "windows"))]
mod render_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod scene_turn_tests;

/// Where a baked camera sits in the frame being composed: its crop and its
/// frame in output pixels, and the corner radius it is drawn with.
#[derive(Clone, Copy, Debug)]
pub struct BakeGeometry {
  pub crop_height: u32,
  pub crop_width: u32,
  pub crop_x: u32,
  pub crop_y: u32,
  pub frame_height: u32,
  pub frame_width: u32,
  pub frame_x: i32,
  pub frame_y: i32,
  pub output_height: u32,
  pub output_width: u32,
  pub radius: u32,
}
