// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shadow a caption's word casts, handed to the shader: the atlas blurs
//! the word's outline beside its type, and the arrow carries how far the
//! shadow falls, how soft it is and its colour in the slots a text box
//! leaves to its pointer, which a word has none of.

use super::arrows::{PreparedType, PreviewArrow};
use screenwide_annotations::flags::CASTS;
use screenwide_text::captions::CaptionPieceShadow;
use screenwide_text::rgba;

/// `arrow` and its type `entry`, a word set at its settled size, casting
/// `shadow`. The shader reads the slots back in `annotation_text.wgsl`'s
/// `annotation_caption_shadow_of`.
pub(super) fn cast(
  arrow: &mut PreviewArrow,
  entry: &mut PreparedType,
  shadow: &CaptionPieceShadow,
) {
  let Some(color) = rgba(&shadow.color) else {
    return;
  };
  arrow.flags |= CASTS;
  entry.shadow = Some(shadow.softness as f32);
  let [red, green, blue, alpha] = color.map(|channel| (channel * 255.0).round() as u32);
  let packed = red | (green << 8) | (blue << 16) | (alpha << 24);
  let slots = &mut arrow.geometry.end_head;
  slots[0] = shadow.offset[0] as f32;
  slots[1] = shadow.offset[1] as f32;
  slots[2] = shadow.softness as f32;
  slots[3] = f32::from_bits(packed);
}
