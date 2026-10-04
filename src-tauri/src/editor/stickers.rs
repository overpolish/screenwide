// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures stickers show, named by asset id. `emoji:` and the emoji
//! itself is drawn by the system's own emoji font - Apple Color Emoji
//! through Core Text, Segoe UI Emoji through DirectWrite - so a sticker looks
//! the way the same emoji does everywhere else on the machine. `image:` and
//! a name is a picture someone gave the editor, kept by [`store`]. The
//! annotation model carries only the id; this reads it and draws the picture
//! at the size the compositor asks for.

use std::sync::Arc;

/// An animated picture's frames, and when each starts.
pub(crate) mod animation;
/// A Windows bitmap, as drags carry it, made into a file the `image` crate
/// reads.
#[cfg(any(target_os = "windows", test))]
pub(crate) mod dib;
/// A picture dropped on the workspace, on its way to becoming a sticker.
pub(crate) mod dropped;
/// Bringing a picture in from a file or the clipboard.
pub mod import;
/// A picture drawn at a size, as the sticker atlas holds it.
mod raster;
/// A picture's shadow, blurred, as the sticker atlas holds it.
mod shadow;
/// The pictures given to stickers, kept in the app's data folder.
pub(crate) mod store;

pub(crate) use raster::rasterize;
pub(crate) use shadow::{rasterize_shadow, shadow_margin, SHADOW_SIDE};

/// The longest emoji an asset id may carry, in UTF-8 bytes: room for the
/// longest family and flag sequences, and nothing that is not an emoji.
const MAX_EMOJI_BYTES: usize = 64;

/// A sticker's picture, ready to be drawn at any size.
#[derive(Clone)]
pub(crate) enum StickerPicture {
  Emoji(String),
  /// A stored picture, premultiplied.
  Image(Arc<image::RgbaImage>),
  /// A stored picture that moves: the atlas draws the frame a moment shows,
  /// as an `Image` of its own.
  Animation(Arc<animation::StickerAnimation>),
}

/// The picture `asset` names, or `None` for an id no picture can have or a
/// stored picture that is no longer there.
pub(crate) fn picture(asset: &str) -> Option<StickerPicture> {
  if let Some(name) = asset.strip_prefix(store::IMAGE_PREFIX) {
    return store::is_name(name).then(|| store::load(name)).flatten();
  }
  let emoji = asset.strip_prefix("emoji:")?;
  let drawable = !emoji.is_empty()
    && emoji.len() <= MAX_EMOJI_BYTES
    && !emoji
      .chars()
      .any(|character| character.is_control() || character.is_whitespace());
  drawable.then(|| StickerPicture::Emoji(emoji.to_owned()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_an_emoji_names_a_picture() {
    assert!(picture("emoji:👍").is_some());
    assert!(picture("emoji:👩‍💻").is_some());
    for refused in ["emoji:", "emoji:a b", "emoji:\n", "👍", "image:👍"] {
      assert!(picture(refused).is_none(), "{refused}");
    }
    assert!(picture(&format!("emoji:{}", "👍".repeat(17))).is_none());
  }
}
