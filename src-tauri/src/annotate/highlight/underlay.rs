// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pixels a display's highlights are recoloured from, and its spotlights
//! blur.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use crate::screenshots::CapturedImage;

static REVISION: AtomicU64 = AtomicU64::new(1);

/// One display's underlay, in its capture's pixels. `revision` changes every
/// time an underlay is made, so the native side uploads each one once.
pub(crate) struct Underlay {
  pub(crate) image: CapturedImage,
  pub(crate) revision: u64,
  /// The image softened for the spotlights' blur, made the first time it is
  /// asked for: most underlays are only ever recoloured by a highlight.
  softened: OnceLock<CapturedImage>,
}

impl Underlay {
  /// `latest` with each `(left, top, crop)` pasted over it.
  pub(super) fn compose(latest: &CapturedImage, pieces: &[(u32, u32, &CapturedImage)]) -> Self {
    let mut image = latest.clone();
    for (left, top, crop) in pieces {
      paste(&mut image, *left, *top, crop);
    }
    Self {
      image,
      revision: REVISION.fetch_add(1, Ordering::Relaxed),
      softened: OnceLock::new(),
    }
  }

  /// The underlay softened, at a fraction of its size, for the spotlights'
  /// blur to stretch back over the display.
  pub(crate) fn softened(&self) -> &CapturedImage {
    self
      .softened
      .get_or_init(|| crate::editor::annotations::spotlight::soften::soften(&self.image))
  }
}

/// The part of `image` from `(left, top)` to `(right, bottom)`, or nothing
/// where that is empty.
pub(super) fn crop(
  image: &CapturedImage,
  left: u32,
  top: u32,
  right: u32,
  bottom: u32,
) -> Option<CapturedImage> {
  let right = right.min(image.width);
  let bottom = bottom.min(image.height);
  if right <= left || bottom <= top {
    return None;
  }
  let width = right - left;
  let mut rgba = Vec::with_capacity((width * (bottom - top) * 4) as usize);
  for y in top..bottom {
    let from = ((y * image.width + left) * 4) as usize;
    rgba.extend_from_slice(image.rgba.get(from..from + (width * 4) as usize)?);
  }
  Some(CapturedImage {
    width,
    height: bottom - top,
    rgba,
  })
}

/// `piece` written over `image` with its corner at `(left, top)`, clipped to
/// `image`.
fn paste(image: &mut CapturedImage, left: u32, top: u32, piece: &CapturedImage) {
  let width = piece.width.min(image.width.saturating_sub(left));
  let height = piece.height.min(image.height.saturating_sub(top));
  for row in 0..height {
    let to = (((top + row) * image.width + left) * 4) as usize;
    let from = ((row * piece.width) * 4) as usize;
    let length = (width * 4) as usize;
    if let (Some(target), Some(source)) = (
      image.rgba.get_mut(to..to + length),
      piece.rgba.get(from..from + length),
    ) {
      target.copy_from_slice(source);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn filled(width: u32, height: u32, value: u8) -> CapturedImage {
    CapturedImage {
      width,
      height,
      rgba: vec![value; (width * height * 4) as usize],
    }
  }

  #[test]
  fn an_earlier_highlight_keeps_the_pixels_it_was_drawn_on() {
    let first = filled(8, 8, 10);
    let kept = crop(&first, 2, 2, 5, 4).unwrap();
    let latest = filled(8, 8, 200);
    let underlay = Underlay::compose(&latest, &[(2, 2, &kept)]);
    let at = |x: u32, y: u32| underlay.image.rgba[((y * 8 + x) * 4) as usize];
    assert_eq!(at(3, 3), 10);
    assert_eq!(at(5, 3), 200);
    assert_eq!(at(3, 4), 200);
  }

  #[test]
  fn a_crop_past_the_edge_is_clipped_to_the_image() {
    let crop = crop(&filled(4, 4, 1), 2, 2, 9, 9).unwrap();
    assert_eq!((crop.width, crop.height), (2, 2));
    assert!(super::crop(&filled(4, 4, 1), 4, 0, 9, 9).is_none());
  }
}
