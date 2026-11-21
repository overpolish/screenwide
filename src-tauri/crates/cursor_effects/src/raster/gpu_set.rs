// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Every style's artwork as the GPU compositors take it, in
//! [`GPU_ARTWORK_STYLES`] order with the custom slot after them and then the
//! pictures of apps' own cursors (`custom`), made again whenever a picture
//! is added or any artwork is replaced by an upscaled one.

use std::sync::{Arc, Mutex};

use image::RgbaImage;
use screenwide_records::cursor::CursorStyle;

use super::{custom, fallback, platform, style_artwork, upscaled, GPU_ARTWORK_STYLES};

/// One style's artwork bitmap plus the mapping the shader needs to place it,
/// mirroring exactly what [`CursorRaster::sample`] does on the CPU. A system
/// style's artwork stretches over the recorded cursor box; artwork placed by
/// its own design frame (the vector fallback, and the arrow a custom cursor
/// draws) keeps that frame's aspect inside the box and anchors `origin` at the
/// recorded position, so the frame and origin travel with it.
#[derive(Clone)]
pub struct GpuArtwork {
  pub design_height: f32,
  pub design_width: f32,
  pub height: u32,
  pub origin_x: f32,
  pub origin_y: f32,
  pub pixels: Vec<u8>,
  /// The fallback arrow deliberately draws outside the recorded cursor box so
  /// its rounded tip stroke survives at the hotspot (`sample`, raster.rs:85).
  pub clip_local_box: bool,
  /// Read over the output pixel's 4x4 box rather than once. Every artwork the
  /// app draws is: the system's is drawn from bitmaps up to 16 times its
  /// point size (the vector cursors, the 10x arrow), which one bilinear read
  /// at a small pointer size would skip across, and the fallback's edge is
  /// hard.
  pub supersample: bool,
  pub use_design: bool,
  pub width: u32,
}

/// Texels per design unit when the vector fallback is baked for the GPU. The
/// shader supersamples the same 4x4 box `sample_for_draw` uses, so the bake
/// stays point-sampled rather than pre-filtered.
const FALLBACK_BAKE_SCALE: u32 = 8;

/// The set last made, for whether the system artwork was loaded and how many
/// upscaled replacements and pictures had landed when it was.
struct Made {
  loaded: bool,
  generation: (u64, u64),
  set: Arc<[GpuArtwork]>,
}

/// Every style's artwork in `GPU_ARTWORK_STYLES` order, the custom slot and
/// the apps' pictures after them. A process that has not loaded the system
/// artwork, as the tests, gets the fallback set. The set is made once and
/// again only when it changes, so a compositor can tell by the `Arc` whether
/// it has the latest.
pub(crate) fn gpu_artworks() -> Arc<[GpuArtwork]> {
  static MADE: Mutex<Option<Made>> = Mutex::new(None);
  let loaded = style_artwork(CursorStyle::Arrow).is_some();
  let generation = (upscaled::generation(), custom::generation());
  let mut made = MADE.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
  if let Some(made) = made
    .as_ref()
    .filter(|made| made.loaded == loaded && made.generation == generation)
  {
    return Arc::clone(&made.set);
  }
  let set: Arc<[GpuArtwork]> = artworks().into();
  *made = Some(Made {
    loaded,
    generation,
    set: Arc::clone(&set),
  });
  set
}

fn artworks() -> Vec<GpuArtwork> {
  let pictures: Vec<GpuArtwork> = custom::read()
    .iter()
    .map(|picture| stretched(&picture.image))
    .collect();
  GPU_ARTWORK_STYLES
    .iter()
    .copied()
    .map(gpu_artwork)
    // `GPU_CUSTOM_ARTWORK_INDEX`: a custom cursor with no picture draws the
    // system arrow fitted inside its recorded box, never that arrow
    // stretched over the box.
    .chain(std::iter::once(custom_gpu_artwork(
      style_artwork(CursorStyle::Arrow).as_deref(),
    )))
    .chain(pictures)
    .collect()
}

/// The artwork a custom cursor draws. Taking the arrow entry as an argument
/// keeps the routing testable in a process that never loaded system artwork.
///
/// Geometry, matching [`CursorRaster::sample`]:
/// - the design frame is the arrow bitmap itself, so the shader's `use_design`
///   path scales it by `min(box_width / frame_width, box_height / frame_height)`
///   and never stretches it to the recorded box's aspect;
/// - `origin` is the arrow's own hotspot, which the shader adds after that
///   scale, so the arrow is anchored by its hotspot at the recorded position.
///   The recorded hotspot addresses artwork that is not being drawn and stays
///   out of it (`output_hotspot`, cursor_effects.rs:94);
/// - the fitted arrow reaches outside the recorded box on the hotspot's side,
///   so only the design frame clips it.
pub(in crate::raster) fn custom_gpu_artwork(arrow: Option<&platform::StyleArtwork>) -> GpuArtwork {
  // Last resort: with no system artwork at all (headless, tests) the baked
  // vector arrow still draws by exactly the same rules.
  let Some(arrow) = arrow else {
    return fallback_gpu_artwork(CursorStyle::Custom);
  };
  GpuArtwork {
    design_height: arrow.image.height() as f32,
    design_width: arrow.image.width() as f32,
    height: arrow.image.height(),
    origin_x: arrow.hotspot_x as f32,
    origin_y: arrow.hotspot_y as f32,
    pixels: arrow.image.as_raw().clone(),
    clip_local_box: false,
    supersample: true,
    use_design: true,
    width: arrow.image.width(),
  }
}

fn gpu_artwork(style: CursorStyle) -> GpuArtwork {
  match style_artwork(style) {
    Some(artwork) => stretched(&artwork.image),
    None => fallback_gpu_artwork(style),
  }
}

/// `image` stretched over the recorded cursor box, about the recorded
/// hotspot: how a system style and an app's own picture are both drawn,
/// since each is the picture of exactly that box.
fn stretched(image: &RgbaImage) -> GpuArtwork {
  GpuArtwork {
    design_height: 0.0,
    design_width: 0.0,
    height: image.height(),
    origin_x: 0.0,
    origin_y: 0.0,
    pixels: image.as_raw().clone(),
    clip_local_box: true,
    supersample: true,
    use_design: false,
    width: image.width(),
  }
}

fn fallback_gpu_artwork(style: CursorStyle) -> GpuArtwork {
  let artwork = fallback::artwork(style);
  // The design frames and origin come from `CursorRaster::sample`
  // (raster.rs:93-108); the bake reproduces its `fallback::sample` lookups.
  let (design_width, design_height) = if artwork == fallback::Artwork::Hand {
    (32.0_f64, 32.0_f64)
  } else {
    (28.0_f64, 40.0_f64)
  };
  let (origin_x, origin_y) = fallback::origin(artwork);
  let width = design_width as u32 * FALLBACK_BAKE_SCALE;
  let height = design_height as u32 * FALLBACK_BAKE_SCALE;
  let mut pixels = vec![0_u8; width as usize * height as usize * 4];
  for y in 0..height {
    for x in 0..width {
      // `sample_image` addresses texel `i` at coordinate `i`, so the bake
      // must place its samples on that same grid rather than at texel centres.
      let sample = fallback::sample(
        artwork,
        f64::from(x) / f64::from(FALLBACK_BAKE_SCALE),
        f64::from(y) / f64::from(FALLBACK_BAKE_SCALE),
      );
      let offset = (y as usize * width as usize + x as usize) * 4;
      for channel in 0..4 {
        pixels[offset + channel] = sample[channel].round().clamp(0.0, 255.0) as u8;
      }
    }
  }
  GpuArtwork {
    design_height: design_height as f32,
    design_width: design_width as f32,
    height,
    origin_x: origin_x as f32,
    origin_y: origin_y as f32,
    pixels,
    clip_local_box: artwork != fallback::Artwork::Arrow,
    supersample: true,
    use_design: true,
    width,
  }
}
