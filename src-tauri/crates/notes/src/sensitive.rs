// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What in the notes may be private. Things written in a fixed form, emails,
//! links, card and account numbers and phone numbers, are found by their
//! form, without a model; names, companies and addresses, and emails and
//! numbers said in words, by the model the notes download brings
//! (`screenwide_speech::runner::sensitive`); and the special kinds, such as
//! health, by the words that say them (`special`). Where a form and the
//! model find the same words, the form's find stands: it knows exactly
//! where a number starts and ends. Where the model and a special word do,
//! the model's stands, so a person called Christian stays a name.

mod special;

use std::sync::LazyLock;

use regex::Regex;
use screenwide_speech::runner::sensitive::{SensitiveKind, SensitiveSpan};

static EMAIL: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}")
    .expect("a valid pattern")
});
static LINK: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(?:https?://|www\.)[^\s<>]+").expect("a valid pattern"));
static IBAN: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"\b[A-Z]{2}\d{2}(?: ?[A-Z0-9]{4}){2,7}(?: ?[A-Z0-9]{1,4})?\b")
    .expect("a valid pattern")
});
static SORT_CODE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\b\d{2}-\d{2}-\d{2}\b").expect("a valid pattern"));
/// A run of digits with the spaces, dashes, dots and brackets numbers are
/// written with, which is a card number when its checksum holds and a phone
/// number when it is the length of one.
static NUMBER: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\+?\(?\d[\d ().-]{6,}\d").expect("a valid pattern"));

/// What in `text` is written in the form of something private, in order.
pub fn patterns(text: &str) -> Vec<SensitiveSpan> {
  let mut found: Vec<(usize, usize, SensitiveKind)> = Vec::new();
  let mut add = |start: usize, end: usize, kind: SensitiveKind| {
    if found.iter().all(|&(a, b, _)| end <= a || start >= b) {
      found.push((start, end, kind));
    }
  };
  for m in EMAIL.find_iter(text) {
    add(m.start(), m.end(), SensitiveKind::Email);
  }
  for m in LINK.find_iter(text) {
    // A sentence's own punctuation after a link is not part of it.
    let link = m
      .as_str()
      .trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '"', '\'']);
    add(m.start(), m.start() + link.len(), SensitiveKind::Link);
  }
  for m in IBAN.find_iter(text) {
    add(m.start(), m.end(), SensitiveKind::Account);
  }
  for m in SORT_CODE.find_iter(text) {
    add(m.start(), m.end(), SensitiveKind::Account);
  }
  for m in NUMBER.find_iter(text) {
    let digits: Vec<u32> = m.as_str().chars().filter_map(|c| c.to_digit(10)).collect();
    if (13..=19).contains(&digits.len()) && luhn(&digits) {
      add(m.start(), m.end(), SensitiveKind::Card);
    } else if (9..=15).contains(&digits.len()) {
      add(m.start(), m.end(), SensitiveKind::Phone);
    }
  }
  spans(text, found)
}

/// Where `text` says something of a special kind, such as health, in
/// order.
pub fn special_words(text: &str) -> Vec<SensitiveSpan> {
  spans(text, special::special(text))
}

/// `found`, as `[start, end)` bytes of `text`, in order and in UTF-16 code
/// units.
fn spans(text: &str, mut found: Vec<(usize, usize, SensitiveKind)>) -> Vec<SensitiveSpan> {
  found.sort_unstable();
  found
    .into_iter()
    .map(|(start, end, kind)| SensitiveSpan {
      start: utf16_offset(text, start),
      end: utf16_offset(text, end),
      kind,
    })
    .collect()
}

/// What the forms, the model and the special words found, as one list in
/// order: where they overlap, a form's find, then the model's.
pub fn merged(
  model: Vec<SensitiveSpan>,
  forms: Vec<SensitiveSpan>,
  special: Vec<SensitiveSpan>,
) -> Vec<SensitiveSpan> {
  let mut all = forms;
  for span in model.into_iter().chain(special) {
    if all
      .iter()
      .all(|kept| span.end <= kept.start || span.start >= kept.end)
    {
      all.push(span);
    }
  }
  all.sort_unstable_by_key(|span| (span.start, span.end));
  all
}

/// Whether `digits` pass the check card numbers carry.
fn luhn(digits: &[u32]) -> bool {
  let sum: u32 = digits
    .iter()
    .rev()
    .enumerate()
    .map(|(index, &digit)| {
      if index % 2 == 1 {
        let doubled = digit * 2;
        if doubled > 9 {
          doubled - 9
        } else {
          doubled
        }
      } else {
        digit
      }
    })
    .sum();
  sum.is_multiple_of(10)
}

fn utf16_offset(text: &str, byte: usize) -> usize {
  text[..byte].encode_utf16().count()
}

#[cfg(test)]
mod tests;
