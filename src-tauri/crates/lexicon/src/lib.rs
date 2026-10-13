// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a transcript's words are: profanity and hesitation sounds, by the
//! language the transcriber reports. The Transcript panel marks them and the
//! captions hide them, both through here, so the two never disagree about
//! which words they are.

mod fillers;
mod fold;
mod profanity;

pub use fillers::find_fillers;
pub use fold::{fold_filler_text, fold_profanity_text};
pub use profanity::{find_profanity, ProfanityLexicon};

/// One word of a transcript as it reads now: a word as heard, or the words a
/// correction covers under its text. Pauses are never units.
#[derive(Clone, Copy, Debug)]
pub struct Unit<'a> {
  pub text: &'a str,
  pub start_ms: f64,
  pub end_ms: f64,
  /// Cut out by the edit: never a finding.
  pub removed: bool,
}

/// A run of units, `first` to `last` by index, and the folded text it
/// matched, so every finding alike can be found together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
  pub folded: String,
  pub first: usize,
  pub last: usize,
}

#[cfg(test)]
mod tests;
