// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a word reads as in a caption, and how strongly the gap after it asks
//! for a caption to end there.

use crate::settings::{CaptionProfanity, CaptionSettings};
use regex::Regex;
use std::sync::LazyLock;

/// Languages written without spaces between words, whose words run together
/// on a caption line.
const UNSPACED_LANGUAGES: &[&str] = &["bo", "ja", "km", "lo", "my", "th", "yue", "zh"];

/// What a caption puts between two words on a line in `language`.
pub(crate) fn word_gap(language: Option<&str>) -> &'static str {
  if language.is_some_and(|code| UNSPACED_LANGUAGES.contains(&code)) {
    ""
  } else {
    " "
  }
}

static WORD_CHAR: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[\p{L}\p{M}\p{N}]$").expect("a valid pattern"));

/// A letter, a mark written on one, or a digit, which a word is made of;
/// anything else is its punctuation. Marks count, since Thai, Hindi and
/// Arabic write vowels and signs with them.
fn is_word_char(c: char) -> bool {
  WORD_CHAR.is_match(c.encode_utf8(&mut [0; 4]))
}

/// `word` without its punctuation. An apostrophe or a hyphen between two
/// letters belongs to the word ("don't", "well-known"), so it stays.
fn without_punctuation(word: &str) -> String {
  let chars: Vec<char> = word.chars().collect();
  let joins = |index: usize| {
    index > 0
      && is_word_char(chars[index - 1])
      && chars.get(index + 1).is_some_and(|next| is_word_char(*next))
  };
  chars
    .iter()
    .enumerate()
    .filter(|&(index, &c)| is_word_char(c) || (matches!(c, '\'' | '’' | '-' | '‐') && joins(index)))
    .map(|(_, c)| *c)
    .collect()
}

/// `word` with its letters starred out: all of them, or all but the first.
fn censored(word: &str, style: CaptionProfanity) -> String {
  let mut kept_first = style == CaptionProfanity::Hide;
  word
    .chars()
    .map(|c| {
      if !is_word_char(c) {
        c
      } else if kept_first {
        '*'
      } else {
        kept_first = true;
        c
      }
    })
    .collect()
}

/// What `word` reads as in a caption, or `None` where it is left out: a
/// filler while fillers are hidden, profanity while it is removed, and a
/// word with nothing left once its punctuation goes.
pub(crate) fn caption_text(
  word: &str,
  settings: &CaptionSettings,
  is_filler: bool,
  is_profanity: bool,
) -> Option<String> {
  if is_filler && settings.hide_fillers {
    return None;
  }
  let censor = is_profanity && settings.censor_profanity;
  if censor && settings.profanity == CaptionProfanity::Remove {
    return None;
  }
  let mut text = if settings.remove_punctuation {
    without_punctuation(word)
  } else {
    word.to_owned()
  };
  if censor {
    text = censored(&text, settings.profanity);
  }
  (!text.trim().is_empty()).then_some(text)
}

/// How strongly the gap after `word`, `pause_ms` long, asks for a caption to
/// end there, read from the word as it was said rather than as the caption
/// shows it: the end of a sentence most, then a pause, then a clause. Zero
/// where nothing does.
pub(crate) fn break_score(word: &str, pause_ms: f64) -> u8 {
  let last = word
    .trim_end_matches(['"', '\'', '’', '”', ')', ']', '」', '』'])
    .chars()
    .next_back();
  let punctuation = match last {
    Some('.' | '!' | '?' | '。' | '！' | '？' | '…') => 3,
    Some(',' | ';' | ':' | '，' | '、' | '；' | '：' | '—') => 1,
    _ => 0,
  };
  let pause = if pause_ms > PAUSE_MS { 2 } else { 0 };
  punctuation.max(pause)
}

/// A pause longer than this between two words reads as a place to break.
const PAUSE_MS: f64 = 300.0;
