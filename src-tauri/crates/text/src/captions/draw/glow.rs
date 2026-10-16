// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The glow around the word being said: the word's outline blurred wide, in
//! the highlight colour, under its type. As each word is said its glow
//! fades in while the word before's fades out, so the glow passes along
//! rather than jumping. It is cast as a shadow that does not fall, which
//! the compositor already draws.

use super::CaptionPieceShadow;
use crate::style::{rgba, TextBackground, TextStyle};

/// How long a glow takes to fade in, and the one before it out.
const FADE_MS: f64 = 120.0;
/// How far a glow spreads, in ems of its word's type.
const SPREAD: f64 = 0.35;

/// How far the glow spreads in a style that glows, in ems: every word is
/// set with it, glowing or not, so a word's type is set once.
pub(crate) fn glow_spread(style: &TextStyle) -> Option<f64> {
  (style.highlight_words && style.background == TextBackground::Glow).then_some(SPREAD)
}

/// The glow word `index` casts at `at_ms`, set at `size`, where `said` is
/// the word being said and `said_ms` when it started: in full a fade past
/// that start, the word before fading out as it fades in, and every other
/// word casting one too faint to see, so its type stays set the same.
pub(super) fn glow(
  style: &TextStyle,
  (index, size): (usize, f64),
  said: Option<(usize, f64)>,
  at_ms: f64,
) -> Option<CaptionPieceShadow> {
  let spread = glow_spread(style)?;
  let strength = said.map_or(0.0, |(said, said_ms)| {
    let arrived = ((at_ms - said_ms) / FADE_MS).clamp(0.0, 1.0);
    if index == said {
      arrived
    } else if index + 1 == said {
      1.0 - arrived
    } else {
      0.0
    }
  });
  let [red, green, blue, alpha] = rgba(&style.highlight_color).unwrap_or([1.0; 4]);
  let faded = [red, green, blue, alpha * strength as f32]
    .map(|channel| format!("{:02x}", (channel * 255.0).round() as u8));
  Some(CaptionPieceShadow {
    color: format!("#{}", faded.concat()),
    offset: [0.0; 2],
    softness: spread * size,
  })
}
