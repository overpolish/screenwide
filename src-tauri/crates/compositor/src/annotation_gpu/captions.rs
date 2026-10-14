// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The captions showing in a frame, drawn by the annotation shader as text
//! boxes: each piece `screenwide_captions` lays out becomes a box placed in
//! canvas pixels, set in the caption style's main face or its accent and
//! blurred as it moves the way an annotation is, and flagged for the parts a
//! caption draws differently - a word's type without its box, and a box
//! without type.

use super::arrows::PreviewGeometry;
use super::caption_exposure::expose;
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
  rgba, CaptionFace, CaptionFont, CaptionFrame, CaptionMoment, CaptionPiece, CaptionPieceKind,
  CaptionStyle, CaptionTrack,
};

/// The face a caption font names, as the text engines take it.
pub fn caption_face(font: &CaptionFont) -> TypeFace {
  TypeFace::new(font.family.as_deref(), font.weight, font.italic)
}

/// The two faces a caption style sets its type in.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionFaces {
  pub main: TypeFace,
  pub accent: TypeFace,
}

impl CaptionFaces {
  pub fn of(style: &CaptionStyle) -> Self {
    let style = style.normalized();
    Self {
      main: caption_face(&style.font),
      accent: caption_face(&style.accent_font),
    }
  }

  pub fn face(&self, face: CaptionFace) -> &TypeFace {
    match face {
      CaptionFace::Main => &self.main,
      CaptionFace::Accent => &self.accent,
    }
  }
}

/// How wide `text` is set in one of `faces` at `size`, both in pixels: what
/// the captions measure their words by, so they lay out by the widths drawn.
pub fn caption_measure(faces: &CaptionFaces) -> impl FnMut(&str, CaptionFace, f64) -> f64 + '_ {
  move |text, face, size| face_text_block(faces.face(face), text, size)[0]
}

/// A caption piece as a text box centred where the piece is, at its full
/// size and solid: [`expose`] draws it as large and as solid as it is, and
/// blurs it along the way it came. Placing sizes a text box by Inter, so it
/// is placed by Inter's size too, which keeps its centre on the piece's;
/// its width is set to the face's after.
fn text_box(index: usize, piece: &CaptionPiece) -> Annotation {
  let color = match &piece.kind {
    CaptionPieceKind::Backdrop { color, .. } | CaptionPieceKind::Pill { color } => color,
    CaptionPieceKind::Word { ink, .. } => ink,
  };
  let [width, height] = box_size(text_block(&piece.text, piece.size), piece.size);
  Annotation {
    above_camera: true,
    animated: true,
    id: format!("caption-{index}"),
    held: None,
    pen: false,
    reveal: AnnotationReveal::WHOLE,
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

/// `geometry`'s box made `size` about its centre: a backdrop's, around the
/// block its caption's words were laid out in.
fn resize(geometry: &mut PreviewGeometry, [width, height]: [f32; 2]) {
  let centre = [
    (geometry.ax + geometry.bx) * 0.5,
    (geometry.ay + geometry.by) * 0.5,
  ];
  geometry.ax = centre[0] - width * 0.5;
  geometry.bx = centre[0] + width * 0.5;
  geometry.ay = centre[1] - height * 0.5;
  geometry.by = centre[1] + height * 0.5;
  geometry.rounding = geometry.rounding.min(width.min(height) * 0.5).max(0.0);
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

/// The caption pieces showing in a frame, each set in its face of `faces`,
/// prepared to draw over everything else on its canvas, whose pixels the
/// pieces are laid out in.
pub fn caption_arrows(pieces: &[CaptionPiece], faces: &CaptionFaces) -> PreparedArrows {
  let boxes: Vec<Annotation> = pieces
    .iter()
    .enumerate()
    .map(|(index, piece)| text_box(index, piece))
    .collect();
  let mut prepared = placed_arrows(&boxes, (0.0, 0.0), (1.0, 1.0), 1.0, None, None);
  let PreparedArrows {
    arrows,
    types,
    samples,
    ..
  } = &mut prepared;
  for ((arrow, entry), piece) in arrows.iter_mut().zip(types.iter_mut()).zip(pieces) {
    let face = faces.face(piece.face);
    entry.face = face.clone();
    match &piece.kind {
      // A box or a pill draws no type, so the atlas sets none for it.
      CaptionPieceKind::Backdrop { .. } | CaptionPieceKind::Pill { .. } => {
        arrow.flags |= BARE;
        entry.text.clear();
      }
      CaptionPieceKind::Word { outline, .. } => {
        arrow.flags |= CAPTION;
        // Set at the size it settles at, so a word growing in is the same
        // type every frame and the atlas rasterises it once; the shader
        // reads that size from `cx`, which a text box leaves unused, and
        // draws the type grown from it.
        entry.size = piece.size as f32;
        arrow.geometry.cx = entry.size;
        // A text box keeps nothing in its parameters either, so a caption
        // word carries its outline's colour there for the shader.
        if let Some([red, green, blue, _]) = outline.as_deref().and_then(rgba) {
          arrow.flags |= SHADOW;
          arrow.params = [red, green, blue];
          entry.outline = OUTLINE * entry.size;
        }
      }
    }
    // A backdrop is sized by the block its words were laid out in, in
    // their own faces; anything else by its text set in its face.
    let backdrop = match piece.kind {
      CaptionPieceKind::Backdrop { block, .. } => {
        Some(box_size(block, piece.size).map(|side| side as f32))
      }
      _ => None,
    };
    let ratio = face_ratio(piece, face);
    let pull =
      matches!(piece.kind, CaptionPieceKind::Pill { .. }).then(|| pill_pull(face, piece.size));
    let geometry = &mut arrow.geometry;
    if let Some(size) = backdrop {
      resize(geometry, size);
    } else {
      widen(geometry, ratio);
      if let Some(pull) = pull {
        inset(geometry, pull);
      }
    }
    expose(arrow, piece, backdrop.is_none(), samples);
  }
  prepared
}

/// What `track` draws at `moment` into the edited video on a canvas
/// `canvas` in size, ready to draw over a frame's own annotations, with the
/// next caption's words asked for unseen so their type is ready when it
/// comes on.
pub fn caption_overlay(
  track: &CaptionTrack,
  moment: CaptionMoment,
  canvas: (u32, u32),
) -> PreparedArrows {
  let frame = CaptionFrame {
    width: canvas.0,
    height: canvas.1,
  };
  let faces = CaptionFaces::of(&track.recording.style);
  let mut pieces = track.pieces_at(frame, moment, &mut caption_measure(&faces));
  pieces.extend(track.upcoming_pieces(frame, moment.at_ms));
  caption_arrows(&pieces, &faces)
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
