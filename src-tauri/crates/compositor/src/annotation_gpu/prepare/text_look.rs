// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A text box drawn in its look, the way a caption's pieces are: its box,
//! where the look has one, as a box alone in the box's colour, and its type
//! as a caption's word is, in its own face and ink with the look's outline
//! and shadow. A look without a box draws its type alone.

use super::super::caption_shadow::cast;
use super::super::captions::{caption_arrows, caption_measure, CaptionFaces, OUTLINE};
use super::super::{PreparedArrows, PreparedType, PreviewArrow};
use screenwide_annotations::flags::{BARE, CAPTION, SHADOW};
use screenwide_annotations::geometry::ArrowGeometry;
use screenwide_annotations::reveal::AnnotationReveal;
use screenwide_annotations::text::native::{NativeTextLook, NativeTextMotion};
use screenwide_text::captions::{
  text_item_pieces, CaptionMoment, CaptionPieceShadow, TextItemSetting,
};

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

/// The box alone, where `look` has one, of a text whose type is drawn in
/// its look's motion instead: drawn by [`moving_type`].
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

/// A text's type drawn in its look's motion at its clip's moment, as a
/// caption's words are: `text` set at `size` about `centre`, its lines lined
/// up by the record's alignment `align`, each piece as solid as the text's
/// own reveal leaves it, so a text with no exit of its own still goes with
/// its box.
fn moving_type(
  text: &str,
  motion: &NativeTextMotion,
  setting: (TextItemSetting, u32),
  reveal: AnnotationReveal,
) -> PreparedArrows {
  let (mut setting, align) = setting;
  setting.align = f64::from(align.min(2)) / 2.0;
  setting.duration_ms = f64::from(motion.clock.duration_ms);
  let faces = CaptionFaces::of(&motion.look);
  let moment = CaptionMoment {
    at_ms: f64::from(motion.clock.elapsed_ms),
    exposure_ms: f64::from(motion.clock.frame_ms),
  };
  let mut pieces = text_item_pieces(
    text,
    &motion.look,
    setting,
    moment,
    &mut caption_measure(&faces),
  );
  let present = [reveal.opacity, reveal.previous[3]].map(|share| f64::from(share.clamp(0.0, 1.0)));
  for piece in &mut pieces {
    piece.opacity = [piece.opacity[0] * present[0], piece.opacity[1] * present[1]];
  }
  caption_arrows(&pieces, &faces)
}

/// Draws a text box's record into `prepared` in its look, in its place
/// among the arrows: its box, where the look has one, then its type, still
/// or, at a recording's frame where its look moves it, in that motion; a
/// box being typed into stands still, its type where the typing is. The
/// moving words' text follows the records' own in the side buffer, after
/// `moving.0`, the moving texts' so far, and `moving.1` bytes of the
/// records'. `whole` is the record prepared whole, `align` its alignment,
/// and `still` whether it was drawn without exposure samples.
pub(super) fn push(
  prepared: &mut PreparedArrows,
  moving: (&mut Vec<u8>, usize),
  (arrow, entry, look): (PreviewArrow, PreparedType, &NativeTextLook),
  (whole, align): (ArrowGeometry, u32),
  (reveal, still): (AnnotationReveal, bool),
) {
  let Some(motion) = look.motion.as_ref().filter(|_| entry.marks.is_none()) else {
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
  let text = entry.text.clone();
  if let Some((arrow, entry)) = box_alone(arrow, entry, look) {
    prepared.types.push(entry);
    prepared.arrows.push(arrow);
  }
  let setting = TextItemSetting {
    centre: whole.start_head.c.map(f64::from),
    size: f64::from(whole.width),
    align: 0.0,
    duration_ms: 0.0,
  };
  let pieces = moving_type(&text, motion, (setting, align), reveal);
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
