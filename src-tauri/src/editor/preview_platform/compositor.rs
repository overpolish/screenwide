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
mod cursor_artwork;
mod draw;
mod layer;
mod mark_blur;
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

pub(crate) use camera::CameraComposition;
pub(crate) use canvas_variants::KindMask;
pub(crate) use layer::{CanvasGeometry, LayerDraw, LayerPlacement};

use super::background_image::BackgroundImageCache;
use super::keyboard_artwork::{KeyboardArtworkCache, KeyboardConstants};
use crate::editor::keyboard_effects::KeyboardOverlay;
use crate::editor::media_preview::BakeGeometry;
use crate::editor::preview_platform::annotation_gpu::{
  numbered_arrows, place_stickers, CounterAtlas, GpuBuffer, PreparedArrows, StickerAtlas,
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
  /// The handed artwork's bitmap and design size, the hotspot in the
  /// recorded cursor box and the design's origin, and the artwork flags.
  cursor_artwork: [f32; 4],
  cursor_frame: [f32; 4],
  cursor_model: [u32; 4],
  /// The annotation tiles: side, grid width and height, and words a tile's
  /// set takes.
  annotation_tiles: [u32; 4],
  /// Where the canvas is drawn in its target: the corner, and the canvas
  /// pixels one target pixel covers. A canvas drawn at its own size from the
  /// target's corner carries the identity.
  placement: [f32; 4],
  /// The spotlights' blur of the annotations under their shade: the layer's
  /// texels per canvas pixel, whether this pass draws that layer (1) or reads
  /// it (2), and the index of the spotlight whose shade it lies under.
  annotation_blur: [f32; 4],
  /// How a scene moved the screen's box while the shutter was open: the scale
  /// and the shift, in canvas pixels, that carry the box as drawn onto where it
  /// was, then how many steps the frame averages. One step draws it sharp.
  scene_motion: [f32; 4],
  /// The same scale and shift for the screen's image, which a zoom carries
  /// further than its box.
  scene_image_motion: [f32; 4],
  /// The camera's frame when the shutter opened, in canvas pixels, and the
  /// part of its picture that frame showed, in shares of the picture.
  camera_motion_frame: [f32; 4],
  camera_motion_crop: [f32; 4],
  /// How opaque the screen and the camera are drawn, which a scene fades as
  /// it hides or shows them; the other two are unused.
  scene_opacity: [f32; 4],
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
  sticker_atlas: StickerAtlas,
  keyboard_cache: KeyboardArtworkCache,
  /// Bound where no shortcut is on screen, no background picture is chosen,
  /// no camera is composed, no type was rasterised or no sticker drawn: one
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
#[cfg(all(test, target_os = "windows", target_arch = "x86_64"))]
mod fpu_tests;
#[cfg(all(test, target_os = "windows"))]
mod precompiled_tests;
#[cfg(all(test, target_os = "windows"))]
mod redact_tests;
#[cfg(all(test, target_os = "windows"))]
mod render_test_helpers;
#[cfg(all(test, target_os = "windows"))]
mod render_tests;
