// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures images show, named by asset id: `image:` and a name is a
//! picture someone gave the editor, kept by [`store`]. The annotation model
//! carries only the id; this reads it and draws the picture at the size the
//! compositor asks for.

use std::sync::Arc;

/// An animated picture's frames, and when each starts.
pub(crate) mod animation;
/// A Windows bitmap, as drags carry it, made into a file the `image` crate
/// reads.
#[cfg(any(target_os = "windows", test))]
pub(crate) mod dib;
/// A picture dropped on the workspace, on its way to becoming an image.
pub(crate) mod dropped;
/// Bringing a picture in from a file or the clipboard.
pub mod import;
/// A picture drawn at a size, as the image atlas holds it.
mod raster;
/// A picture's shadow, blurred, as the image atlas holds it.
mod shadow;
/// The pictures given to images, kept in the app's data folder.
pub(crate) mod store;

pub(crate) use raster::rasterize;
pub(crate) use shadow::{rasterize_shadow, shadow_margin, SHADOW_SIDE};

/// An image's picture, ready to be drawn at any size.
#[derive(Clone)]
pub(crate) enum StoredImage {
  /// A stored picture that holds still, premultiplied.
  Still(Arc<image::RgbaImage>),
  /// A stored picture that moves: the atlas draws the frame a moment shows,
  /// as a `Still` of its own.
  Animation(Arc<animation::ImageAnimation>),
}

/// The picture `asset` names, or `None` for an id no picture can have or a
/// stored picture that is no longer there.
pub(crate) fn picture(asset: &str) -> Option<StoredImage> {
  let name = asset.strip_prefix(store::IMAGE_PREFIX)?;
  store::is_name(name).then(|| store::load(name)).flatten()
}
