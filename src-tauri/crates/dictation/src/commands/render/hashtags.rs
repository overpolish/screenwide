// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! "Hashtag" said before a word or number, written as the mark joined to it
//! ("closes hashtag 50" as "closes #50"). It is said inside a sentence, not
//! on its own as a command is, so the words round it tell the mark from the
//! noun: "add a hashtag to the post" stays as said.

use screenwide_speech::runner::dictation::DictatedWord;

use super::super::{plain, Phrases, Step};

/// The places among the first `upto` of `words` of each "hashtag" written
/// as the mark on the word after it.
pub(super) fn hashtags(
  words: &[DictatedWord],
  steps: &[Step],
  (upto, phrases): (usize, &Phrases),
) -> Vec<usize> {
  let Some(hashtag) = &phrases.hashtag else {
    return Vec::new();
  };
  let mut marks = Vec::new();
  for (place, step) in steps.iter().enumerate() {
    let Step::Word(at) = *step else {
      continue;
    };
    if at >= upto {
      break;
    }
    // Written with a mark after it ("hashtag, …"), it is not joined on.
    if !words[at].text.eq_ignore_ascii_case(hashtag.said) {
      continue;
    }
    let word_at = |place: Option<usize>| match place.and_then(|place| steps.get(place)) {
      Some(&Step::Word(word)) if word < upto => Some(plain(&words[word].text)),
      _ => None,
    };
    let Some(after) = word_at(Some(place + 1)).filter(|word| !word.is_empty()) else {
      continue;
    };
    let before = word_at(place.checked_sub(1));
    let noun = before.is_some_and(|word| hashtag.before_it.contains(&word.as_str()))
      || hashtag.after_it.contains(&after.as_str());
    if !noun {
      marks.push(at);
    }
  }
  marks
}
