// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sticker's picture drawn at a size, as the sticker atlas holds it.
//!
//! An emoji is type, so its font says nothing of where its picture sits in
//! its line or how much of the em it fills, and the two system fonts answer
//! differently. Each emoji is drawn once at a measuring size and its ink
//! found, so every later draw scales and centres that ink to fill the cell.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use super::StickerPicture;
use crate::editor::preview_platform::type_device::draw_emoji;

/// The size an emoji is measured at, in pixels to the em.
const MEASURE: f64 = 128.0;

/// Where an emoji's ink lies, in ems from the top-left of its line.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ink {
  left: f64,
  top: f64,
  width: f64,
  height: f64,
}

/// The ink `emoji` leaves when drawn at [`MEASURE`], or `None` where the
/// font draws nothing for it.
fn measure(emoji: &str) -> Option<Ink> {
  let side = (MEASURE * 3.0) as u32;
  let pixels = draw_emoji(emoji, MEASURE, (side, side), (MEASURE, MEASURE)).ok()?;
  let (mut low, mut high) = ((side, side), (0, 0));
  for (index, pixel) in pixels.as_chunks::<4>().0.iter().enumerate() {
    if pixel[3] == 0 {
      continue;
    }
    let (x, y) = (index as u32 % side, index as u32 / side);
    low = (low.0.min(x), low.1.min(y));
    high = (high.0.max(x + 1), high.1.max(y + 1));
  }
  (high.0 > low.0 && high.1 > low.1).then(|| Ink {
    left: (f64::from(low.0) - MEASURE) / MEASURE,
    top: (f64::from(low.1) - MEASURE) / MEASURE,
    width: f64::from(high.0 - low.0) / MEASURE,
    height: f64::from(high.1 - low.1) / MEASURE,
  })
}

/// [`measure`], kept: the atlas draws an emoji afresh at every size it is
/// shown at, and its ink in ems is the same at all of them.
fn ink(emoji: &str) -> Option<Ink> {
  static INKS: OnceLock<Mutex<HashMap<String, Option<Ink>>>> = OnceLock::new();
  let inks = INKS.get_or_init(Default::default);
  if let Some(known) = inks.lock().ok()?.get(emoji) {
    return *known;
  }
  let found = measure(emoji);
  inks.lock().ok()?.insert(emoji.to_owned(), found);
  found
}

/// `picture` drawn to fill `size` pixels, less a transparent pixel of margin
/// on every side, as premultiplied BGRA rows, top row first. The margin keeps
/// the filtered reads at the picture's edge off the next cell over.
pub(crate) fn rasterize(picture: &StickerPicture, size: (u32, u32)) -> Option<Vec<u8>> {
  let inner = (f64::from(size.0) - 2.0, f64::from(size.1) - 2.0);
  if inner.0 <= 0.0 || inner.1 <= 0.0 {
    return None;
  }
  match picture {
    StickerPicture::Emoji(emoji) => {
      let ink = ink(emoji)?;
      let font = (inner.0 / ink.width).min(inner.1 / ink.height);
      let origin = (
        1.0 + (inner.0 - ink.width * font) / 2.0 - ink.left * font,
        1.0 + (inner.1 - ink.height * font) / 2.0 - ink.top * font,
      );
      draw_emoji(emoji, font, size, origin).ok()
    }
    StickerPicture::Image(image) => Some(framed(image, size)),
    // The atlas draws the frame each moment shows; asked for the picture
    // alone, a moving one is its first.
    StickerPicture::Animation(animation) => Some(framed(animation.frames.first()?, size)),
  }
}

/// A stored picture stretched over the cell inside its margin, which its
/// proportions already match, as premultiplied BGRA.
fn framed(image: &image::RgbaImage, size: (u32, u32)) -> Vec<u8> {
  let (inner_width, inner_height) = (size.0 - 2, size.1 - 2);
  let drawn = crate::editor::stickers::store::resized(image, inner_width, inner_height);
  let mut pixels = vec![0_u8; size.0 as usize * size.1 as usize * 4];
  for (y, row) in drawn.rows().enumerate() {
    let start = ((y + 1) * size.0 as usize + 1) * 4;
    for (out, pixel) in pixels[start..start + inner_width as usize * 4]
      .as_chunks_mut::<4>()
      .0
      .iter_mut()
      .zip(row)
    {
      *out = [pixel[2], pixel[1], pixel[0], pixel[3]];
    }
  }
  pixels
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn an_emoji_fills_its_cell_inside_a_clear_margin() {
    let size = (66, 66);
    let pixels = rasterize(&StickerPicture::Emoji("🔴".to_owned()), size).expect("a drawn emoji");
    let at = |x: usize, y: usize| &pixels[(y * size.0 as usize + x) * 4..][..4];
    // The margin is clear, the circle reaches close to it either side, and
    // its middle is solid red, as BGRA.
    assert!((0..66).all(|edge| at(0, edge)[3] == 0 && at(edge, 0)[3] == 0));
    assert!(
      at(5, 33)[3] > 0 && at(60, 33)[3] > 0,
      "the ink is not centred"
    );
    let middle = at(33, 33);
    assert_eq!(middle[3], 255);
    assert!(middle[2] > 150 && middle[0] < 100, "{middle:?}");
  }
}
