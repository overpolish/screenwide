// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[path = "raster/sampling.rs"]
mod sampling;
#[cfg(test)]
#[path = "raster/tests.rs"]
mod tests;

use crate::recording::cursor::CursorStyle;
#[cfg(test)]
use image::RgbaImage;

#[cfg(target_os = "macos")]
#[path = "raster/platform_macos.rs"]
mod platform;
#[cfg(target_os = "macos")]
pub(super) use platform::gpu_rotation_radians;
#[cfg(not(target_os = "macos"))]
#[path = "raster/platform_unsupported.rs"]
mod platform;

mod fallback;

pub(super) fn uses_same_artwork(left: CursorStyle, right: CursorStyle) -> bool {
  // A custom cursor is recorded without its pixels, so it draws the system
  // arrow fitted inside its recorded box and anchored by the arrow's own
  // hotspot, not stretched over the box from the recorded one. That is a
  // different mapping from the `Arrow` style even though the bitmap matches,
  // so collapsing the two would hold a custom cursor's box and hotspot over
  // the arrow that follows it.
  if (left == CursorStyle::Custom) != (right == CursorStyle::Custom) {
    return false;
  }
  platform::canonical_style(left) == platform::canonical_style(right)
}

pub(super) fn initialize_system_artwork() {
  platform::initialize();
}

/// Artwork order shared by the GPU compositors' style-indexed textures and
/// `GpuCursor::style`. Every system style that resolves to its own artwork
/// appears once; `Custom` takes the extra slot after them.
pub(super) const GPU_ARTWORK_STYLES: [CursorStyle; 16] = [
  CursorStyle::Arrow,
  CursorStyle::ClosedHand,
  CursorStyle::ContextMenu,
  CursorStyle::Crosshair,
  CursorStyle::DisappearingItem,
  CursorStyle::DragCopy,
  CursorStyle::DragLink,
  CursorStyle::IBeam,
  CursorStyle::NotAllowed,
  CursorStyle::OpenHand,
  CursorStyle::PointingHand,
  CursorStyle::ResizeHorizontal,
  CursorStyle::ResizeVertical,
  CursorStyle::VerticalIBeam,
  CursorStyle::ZoomIn,
  CursorStyle::ZoomOut,
];

/// The slot after the system styles, holding the artwork a custom cursor
/// draws. The sidecar records a custom cursor's box and hotspot but none of
/// its pixels, and that box has no reason to share the system arrow's aspect,
/// so stretching the arrow over it squashes the drawn cursor. This slot fits
/// the system arrow inside the recorded box at the arrow's own aspect instead,
/// anchored by the arrow's own hotspot, and falls back to the baked vector
/// arrow only where the system artwork could not be loaded at all.
pub(super) const GPU_CUSTOM_ARTWORK_INDEX: u32 = GPU_ARTWORK_STYLES.len() as u32;

pub(super) fn artwork_index(style: CursorStyle) -> u32 {
  if style == CursorStyle::Custom {
    return GPU_CUSTOM_ARTWORK_INDEX;
  }
  let style = platform::canonical_style(style);
  GPU_ARTWORK_STYLES
    .iter()
    .position(|candidate| *candidate == style)
    .unwrap_or(0) as u32
}

/// One style's artwork bitmap plus the mapping the shader needs to place it,
/// mirroring exactly what [`CursorRaster::sample`] does on the CPU. A system
/// style's artwork stretches over the recorded cursor box; artwork placed by
/// its own design frame (the vector fallback, and the arrow a custom cursor
/// draws) keeps that frame's aspect inside the box and anchors `origin` at the
/// recorded position, so the frame and origin travel with it.
#[cfg(target_os = "macos")]
pub(crate) struct GpuArtwork {
  pub design_height: f32,
  pub design_width: f32,
  pub height: u32,
  pub origin_x: f32,
  pub origin_y: f32,
  pub pixels: Vec<u8>,
  /// The fallback arrow deliberately draws outside the recorded cursor box so
  /// its rounded tip stroke survives at the hotspot (`sample`, raster.rs:85).
  pub clip_local_box: bool,
  pub supersample: bool,
  pub use_design: bool,
  pub width: u32,
}

/// Texels per design unit when the vector fallback is baked for the GPU. The
/// shader supersamples the same 4x4 box `sample_for_draw` uses, so the bake
/// stays point-sampled rather than pre-filtered.
#[cfg(target_os = "macos")]
const FALLBACK_BAKE_SCALE: u32 = 8;

#[cfg(target_os = "macos")]
pub(super) fn gpu_artworks() -> Vec<GpuArtwork> {
  GPU_ARTWORK_STYLES
    .iter()
    .copied()
    .map(gpu_artwork)
    // `GPU_CUSTOM_ARTWORK_INDEX`: a custom cursor draws the system arrow
    // fitted inside its recorded box, never that arrow stretched over the box.
    .chain(std::iter::once(custom_gpu_artwork(
      platform::style_artwork(CursorStyle::Arrow),
    )))
    .collect()
}

/// The artwork a custom cursor draws. Taking the arrow entry as an argument
/// keeps the routing testable in a process that never loaded system artwork.
///
/// Geometry, matching [`CursorRaster::sample`] and the Windows compositor's
/// `native_cursor_hotspots` rule (preview_platform/surface_windows
/// /compositor.rs:118):
/// - the design frame is the arrow bitmap itself, so the shader's `use_design`
///   path scales it by `min(box_width / frame_width, box_height / frame_height)`
///   and never stretches it to the recorded box's aspect;
/// - `origin` is the arrow's own hotspot, which the shader adds after that
///   scale, so the arrow is anchored by its hotspot at the recorded position.
///   The recorded hotspot addresses artwork that is not being drawn and stays
///   out of it (`output_hotspot`, cursor_effects.rs:94);
/// - the fitted arrow reaches outside the recorded box on the hotspot's side,
///   so only the design frame clips it.
#[cfg(target_os = "macos")]
fn custom_gpu_artwork(arrow: Option<&platform::StyleArtwork>) -> GpuArtwork {
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
    supersample: false,
    use_design: true,
    width: arrow.image.width(),
  }
}

#[cfg(target_os = "macos")]
fn gpu_artwork(style: CursorStyle) -> GpuArtwork {
  if let Some(image) = platform::artwork(style) {
    return GpuArtwork {
      design_height: 0.0,
      design_width: 0.0,
      height: image.height(),
      origin_x: 0.0,
      origin_y: 0.0,
      pixels: image.as_raw().clone(),
      clip_local_box: true,
      supersample: false,
      use_design: false,
      width: image.width(),
    };
  }
  fallback_gpu_artwork(style)
}

#[cfg(target_os = "macos")]
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

/// Places system artwork by its own design frame instead of stretching it over
/// the recorded cursor box: the frame is fitted into the box at a single
/// aspect-preserving scale and `origin` (the artwork's own hotspot, in frame
/// units) lands at the recorded position. Only a custom cursor uses this - its
/// pixels are not recorded, so neither its box's aspect nor its hotspot
/// describes the arrow that stands in for it.
#[cfg(test)]
#[derive(Clone, Copy)]
struct SystemDesign {
  height: f64,
  origin_x: f64,
  origin_y: f64,
  width: f64,
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) struct CursorRaster {
  artwork: fallback::Artwork,
  cos: f64,
  height: f64,
  hotspot_x: f64,
  hotspot_y: f64,
  scale: f64,
  sin: f64,
  system_artwork: Option<&'static RgbaImage>,
  system_design: Option<SystemDesign>,
  width: f64,
}
