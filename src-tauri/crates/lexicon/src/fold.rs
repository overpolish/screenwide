// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text as it is matched against a list: one form however it is cased,
//! composed or punctuated.

use icu_normalizer::ComposingNormalizerBorrowed;
use regex::Regex;
use std::sync::LazyLock;

static NOT_WORD: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[^\p{L}\p{M}\p{N}]+").expect("a valid pattern"));
static NOT_LETTER: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[^\p{L}\p{M}]+").expect("a valid pattern"));
/// A letter with the marks written on it, the piece a drawn-out sound
/// repeats, or marks with no letter before them, which are kept as they are.
static CLUSTER: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(\p{L}\p{M}*)|\p{M}+").expect("a valid pattern"));

fn composed_lowercase(text: &str) -> String {
  ComposingNormalizerBorrowed::new_nfc()
    .normalize(text)
    .to_lowercase()
}

/// Text as profanity is matched: no case, and nothing but letters, marks and
/// digits, so "Damn!" and "damn" fold alike.
pub fn fold_profanity_text(text: &str) -> String {
  NOT_WORD
    .replace_all(&composed_lowercase(text), "")
    .into_owned()
}

/// Text as a filler is matched: no case, no punctuation or digits, and any
/// letter said more than once in a row kept once, so "Ummm," "umm" and "um"
/// fold alike. Combining marks stay, since Thai and Hindi vowels are written
/// with them.
pub fn fold_filler_text(text: &str) -> String {
  let lowered = composed_lowercase(text);
  let letters = NOT_LETTER.replace_all(&lowered, "");
  let mut folded = String::with_capacity(letters.len());
  let mut previous = "";
  for found in CLUSTER.captures_iter(&letters) {
    match found.get(1) {
      Some(letter) if letter.as_str() == previous => {}
      Some(letter) => {
        folded.push_str(letter.as_str());
        previous = letter.as_str();
      }
      None => {
        folded.push_str(&found[0]);
        previous = "";
      }
    }
  }
  folded
}
