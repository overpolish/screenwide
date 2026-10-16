// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the caption showing is drawn, for the live preview's hit-testing.

use super::captions::{caption_measure, CaptionFaces};
use screenwide_annotations::text::geometry::box_size;
use screenwide_text::captions::{CaptionFrame, CaptionMoment, CaptionPieceKind, CaptionTrack};

/// A line's height in ems; the twin of `LINE_HEIGHT` in the captions' layout.
const LINE_HEIGHT: f64 = 1.25;

/// The block the caption showing at `moment` takes on a `canvas`-sized
/// picture, as x, y, width and height shares of it; None while no caption
/// shows. A press is aimed at the words, so each piece counts as the box its
/// type or backdrop takes at its current size, leaving out turns and shadows.
pub fn caption_visible_bounds(
  track: &CaptionTrack,
  moment: CaptionMoment,
  canvas: (u32, u32),
) -> Option<[f64; 4]> {
  if canvas.0 == 0 || canvas.1 == 0 {
    return None;
  }
  let frame = CaptionFrame {
    width: canvas.0,
    height: canvas.1,
  };
  let faces = CaptionFaces::of(&track.recording.style);
  let mut measure = caption_measure(&faces);
  let pieces = track.pieces_at(frame, moment, &mut measure);
  let mut bounds = [
    f64::INFINITY,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NEG_INFINITY,
  ];
  for piece in pieces.iter().filter(|piece| piece.opacity[0] > 0.01) {
    let size = match piece.kind {
      CaptionPieceKind::Backdrop { block, .. } => box_size(block, piece.size),
      CaptionPieceKind::Underline { block, .. } => block,
      CaptionPieceKind::Word { .. } | CaptionPieceKind::Pill { .. } => [
        measure(&piece.text, piece.face, piece.size),
        piece.size * LINE_HEIGHT,
      ],
    };
    let half = [
      size[0] * piece.scale[0] / 2.0,
      size[1] * piece.scale[0] / 2.0,
    ];
    bounds[0] = bounds[0].min(piece.centre[0] - half[0]);
    bounds[1] = bounds[1].min(piece.centre[1] - half[1]);
    bounds[2] = bounds[2].max(piece.centre[0] + half[0]);
    bounds[3] = bounds[3].max(piece.centre[1] + half[1]);
  }
  let (width, height) = (f64::from(canvas.0), f64::from(canvas.1));
  (bounds[0].is_finite() && bounds[2] > bounds[0] && bounds[3] > bounds[1]).then(|| {
    [
      bounds[0] / width,
      bounds[1] / height,
      (bounds[2] - bounds[0]) / width,
      (bounds[3] - bounds[1]) / height,
    ]
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_placement::Placement;
  use screenwide_text::captions::{Caption, CaptionWord, RecordingCaptions};

  fn track(position: Placement) -> CaptionTrack {
    let word = |text: &str, start_ms: f64| CaptionWord {
      text: text.to_owned(),
      start_ms,
      end_ms: start_ms + 400.0,
      source_start_ms: start_ms,
      source_end_ms: start_ms + 400.0,
      emphasis: false,
    };
    let mut recording = RecordingCaptions {
      enabled: true,
      ..Default::default()
    };
    recording.style.position = position;
    CaptionTrack {
      captions: vec![Caption {
        id: 0,
        start_ms: 0.0,
        end_ms: 2_000.0,
        text: "Hello there".to_owned(),
        lines: vec![vec![word("Hello", 0.0), word("there", 500.0)]],
        spaced: true,
      }],
      recording,
    }
  }

  #[test]
  fn the_block_sits_where_the_caption_is_placed() {
    let canvas = (1_920, 1_080);
    let at = |position| {
      caption_visible_bounds(&track(position), CaptionMoment::still(900.0), canvas).unwrap()
    };
    let [x, y, width, height] = at(Placement::Bottom);
    assert!((x + width / 2.0 - 0.5).abs() < 0.01, "centred across");
    assert!(y > 2.0 / 3.0 && y + height < 1.0, "in the bottom third");
    let [x, y, ..] = at(Placement::TopLeft);
    assert!(x < 0.1 && y < 0.2, "at the top left");
  }

  #[test]
  fn nothing_shows_between_captions() {
    let bounds = caption_visible_bounds(
      &track(Placement::Bottom),
      CaptionMoment::still(5_000.0),
      (1_920, 1_080),
    );
    assert_eq!(bounds, None);
  }
}
