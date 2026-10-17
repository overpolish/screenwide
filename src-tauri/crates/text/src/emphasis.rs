// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The words of a text chosen to be set apart: in the accent where its look
//! sets a key word apart, or highlighted where its look highlights words.
//! Each is kept as the byte range of one whole word of the text, a word
//! being a run of characters between white space. An edit carries them to
//! the words they become, so typing into or beside a word keeps it chosen.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::style::{TextAccent, TextStyle};

/// One word of a text, by the byte range it takes in the text's UTF-8.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, TS)]
#[ts(export)]
pub struct TextSpan {
  pub start: usize,
  pub end: usize,
}

/// Whether `look` draws the words chosen in a text any differently: in its
/// accent, or highlighted.
pub fn sets_apart(look: &TextStyle) -> bool {
  look.accent == TextAccent::Word || look.highlight_words
}

/// Each word of `text`, in order.
pub fn words(text: &str) -> impl Iterator<Item = TextSpan> + '_ {
  let mut start = None;
  text
    .char_indices()
    .map(Some)
    .chain([None])
    .filter_map(move |at| match at {
      Some((index, character)) if !character.is_whitespace() => {
        start.get_or_insert(index);
        None
      }
      Some((index, _)) => start.take().map(|start| TextSpan { start, end: index }),
      None => start.take().map(|start| TextSpan {
        start,
        end: text.len(),
      }),
    })
}

/// Whether the word `word` is among `chosen`.
pub fn is_chosen(chosen: &[TextSpan], word: TextSpan) -> bool {
  chosen.binary_search(&word).is_ok()
}

/// The words of `text` that `spans` reach into, in order: what a text's
/// choice is held as, whatever ranges it was read with.
pub fn chosen_words(text: &str, spans: &[TextSpan]) -> Vec<TextSpan> {
  words(text)
    .filter(|word| {
      spans
        .iter()
        .any(|span| span.start < span.end && span.start < word.end && word.start < span.end)
    })
    .collect()
}

/// `chosen`, words of `before`, carried to the words of `after` they become.
/// The edit is read as one change between the two texts' common start and
/// common end. A word the change replaces part of reaches over what took
/// its place; one typed beside a chosen word joins it only where nothing
/// parts the two; one the change removes whole is gone.
pub fn follow_edit(chosen: &[TextSpan], before: &str, after: &str) -> Vec<TextSpan> {
  if before == after {
    return chosen_words(after, chosen);
  }
  let mut prefix = before
    .bytes()
    .zip(after.bytes())
    .take_while(|(a, b)| a == b)
    .count();
  while !before.is_char_boundary(prefix) || !after.is_char_boundary(prefix) {
    prefix -= 1;
  }
  let room = before.len().min(after.len()) - prefix;
  let mut suffix = before
    .bytes()
    .rev()
    .zip(after.bytes().rev())
    .take(room)
    .take_while(|(a, b)| a == b)
    .count();
  while !before.is_char_boundary(before.len() - suffix)
    || !after.is_char_boundary(after.len() - suffix)
  {
    suffix -= 1;
  }
  let (old_end, new_end) = (before.len() - suffix, after.len() - suffix);
  let shift = |at: usize| at + after.len() - before.len();
  let moved: Vec<TextSpan> = chosen
    .iter()
    .map(|span| TextSpan {
      // A change right before a word pushes it along, one right after
      // leaves it where it ends.
      start: if span.start >= old_end {
        shift(span.start)
      } else if span.start <= prefix {
        span.start
      } else {
        prefix
      },
      end: if span.end <= prefix {
        span.end
      } else if span.end >= old_end {
        shift(span.end)
      } else {
        new_end
      },
    })
    .collect();
  chosen_words(after, &moved)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn span(start: usize, end: usize) -> TextSpan {
    TextSpan { start, end }
  }

  /// The chosen words of `after` once `chosen`, given as words of `before`,
  /// follow the edit, as text.
  fn followed<'a>(before: &str, chosen: &[&str], after: &'a str) -> Vec<&'a str> {
    let chosen: Vec<TextSpan> = words(before)
      .filter(|word| chosen.contains(&&before[word.start..word.end]))
      .collect();
    follow_edit(&chosen, before, after)
      .iter()
      .map(|word| &after[word.start..word.end])
      .collect()
  }

  #[test]
  fn reads_words_between_any_white_space() {
    let text = "  one\ttwo\nthree  é ";
    let found: Vec<&str> = words(text)
      .map(|word| &text[word.start..word.end])
      .collect();
    assert_eq!(found, ["one", "two", "three", "é"]);
    assert_eq!(words("").count(), 0);
  }

  #[test]
  fn holds_a_choice_as_the_whole_words_it_reaches_into() {
    let text = "save your changes";
    assert_eq!(chosen_words(text, &[span(6, 7)]), [span(5, 9)]);
    // Reaching over a space takes both words; resting on space alone none.
    assert_eq!(chosen_words(text, &[span(3, 7)]), [span(0, 4), span(5, 9)]);
    assert!(chosen_words(text, &[span(4, 5), span(9, 9)]).is_empty());
  }

  #[test]
  fn carries_chosen_words_through_typing() {
    // Typed onto either end of a chosen word, the word grows and stays.
    assert_eq!(followed("cat dog", &["cat"], "cats dog"), ["cats"]);
    assert_eq!(followed("cat dog", &["dog"], "cat hotdog"), ["hotdog"]);
    // A new word typed before or after one stays plain.
    assert_eq!(followed("cat", &["cat"], "the cat"), ["cat"]);
    assert_eq!(followed("cat", &["cat"], "cat nap"), ["cat"]);
    // A word earlier in the text moves the chosen ones after it along.
    assert_eq!(followed("a big cat", &["cat"], "a very big cat"), ["cat"]);
    // A space typed inside a chosen word leaves both halves chosen.
    assert_eq!(followed("catdog", &["catdog"], "cat dog"), ["cat", "dog"]);
    // Part of a chosen word typed over keeps the word.
    assert_eq!(followed("big cat", &["cat"], "big cow"), ["cow"]);
    // A chosen word deleted whole is gone; its neighbours stay as they were.
    assert_eq!(
      followed("big cat now", &["cat"], "big now"),
      Vec::<&str>::new()
    );
    assert_eq!(
      followed("big cat now", &["big", "now"], "big now"),
      ["big", "now"]
    );
    // Characters of more than one byte keep every range on a boundary.
    assert_eq!(
      followed("café au lait", &["lait"], "cafés au lait"),
      ["lait"]
    );
  }
}
