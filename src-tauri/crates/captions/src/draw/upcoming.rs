// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The next caption's words, readied before it comes on.

use super::{full_hex, showing, CaptionPiece, CaptionPieceKind, CaptionTrack, POP_MS};
use crate::style::CaptionBackground;
use crate::{type_size_px, CaptionFrame};

/// How long after a caption comes on the next one's words are readied: past
/// its entrance, so the work of readying them never lands on the frames it
/// animates in.
const AFTER_ENTRANCE_MS: f64 = POP_MS + 100.0;

impl CaptionTrack {
  /// The words of the caption after the one showing `at_ms` into the edited
  /// video, set as they will be but drawing nothing. Type is rasterised the
  /// first frame it is asked for, which for a caption coming on is the frame
  /// its entrance starts; asked for while the caption before still shows,
  /// once that one has come on, it is ready when that frame comes.
  pub fn upcoming_pieces(&self, frame: CaptionFrame, at_ms: f64) -> Vec<CaptionPiece> {
    let entering = showing(&self.captions, at_ms)
      .is_some_and(|caption| at_ms - caption.start_ms < AFTER_ENTRANCE_MS);
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
    // The word on a pill is set without its outline, so it is readied both
    // ways.
    let mut outlines = vec![outline.clone()];
    if style.background == CaptionBackground::Word && outline.is_some() {
      outlines.push(None);
    }
    next
      .lines
      .iter()
      .flatten()
      .flat_map(|word| {
        outlines.iter().map(move |outline| CaptionPiece {
          kind: CaptionPieceKind::Word {
            ink: "#00000000".to_owned(),
            outline: outline.clone(),
          },
          text: word.text.clone(),
          centre: [0.0; 2],
          size,
          scale: [1.0; 2],
          opacity: [0.0; 2],
        })
      })
      .collect()
  }
}
