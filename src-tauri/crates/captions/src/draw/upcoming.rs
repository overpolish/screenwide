// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The next caption's words, readied before it comes on.

use super::glow::glow_spread;
use super::{full_hex, showing, CaptionPiece, CaptionPieceKind, CaptionPieceShadow, CaptionTrack};
use crate::style::{CaptionBackground, CaptionFace};
use crate::{type_size_px, CaptionFrame};

/// How long after a caption's motion settles the next one's words are
/// readied, so the work of readying them never lands on the frames it
/// animates in.
const AFTER_MOTION_MS: f64 = 100.0;

impl CaptionTrack {
  /// The words of the caption after the one showing `at_ms` into the edited
  /// video, set as they will be but drawing nothing. Type is rasterised the
  /// first frame it is asked for, which for a caption coming on is the frame
  /// its entrance starts; asked for while the caption before still shows,
  /// once that one has come on, it is ready when that frame comes.
  pub fn upcoming_pieces(&self, frame: CaptionFrame, at_ms: f64) -> Vec<CaptionPiece> {
    let settles_ms = self.recording.style.motion.settles_ms() + AFTER_MOTION_MS;
    let entering =
      showing(&self.captions, at_ms).is_some_and(|caption| at_ms - caption.start_ms < settles_ms);
    let after = self
      .captions
      .partition_point(|caption| caption.start_ms <= at_ms);
    let Some(next) = self
      .captions
      .get(after)
      .filter(|_| self.recording.enabled && !entering)
    else {
      return Vec::new();
    };
    let style = self.recording.style.normalized();
    let size = type_size_px(&style, frame);
    let outline = style.outline.then(|| full_hex(&style.outline_color));
    let softness =
      glow_spread(&style).or_else(|| style.shadow.as_ref().map(|shadow| shadow.softness));
    // The word on a pill is set without its outline or shadow, so it is
    // readied both ways.
    let mut looks = vec![(outline.clone(), softness)];
    if style.background == CaptionBackground::Word && (outline.is_some() || softness.is_some()) {
      looks.push((None, None));
    }
    next
      .lines
      .iter()
      .flatten()
      .flat_map(|word| {
        let face = if word.emphasis {
          CaptionFace::Accent
        } else {
          CaptionFace::Main
        };
        let size = size * style.scale_of(face);
        // The cell the type is set in only depends on how soft its shadow
        // is, so that is all a readied word carries of it.
        looks.iter().map(move |(outline, softness)| CaptionPiece {
          kind: CaptionPieceKind::Word {
            ink: "#00000000".to_owned(),
            outline: outline.clone(),
            shadow: softness.map(|softness| CaptionPieceShadow {
              color: "#00000000".to_owned(),
              offset: [0.0; 2],
              softness: softness * size,
            }),
          },
          text: word.text.clone(),
          face,
          centre: [0.0; 2],
          previous_centre: [0.0; 2],
          size,
          scale: [1.0; 2],
          opacity: [0.0; 2],
          turn: [[0.0; 3]; 2],
          line_turn: None,
        })
      })
      .collect()
  }
}
