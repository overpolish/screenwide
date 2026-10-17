// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box drawn in its look, the way a caption's pieces are: its box,
//! where the look has one, as a box alone in the box's colour, and its type
//! as a caption's word is, in its own face and ink with the look's outline
//! and shadow. A look without a box draws its type alone.

use super::super::caption_shadow::cast;
use super::super::captions::OUTLINE;
use super::super::{PreparedArrows, PreparedType, PreviewArrow};
use super::text_words::{word_pieces, WordsSetting};
use screenwide_annotations::flags::{BARE, CAPTION, SHADOW};
use screenwide_annotations::geometry::ArrowGeometry;
use screenwide_annotations::reveal::AnnotationReveal;
use screenwide_annotations::text::native::NativeTextLook;
use screenwide_text::captions::CaptionPieceShadow;

/// What a text box's record draws in `look`: the arrow and type entry it was
/// prepared as, made the box alone where the look has one, and then its
/// type. `size` is the type size it settles at, in canvas pixels, and
/// `opacity` what its reveal folds into a still arrow's colour.
fn dressed(
  mut arrow: PreviewArrow,
  mut entry: PreparedType,
  look: &NativeTextLook,
  size: f32,
  opacity: f32,
) -> Vec<(PreviewArrow, PreparedType)> {
  let mut drawn = Vec::with_capacity(2);
  let mut type_arrow = arrow;
  let mut type_entry = entry.clone();
  if look.boxed {
    arrow.flags |= BARE;
    entry.text.clear();
    entry.marks = None;
    drawn.push((arrow, entry));
    // The box wears the hover halo; the type over it wears none.
    type_arrow.hover = 0.0;
  }
  type_arrow.flags |= CAPTION;
  type_arrow.color = [look.ink[0], look.ink[1], look.ink[2], look.ink[3] * opacity];
  type_entry.face = look.face.clone();
  // Set at the size it settles at, so a box growing in sets the same type
  // every frame, as a caption's word is.
  type_entry.size = size;
  type_arrow.geometry.cx = size;
  if let Some([red, green, blue, _]) = look.outline {
    type_arrow.flags |= SHADOW;
    type_arrow.params = [red, green, blue];
  }
  if look.outline.is_some() || look.shadow.is_some() {
    type_entry.outline = OUTLINE * size;
  }
  if let Some(shadow) = &look.shadow {
    let em = f64::from(size);
    cast(
      &mut type_arrow,
      &mut type_entry,
      &CaptionPieceShadow {
        color: shadow.color.clone(),
        offset: shadow.offset().map(|ems| ems * em),
        softness: shadow.softness * em,
      },
    );
  }
  drawn.push((type_arrow, type_entry));
  drawn
}

/// The box alone, where `look` has one, of a text whose type is drawn word
/// by word instead: drawn by [`word_pieces`].
fn box_alone(
  mut arrow: PreviewArrow,
  mut entry: PreparedType,
  look: &NativeTextLook,
) -> Option<(PreviewArrow, PreparedType)> {
  look.boxed.then(|| {
    arrow.flags |= BARE;
    entry.text.clear();
    entry.marks = None;
    (arrow, entry)
  })
}

/// Draws a text box's record into `prepared` in its look, in its place
/// among the arrows: its box, where the look has one, then its type. Type
/// whose look sets chosen words apart, or moves it at a recording's frame,
/// is drawn word by word; a box being typed into stands still, and only one
/// with words set apart keeps to its words while it is, caret and all. The
/// words' text follows the records' own in the side buffer, after
/// `moving.0`, the word-by-word texts' so far, and `moving.1` bytes of the
/// records'. `whole` is the record prepared whole, `align` its alignment,
/// and `still` whether it was drawn without exposure samples.
pub(super) fn push(
  prepared: &mut PreparedArrows,
  moving: (&mut Vec<u8>, usize),
  (arrow, entry, look): (PreviewArrow, PreparedType, &NativeTextLook),
  (whole, align): (ArrowGeometry, u32),
  (reveal, still): (AnnotationReveal, bool),
) {
  let typing = entry.marks;
  let Some(words) = look
    .words
    .as_ref()
    .filter(|words| !words.emphasis.is_empty() || typing.is_none())
  else {
    let opacity = if still {
      reveal.opacity.clamp(0.0, 1.0)
    } else {
      1.0
    };
    for (arrow, entry) in dressed(arrow, entry, look, whole.width, opacity) {
      prepared.types.push(entry);
      prepared.arrows.push(arrow);
    }
    return;
  };
  let (moving_text, records_text) = moving;
  if let Some((arrow, entry)) = box_alone(arrow, entry, look) {
    prepared.types.push(entry);
    prepared.arrows.push(arrow);
  }
  let setting = WordsSetting {
    centre: whole.start_head.c.map(f64::from),
    size: f64::from(whole.width),
    align,
  };
  let pieces = word_pieces(words, &setting, reveal, (typing, look.ink));
  let text_base = (records_text + moving_text.len()) as u32;
  let samples = prepared.samples.len() as u32;
  prepared
    .arrows
    .extend(pieces.arrows.into_iter().map(|mut arrow| {
      arrow.sample_first += samples;
      arrow.data_offset += text_base;
      arrow
    }));
  prepared.types.extend(pieces.types);
  prepared.samples.extend(pieces.samples);
  moving_text.extend(pieces.text);
}
