// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which picture each image in a composition is drawn from: a moving one at
//! the frame this moment shows, and a still drawn at least [`UPSCALE_FROM`]
//! times its own size from its upscaled picture where it has one and is not
//! stretched, as stretching blurs it. Each comes with the key the image atlas
//! keeps its cell under, which differs for the upscaled picture, so an image
//! whose upscaled picture lands while it shows is drawn again from it.

use std::sync::Arc;

use screenwide_images::StoredImage;

use super::{PreparedArrows, PreparedImage};

/// How many times its picture's own size an image is drawn at before its
/// upscaled picture is used: past it, a stretched picture starts to look
/// soft.
const UPSCALE_FROM: f32 = 1.25;

/// The picture an image is drawn from, the frame a moving one shows, and
/// the key its cell is kept under.
pub(super) struct Picked {
  pub(super) picture: StoredImage,
  pub(super) frame: Option<u32>,
  pub(super) key: Vec<u8>,
}

/// Each image's picture, in `prepared.images` order; `None` for one whose
/// picture cannot be found.
pub(super) fn pick(prepared: &PreparedArrows) -> Vec<Option<Picked>> {
  prepared
    .images
    .iter()
    .map(|image| {
      Some(match screenwide_images::picture(&image.asset)? {
        StoredImage::Animation(animation) => {
          let index = animation.frame_index(image.frame, image.clock_ms, image.once);
          Picked {
            picture: StoredImage::Still(Arc::clone(animation.frames.get(index)?)),
            frame: Some(index as u32),
            key: image.asset.clone().into_bytes(),
          }
        }
        StoredImage::Still(still) => match upscaled(
          image,
          still.width().max(still.height()),
          prepared.pixel_scale,
        ) {
          Some(upscaled) => Picked {
            picture: upscaled,
            frame: None,
            key: format!("{}#upscaled", image.asset).into_bytes(),
          },
          None => Picked {
            picture: StoredImage::Still(still),
            frame: None,
            key: image.asset.clone().into_bytes(),
          },
        },
      })
    })
    .collect()
}

/// `image`'s upscaled picture, where it is drawn at least [`UPSCALE_FROM`]
/// times `own`, its own picture's longer side, is not stretched, and has
/// one.
fn upscaled(image: &PreparedImage, own: u32, pixel_scale: f32) -> Option<StoredImage> {
  if image.stretched {
    return None;
  }
  let scale = if pixel_scale.is_finite() && pixel_scale > 0.0 {
    pixel_scale
  } else {
    1.0
  };
  let drawn = image.size[0].max(image.size[1]) / scale;
  (drawn >= own as f32 * UPSCALE_FROM)
    .then(|| screenwide_images::upscaled_picture(&image.asset))
    .flatten()
}
