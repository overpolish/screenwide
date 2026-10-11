// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One-pass canvas compositor on the shared wgpu device: the background, the
//! picture, the camera, the annotations, the keyboard overlay and the cursor,
//! drawn by `preview.wgsl` into whatever target the caller hands it. The
//! platform brings the frames in and takes the canvas out.

mod background;
mod camera;
mod canvas_modules;
mod canvas_pipelines;
mod canvas_precompiled;
mod canvas_variants;
mod constants;
mod cursor_artwork;
mod draw;
mod layer;
mod mark_blur;
pub(crate) mod mips;
mod pipeline;
mod preview_bindings;
mod redact;
mod redact_pipeline;
mod redact_targets;
mod scene_motion;
mod source;
#[cfg(target_os = "macos")]
pub(crate) mod still;
mod submit;
mod tiles;
#[cfg(target_os = "macos")]
pub(crate) mod video_planes;
mod whole_blur;

pub(crate) use camera::CameraComposition;
pub(crate) use canvas_variants::KindMask;
pub(crate) use layer::{CanvasGeometry, LayerDraw, LayerPlacement};

use self::constants::Constants;

use super::background_image::BackgroundImageCache;
use super::keyboard_artwork::{KeyboardArtworkCache, KeyboardConstants};
use crate::editor::keyboard_effects::KeyboardOverlay;
use crate::editor::media_preview::BakeGeometry;
use crate::editor::preview_platform::annotation_gpu::{
  numbered_arrows, place_images, CounterAtlas, GpuBuffer, ImageAtlas, PreparedArrows,
};
use crate::gpu::Gpu;
use crate::screenshots::{
  colour_f32, foreground_bounds_f32, generator_palette, mesh_generator, optional_colour_f32,
  output_placement, validate_mesh, ScreenshotOutputSettings,
};

const SHADER: &str = include_str!(concat!(env!("OUT_DIR"), "/preview.wgsl"));

/// Everything the canvas pass composes into and from, as one format.
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// What one composition draws besides the picture and the annotations.
#[derive(Clone, Copy)]
pub(crate) struct ComposedFrame {
  pub cursor: Option<crate::editor::cursor_effects::GpuCursor>,
  pub keyboard: Option<crate::editor::keyboard_effects::KeyboardOverlay>,
  /// A layer laid over the layers under it: no background, and blended
  /// over what the target already holds.
  pub foreground_only: bool,
  pub seconds: f64,
}

/// The crop tool's loupe: where it sits, what it shows and how.
#[derive(Clone, Copy)]
pub(crate) struct CropMagnifier {
  pub(crate) bounds: [f32; 4],
  /// Where the loupe is drawn in the window, which only the Windows preview
  /// places an overlay at.
  #[cfg_attr(target_os = "macos", allow(dead_code))]
  pub(crate) display_box: [f32; 4],
  pub(crate) geometry: [f32; 4],
  pub(crate) options: [f32; 4],
}

/// The cursor artwork the platform draws the recorded pointer with: one
/// premultiplied BGRA bitmap per style, every one `size`, one after another,
/// and each style's hotspot in it, normalised.
pub(crate) struct NativeCursors {
  pub(crate) size: (u32, u32),
  pub(crate) layers: Vec<u8>,
  pub(crate) hotspots: [[f32; 4]; 8],
}

#[cfg(target_os = "macos")]
impl NativeCursors {
  /// For a compositor whose cursor is drawn from artwork it is handed: one
  /// transparent texel a style.
  pub(crate) fn none() -> Self {
    Self {
      size: (1, 1),
      layers: vec![0; 4 * 8],
      hotspots: [[0.0; 4]; 8],
    }
  }
}

/// One cursor style drawn from artwork the caller hands over (macOS): its
/// straight RGBA bitmap, the drawing's own size and origin where it was
/// designed rather than captured, and how it is drawn.
#[cfg(target_os = "macos")]
pub(crate) struct CursorArtwork<'a> {
  pub(crate) pixels: &'a [u8],
  pub(crate) size: (u32, u32),
  pub(crate) design: (f32, f32),
  pub(crate) origin: (f32, f32),
  pub(crate) use_design: bool,
  pub(crate) clip_local_box: bool,
  pub(crate) supersample: bool,
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

pub(crate) struct Compositor {
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
  /// The styles `cursor_view` holds when it holds handed artwork, and a
  /// fingerprint of that artwork, so an unchanged set is not uploaded again.
  cursor_artworks: Vec<ArtworkStyle>,
  #[cfg(target_os = "macos")]
  cursor_artwork_key: u64,
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
  redactor: redact::Redactor,
  /// The pictures baked cameras are composed into, one for each slot.
  camera_canvases: std::sync::Mutex<std::collections::HashMap<usize, SourceTexture>>,
  mark_blur: mark_blur::MarkBlur,
  mips: mips::Mips,
  sampler: wgpu::Sampler,
  point_sampler: wgpu::Sampler,
}

/// A frame the canvas samples: a screenshot uploaded once, or a decoded frame
/// in a texture the platform's decoder writes into.
#[derive(Clone)]
pub(crate) struct SourceTexture {
  pub(crate) size: (u32, u32),
  pub(crate) texture: wgpu::Texture,
  pub(crate) view: wgpu::TextureView,
  /// The Direct3D 11 side of a texture a Windows decoder copies into.
  #[cfg(target_os = "windows")]
  pub(crate) shared: Option<std::sync::Arc<crate::gpu::SharedTexture>>,
  /// A screenshot's own pixels, which its redactions read their fills from.
  /// A video frame has none: its fills are read from its clip's frames.
  pub(crate) picture: Option<std::sync::Arc<crate::screenshots::CapturedImage>>,
  /// A camera with Enhance on, which the canvas enlarges along its edges
  /// (FSR 1's EASU) wherever it draws it larger than it is, rather than
  /// stretching it.
  pub(crate) enhanced: bool,
  /// A camera whose background is taken away to nothing, whose drop shadow
  /// the canvas casts from the person's outline rather than its frame.
  pub(crate) cut_out: bool,
}

impl Compositor {
  pub(crate) fn gpu(&self) -> &'static Gpu {
    self.gpu
  }

  /// Where the shortcut strip shows, for the live preview's hit-testing.
  pub(crate) fn keyboard_visible_bounds(
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
    let shift = crate::screenshots::generator_seed_shift(seed);
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

#[cfg(all(test, target_os = "windows"))]
mod crop_preview_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod cut_out_shadow_tests;
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
