// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Caption pieces as the text boxes the annotation pipeline draws: each one
//! box of its own colour, placed where the piece is, and placed into drawn
//! pixels as the one arrow it already is.

use super::PreparedArrows;
use screenwide_annotations::native::native_annotations;
use screenwide_annotations::redact::native::RedactSource;
use screenwide_annotations::reveal::AnnotationReveal;
use screenwide_annotations::text::geometry::box_size;
use screenwide_annotations::text::model::{dressed_in, TextControlPoint};
use screenwide_annotations::{
  Annotation, AnnotationAlign, AnnotationHead, AnnotationPoint, AnnotationRedaction,
  AnnotationShape, AnnotationStyle,
};
use screenwide_text::captions::{CaptionPiece, CaptionPieceKind};
use screenwide_text::engine::metrics::text_block;
use screenwide_text::{TextBackground, TextStyle};

/// A caption piece as a text box centred where the piece is, at its full
/// size and solid: [`expose`] draws it as large and as solid as it is, and
/// blurs it along the way it came. Placing sizes a text box by Inter, so it
/// is placed by Inter's size too, which keeps its centre on the piece's;
/// its width is set to the face's after.
pub(super) fn text_box(index: usize, piece: &CaptionPiece) -> Annotation {
  let color = match &piece.kind {
    CaptionPieceKind::Backdrop { color, .. }
    | CaptionPieceKind::Pill { color }
    | CaptionPieceKind::Underline { color, .. } => color,
    CaptionPieceKind::Word { ink, .. } => ink,
  };
  let [width, height] = box_size(text_block(&piece.text, piece.size), piece.size);
  // A piece is one box of its own colour, its face and ink set by the
  // caption after: its look only sizes it, as Inter, and colours it.
  let mut look = TextStyle {
    background: TextBackground::Box,
    ..TextStyle::default()
  };
  dressed_in(&mut look, color);
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
      text: piece.text.clone(),
      look: Box::new(look),
      controls: std::collections::BTreeMap::<String, TextControlPoint>::new(),
      clock: None,
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

/// Caption pieces placed into drawn pixels, each text box as the one box
/// and type its piece already is: the piece set its own face, ink and look.
pub(super) fn placed_caption_pieces(annotations: &[Annotation]) -> PreparedArrows {
  if annotations.is_empty() {
    return PreparedArrows::default();
  }
  super::prepare::placed_native(
    native_annotations(annotations, RedactSource::None, 1.0),
    (0.0, 0.0),
    (1.0, 1.0),
    None,
    None,
    false,
  )
}
