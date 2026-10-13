// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The captions showing in a frame, drawn by the annotation shader as text
//! boxes: each piece `screenwide_captions` lays out becomes a box placed in
//! canvas pixels, set in the caption style's face and blurred as it grows
//! the way an annotation is, and flagged for the parts a caption draws
//! differently - a word's type without its box, and a box without type.

use super::arrows::PreviewGeometry;
use super::{placed_arrows, PreparedArrows};
use screenwide_annotations::flags::{BARE, CAPTION, SHADOW};
use screenwide_annotations::reveal::AnnotationReveal;
use screenwide_annotations::text::face::TypeFace;
use screenwide_annotations::text::geometry::box_size;
use screenwide_annotations::text::metrics::{face_text_block, text_block};
use screenwide_annotations::text::model::TextPointer;
use screenwide_annotations::{
  Annotation, AnnotationAlign, AnnotationHead, AnnotationPoint, AnnotationRedaction,
  AnnotationShape, AnnotationStyle,
};
use screenwide_captions::{
  CaptionFont, CaptionFrame, CaptionPiece, CaptionPieceKind, CaptionTrack,
};

/// The face a caption font names, as the text engines take it.
pub fn caption_face(font: &CaptionFont) -> TypeFace {
  TypeFace::new(font.family.as_deref(), font.weight)
}

/// How wide `text` is set in `face` at `size`, both in pixels: what the
/// captions measure their words by, so they lay out by the widths drawn.
pub fn caption_measure(face: &TypeFace) -> impl FnMut(&str, f64) -> f64 + '_ {
  move |text, size| face_text_block(face, text, size)[0]
}

/// A caption piece as a text box centred where the piece is, growing about
/// that centre by its scale. Placing sizes a text box by Inter, so it is
/// placed by Inter's size too, which keeps its centre on the piece's; its
/// width is set to the face's after.
fn text_box(index: usize, piece: &CaptionPiece) -> Annotation {
  let color = match &piece.kind {
    CaptionPieceKind::Backdrop { color } | CaptionPieceKind::Pill { color } => color,
    CaptionPieceKind::Word { ink, .. } => ink,
  };
  let [width, height] = box_size(text_block(&piece.text, piece.size), piece.size);
  let as_f32 = |value: f64| value as f32;
  Annotation {
    above_camera: true,
    animated: true,
    id: format!("caption-{index}"),
    held: None,
    pen: false,
    reveal: AnnotationReveal {
      low: 0.0,
      high: 1.0,
      scale: as_f32(piece.scale[0]),
      opacity: as_f32(piece.opacity[0]),
      previous: [0.0, 1.0, as_f32(piece.scale[1]), as_f32(piece.opacity[1])],
    },
    shape: AnnotationShape::Text {
      origin: AnnotationPoint {
        x: piece.centre[0] - width / 2.0,
        y: piece.centre[1] - height / 2.0,
      },
      pointer: TextPointer::default(),
      text: piece.text.clone(),
    },
    style: AnnotationStyle {
      align: AnnotationAlign::Center,
      blur: false,
      color: color.clone(),
      head: AnnotationHead::default(),
      hand_drawn: false,
      manual: false,
      radius: 0.0,
      redaction: AnnotationRedaction::default(),
      softness: 0.0,
      shadow: false,
      strength: 0.0,
      tint: false,
      width: piece.size,
    },
  }
}

/// How much closer than a text box hugs its text a pill hugs its word
/// across, in ems: a box's padding would reach under the neighbours. That
/// leaves the pill reaching [`PILL_REACH`] past its word.
const PILL_INSET: f32 = 0.3;
const PILL_REACH: f32 = 0.2;
/// How tall a pill is, in ems: shorter than the 1.25 em lines are apart,
/// so a pill on one line stays off the type of the line above and below.
const PILL_HEIGHT: f32 = 1.1;
/// A pill's corners, in ems; never more than half its side, so a pill
/// under a single narrow letter stays a rounded box rather than a lens.
const PILL_RADIUS: f32 = 0.25;
/// The most of the space beside its word a pill may cover, so the next
/// word never seems to sit on it.
const PILL_SPACE_SHARE: f32 = 0.45;

/// How much further than [`PILL_INSET`] a pill in `face` at `size` is drawn
/// in on each side, in ems: none where the face's space has room for its
/// reach, otherwise enough to keep it off the neighbours.
fn pill_pull(face: &TypeFace, size: f64) -> f32 {
  let space = face_text_block(face, " ", size)[0] / size.max(1.0);
  (PILL_REACH - space as f32 * PILL_SPACE_SHARE).max(0.0)
}

/// `geometry`'s text box drawn in to its pill: [`PILL_INSET`] and `pull`
/// more across, [`PILL_HEIGHT`] tall about its middle, with its corners.
/// Its width is its type size, grown as the pill is.
fn inset(geometry: &mut PreviewGeometry, pull: f32) {
  let em = geometry.width;
  let x = (PILL_INSET + pull) * em;
  geometry.ax += x;
  geometry.bx -= x;
  let y = ((geometry.by - geometry.ay) - PILL_HEIGHT * em).max(0.0) * 0.5;
  geometry.ay += y;
  geometry.by -= y;
  let half_side = (geometry.bx - geometry.ax).min(geometry.by - geometry.ay) * 0.5;
  geometry.rounding = (PILL_RADIUS * em).min(half_side).max(0.0);
}

/// `geometry`'s box made `ratio` times as wide about its centre.
fn widen(geometry: &mut PreviewGeometry, ratio: f32) {
  let centre = (geometry.ax + geometry.bx) * 0.5;
  let half = (geometry.bx - geometry.ax) * 0.5 * ratio;
  geometry.ax = centre - half;
  geometry.bx = centre + half;
  geometry.rounding = geometry.rounding.min(half).max(0.0);
}

/// How many times wider `piece`'s box is set in `face` than in the
/// annotations' Inter. Placing a text box sizes it by Inter, which is all
/// an annotation is ever set in; a caption's box has to hold its own type.
fn face_ratio(piece: &CaptionPiece, face: &TypeFace) -> f32 {
  let width = |block: [f64; 2]| box_size(block, piece.size)[0];
  let inter = width(text_block(&piece.text, piece.size));
  let own = width(face_text_block(face, &piece.text, piece.size));
  if inter > 0.0 {
    (own / inter) as f32
  } else {
    1.0
  }
}

/// How wide a caption word's outline is, in ems.
const OUTLINE: f32 = 0.08;

/// The caption pieces showing in a frame, set in `face`, prepared to draw
/// over everything else on its canvas, whose pixels the pieces are laid out
/// in.
pub fn caption_arrows(pieces: &[CaptionPiece], face: &TypeFace) -> PreparedArrows {
  let boxes: Vec<Annotation> = pieces
    .iter()
    .enumerate()
    .map(|(index, piece)| text_box(index, piece))
    .collect();
  let mut prepared = placed_arrows(&boxes, (0.0, 0.0), (1.0, 1.0), 1.0, None, None);
  let pieces_in_place = prepared
    .arrows
    .iter_mut()
    .zip(prepared.types.iter_mut())
    .zip(pieces);
  for ((arrow, entry), piece) in pieces_in_place {
    entry.face = face.clone();
    match piece.kind {
      // A box or a pill draws no type, so the atlas sets none for it.
      CaptionPieceKind::Backdrop { .. } | CaptionPieceKind::Pill { .. } => {
        arrow.flags |= BARE;
        entry.text.clear();
      }
      CaptionPieceKind::Word { outlined, .. } => {
        arrow.flags |= CAPTION;
        if outlined {
          arrow.flags |= SHADOW;
          entry.outline = OUTLINE * entry.size;
        }
      }
    }
    let ratio = face_ratio(piece, face);
    let pull =
      matches!(piece.kind, CaptionPieceKind::Pill { .. }).then(|| pill_pull(face, piece.size));
    let first = arrow.sample_first as usize;
    let samples = &mut prepared.samples[first..first + arrow.sample_count as usize];
    for geometry in std::iter::once(&mut arrow.geometry)
      .chain(samples.iter_mut().map(|sample| &mut sample.geometry))
    {
      widen(geometry, ratio);
      if let Some(pull) = pull {
        inset(geometry, pull);
      }
    }
  }
  prepared
}

/// What `track` draws `output_ms` into the edited video on a canvas `canvas`
/// in size, ready to draw over a frame's own annotations.
pub fn caption_overlay(track: &CaptionTrack, output_ms: f64, canvas: (u32, u32)) -> PreparedArrows {
  let frame = CaptionFrame {
    width: canvas.0,
    height: canvas.1,
  };
  let face = caption_face(&track.recording.style.normalized().font);
  let pieces = track.pieces_at(frame, output_ms, &mut caption_measure(&face));
  caption_arrows(&pieces, &face)
}

impl PreparedArrows {
  /// Draws `above`, prepared in the same canvas pixels, over everything
  /// these draw, the camera included. Only text boxes are taken: their
  /// type follows in the side buffer, and they read no points.
  pub fn draw_above(&mut self, above: PreparedArrows) {
    let samples = self.samples.len() as u32;
    let text = self.text.len() as u32;
    self
      .arrows
      .extend(above.arrows.into_iter().map(|mut arrow| {
        arrow.sample_first += samples;
        arrow.data_offset += text;
        arrow
      }));
    self.types.extend(above.types);
    self.samples.extend(above.samples);
    self.text.extend(above.text);
    if self.pixel_scale <= 0.0 {
      self.pixel_scale = above.pixel_scale;
    }
  }
}
