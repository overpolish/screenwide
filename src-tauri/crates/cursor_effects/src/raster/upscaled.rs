// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Artwork too small to draw large, replaced by the app with the same
//! artwork drawn larger by its upscaling model.
//!
//! A style whose artwork has fewer than [`UPSCALE_BELOW`] texels per point
//! (a 2x bitmap, a Windows scheme cursor of 32 or 64 pixels, a macOS cursor
//! whose vector drawing could not be read) blurs at the larger pointer
//! sizes, as does an app's own cursor's picture (`custom`). The app asks for
//! those ([`to_upscale`]), draws each four times as large with its bundled
//! model, which keeps drawn art sharp, and hands it back ([`use_upscaled`]);
//! every compositor draws it from the next frame on.

use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use image::RgbaImage;
use screenwide_records::cursor::CursorStyle;

use super::{custom, platform, GPU_ARTWORK_STYLES};

/// Below this many texels per point, artwork blurs when the pointer is drawn
/// large, and is upscaled.
pub const UPSCALE_BELOW: f64 = 8.0;
/// The most texels per point upscaled artwork is kept at, as the vector
/// cursors are drawn: enough for the largest pointer on a retina recording,
/// and no more GPU memory than those.
const KEPT_AT: f64 = 16.0;

/// The styles replaced so far.
static REPLACED: RwLock<Vec<(CursorStyle, Arc<platform::StyleArtwork>)>> = RwLock::new(Vec::new());
/// How many replacements have landed, which the GPU set is made again for.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// A style's artwork: the system's, loaded once for the process, or its
/// upscaled replacement.
pub(super) enum Artwork {
  System(&'static platform::StyleArtwork),
  Upscaled(Arc<platform::StyleArtwork>),
}

impl Deref for Artwork {
  type Target = platform::StyleArtwork;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::System(artwork) => artwork,
      Self::Upscaled(artwork) => artwork,
    }
  }
}

pub(super) fn generation() -> u64 {
  GENERATION.load(Ordering::Acquire)
}

/// `style`'s artwork: its upscaled replacement where it has one, else the
/// system's.
pub(super) fn style_artwork(style: CursorStyle) -> Option<Artwork> {
  let style = platform::canonical_style(style);
  let replaced = REPLACED
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .iter()
    .find_map(|(candidate, artwork)| (*candidate == style).then(|| Arc::clone(artwork)));
  match replaced {
    Some(artwork) => Some(Artwork::Upscaled(artwork)),
    None => platform::style_artwork(style).map(Artwork::System),
  }
}

/// How many texels per point `artwork` is drawn with.
fn texels_per_point(artwork: &platform::StyleArtwork) -> f64 {
  if artwork.points.0 <= 0.0 {
    return 0.0;
  }
  f64::from(artwork.image.width()) / artwork.points.0
}

/// Artwork the app can draw larger: a system style's, or the picture of an
/// app's own cursor, by its name in the recording.
#[derive(Clone, Debug, PartialEq)]
pub enum Upscalable {
  Style(CursorStyle),
  Picture(String),
}

/// The artwork too small to draw large and not upscaled yet, each with its
/// picture: the system styles', and the pictures of apps' own cursors the
/// recordings opened so far show.
pub fn to_upscale() -> Vec<(Upscalable, RgbaImage)> {
  let mut styles: Vec<CursorStyle> = GPU_ARTWORK_STYLES
    .iter()
    .map(|style| platform::canonical_style(*style))
    .collect();
  styles.sort_by_key(|style| format!("{style:?}"));
  styles.dedup();
  styles
    .into_iter()
    .filter_map(|style| {
      let artwork = style_artwork(style)?;
      (texels_per_point(&artwork) < UPSCALE_BELOW)
        .then(|| (Upscalable::Style(style), artwork.image.clone()))
    })
    .chain(
      custom::to_upscale()
        .into_iter()
        .map(|(name, image)| (Upscalable::Picture(name), image)),
    )
    .collect()
}

/// Draws `what` from `image` from now on: its artwork drawn larger, at the
/// same proportions.
pub fn use_upscaled(what: Upscalable, image: RgbaImage) {
  match what {
    Upscalable::Style(style) => use_upscaled_style(style, image),
    Upscalable::Picture(name) => custom::use_upscaled(&name, image),
  }
}

/// Draws `style` from `image` from now on. Kept at no more than [`KEPT_AT`]
/// texels per point.
fn use_upscaled_style(style: CursorStyle, image: RgbaImage) {
  let style = platform::canonical_style(style);
  let Some(original) = platform::style_artwork(style) else {
    return;
  };
  if image.width() <= original.image.width() || original.points.0 <= 0.0 {
    return;
  }
  let most = (original.points.0 * KEPT_AT).round() as u32;
  let image = if image.width() > most {
    let height = (f64::from(image.height()) * f64::from(most) / f64::from(image.width())).round();
    image::imageops::resize(
      &image,
      most,
      height.max(1.0) as u32,
      image::imageops::FilterType::Lanczos3,
    )
  } else {
    image
  };
  let scale_x = f64::from(image.width()) / f64::from(original.image.width());
  let scale_y = f64::from(image.height()) / f64::from(original.image.height());
  let replacement = Arc::new(platform::StyleArtwork {
    hotspot_x: original.hotspot_x * scale_x,
    hotspot_y: original.hotspot_y * scale_y,
    image,
    points: original.points,
  });
  let mut replaced = REPLACED
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  replaced.retain(|(candidate, _)| *candidate != style);
  replaced.push((style, replacement));
  GENERATION.fetch_add(1, Ordering::AcqRel);
}
