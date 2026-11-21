// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod sampling;
#[cfg(test)]
mod tests;

#[cfg(test)]
use image::RgbaImage;
use screenwide_records::cursor::CursorStyle;

#[cfg(target_os = "macos")]
mod platform_macos;
#[cfg(target_os = "macos")]
use self::platform_macos as platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform_unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use self::platform_unsupported as platform;
#[cfg(target_os = "windows")]
mod platform_windows;
#[cfg(target_os = "windows")]
use self::platform_windows as platform;

#[cfg(any(target_os = "windows", test))]
mod cursor_file;
mod custom;
mod fallback;
mod gpu_set;
pub(super) mod system_cursor;
mod upscaled;

#[cfg(test)]
use gpu_set::custom_gpu_artwork;
pub(crate) use gpu_set::gpu_artworks;
pub use gpu_set::GpuArtwork;
#[cfg(target_os = "windows")]
pub use platform_windows::drawn as drawn_windows_cursor;
use upscaled::style_artwork;
pub use upscaled::{to_upscale, use_upscaled, Upscalable};

/// The artwork index of an app's own cursor, its picture `name` read from
/// `file`; `None` where it cannot be drawn, and the arrow stands in.
pub(super) fn picture_index(name: &str, file: &std::path::Path) -> Option<u32> {
  custom::artwork_index(name, file)
}

pub(super) fn uses_same_artwork(left: CursorStyle, right: CursorStyle) -> bool {
  // A custom cursor without a picture draws the system arrow fitted inside
  // its recorded box and anchored by the arrow's own hotspot, not stretched
  // over the box from the recorded one. That is a different mapping from
  // the `Arrow` style even though the bitmap matches, so collapsing the two
  // would hold a custom cursor's box and hotspot over the arrow that
  // follows it. Two custom cursors' pictures are told apart by the caller.
  if (left == CursorStyle::Custom) != (right == CursorStyle::Custom) {
    return false;
  }
  platform::canonical_style(left) == platform::canonical_style(right)
}

pub(super) fn initialize_system_artwork() {
  platform::initialize();
}

/// The box a style's system artwork is drawn in at its own size - its width
/// and height in points and its hotspot within them - or `None` where there
/// is no artwork for it, as for a custom cursor.
pub(super) fn natural_box(style: CursorStyle) -> Option<[f64; 4]> {
  if style == CursorStyle::Custom {
    return None;
  }
  let artwork = style_artwork(style)?;
  let (width, height) = artwork.points;
  let (texels_wide, texels_high) = (artwork.image.width(), artwork.image.height());
  if width <= 0.0 || height <= 0.0 || texels_wide == 0 || texels_high == 0 {
    return None;
  }
  Some([
    width,
    height,
    artwork.hotspot_x * width / f64::from(texels_wide),
    artwork.hotspot_y * height / f64::from(texels_high),
  ])
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
/// with no picture draws: one recorded before pictures were kept, or whose
/// picture could not be read. Its box has no reason to share the system
/// arrow's aspect, so stretching the arrow over it squashes the drawn
/// cursor. This slot fits the system arrow inside the recorded box at the
/// arrow's own aspect instead, anchored by the arrow's own hotspot, and falls
/// back to the baked vector arrow only where the system artwork could not be
/// loaded at all. Apps' own pictures take the slots after it (`custom`).
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

/// Whether `style` is drawn a quarter turn round: a vertical style drawn with
/// horizontal artwork, borrowed from another style or from the fallback.
pub(super) fn turned_quarter(style: CursorStyle) -> bool {
  fallback::is_vertical(style)
    && (platform::canonical_style(style) != style || style_artwork(style).is_none())
}

pub(super) fn gpu_rotation_radians(style: CursorStyle) -> f32 {
  if turned_quarter(style) {
    std::f32::consts::FRAC_PI_2
  } else {
    0.0
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
