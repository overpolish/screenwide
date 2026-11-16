// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether a correction kept everything the speaker meant to say. The heard
//! and corrected words are lined up, and every stretch of heard words the
//! correction left out must be one the speaker replaced: words followed by
//! one that marks them replaced ("2, actually 3"), or a start the speaker
//! made again ("on Monday, on Tuesday", "forty, forty-five"). A mark alone
//! is a word the speaker meant ("I actually enjoyed it"), and so is a
//! stretch that says "not" ("not Tuesday, Wednesday"): a contrast, not a
//! correction. A correction may write a word or two in place of ones it
//! left out, but not move a word the speaker said.

/// Words that mark what was just said as replaced: "actually", "no", "I
/// mean", "sorry", "wait", "make that", "or rather".
const MARKS: [&str; 8] = [
  "actually",
  "no",
  "mean",
  "sorry",
  "wait",
  "make",
  "rather",
  "correction",
];
/// Sounds a speaker makes that a correction may leave out anywhere.
const FILLERS: [&str; 5] = ["um", "uh", "er", "erm", "ah"];
/// The most words one replaced stretch may hold.
const MOST_REPLACED: usize = 8;
/// The most words a correction may write that were not said.
const MOST_WRITTEN: usize = 3;
/// How many words after a start made again are looked through for the
/// "and" or "or" that would make it a list's next item.
const LIST_REACH: usize = 6;

const NUMBERS: [&str; 31] = [
  "zero",
  "one",
  "two",
  "three",
  "four",
  "five",
  "six",
  "seven",
  "eight",
  "nine",
  "ten",
  "eleven",
  "twelve",
  "thirteen",
  "fourteen",
  "fifteen",
  "sixteen",
  "seventeen",
  "eighteen",
  "nineteen",
  "twenty",
  "thirty",
  "forty",
  "fifty",
  "sixty",
  "seventy",
  "eighty",
  "ninety",
  "hundred",
  "thousand",
  "million",
];

/// Whether `corrected` keeps everything said in `heard` but what the speaker
/// replaced, its line breaks too, which were said as commands, and is not
/// empty where something was said.
pub fn kept(heard: &str, corrected: &str) -> bool {
  if corrected.trim().matches('\n').count() < heard.trim().matches('\n').count() {
    return false;
  }
  let said = words(heard);
  let wrote = words(corrected);
  if said.is_empty() || wrote.is_empty() {
    return said.is_empty() == wrote.is_empty();
  }
  let matched = lined_up(&said, &wrote);
  let mut placed = vec![false; wrote.len()];
  for &place in matched.iter().flatten() {
    placed[place] = true;
  }
  let mut written = wrote
    .iter()
    .zip(&placed)
    .filter(|(_, placed)| !**placed)
    .map(|(word, _)| word);
  if written.clone().count() > MOST_WRITTEN || written.any(|word| said.contains(word)) {
    return false;
  }
  let mut at = 0;
  while at < said.len() {
    if matched[at].is_some() || FILLERS.contains(&said[at].as_str()) {
      at += 1;
      continue;
    }
    let start = at;
    while at < said.len() && matched[at].is_none() {
      at += 1;
    }
    if !replaced(&said[start..at], &said[at..]) {
      return false;
    }
  }
  true
}

/// Whether `left_out`, followed by `after`, is a stretch the speaker
/// replaced.
fn replaced(left_out: &[String], after: &[String]) -> bool {
  let negated = left_out
    .iter()
    .any(|word| word == "not" || word == "never" || word.ends_with("n't"));
  if left_out.len() > MOST_REPLACED || negated {
    return false;
  }
  if let Some(mark) = left_out
    .iter()
    .position(|word| MARKS.contains(&word.as_str()))
  {
    return mark > 0;
  }
  let (Some(first), Some(next)) = (left_out.first(), after.first()) else {
    return false;
  };
  // A start made again in a list ("a pen, a notebook and your laptop") is
  // the list's next item.
  let listed = after
    .iter()
    .take(LIST_REACH)
    .any(|word| word == "and" || word == "or");
  // Made again as said ("on Monday, on Tuesday"), or with a word put
  // before it ("faster, much faster").
  let again = after
    .iter()
    .take(2)
    .any(|word| word.starts_with(first.as_str()));
  !listed && (again || is_number(first) && is_number(next))
}

fn is_number(word: &str) -> bool {
  word.chars().all(|letter| letter.is_ascii_digit()) || NUMBERS.contains(&word)
}

/// Each heard word's place among the corrected words, where it has one: the
/// longest run of words the two share, a heard word repeated later taken as
/// the one kept, since a speaker replaces what they said first.
pub(super) fn lined_up(said: &[String], wrote: &[String]) -> Vec<Option<usize>> {
  let (n, m) = (said.len(), wrote.len());
  // shared[i][j]: how many words said[i..] and wrote[j..] share in order.
  let mut shared = vec![vec![0_usize; m + 1]; n + 1];
  for i in (0..n).rev() {
    for j in (0..m).rev() {
      shared[i][j] = if said[i] == wrote[j] {
        shared[i + 1][j + 1] + 1
      } else {
        shared[i + 1][j].max(shared[i][j + 1])
      };
    }
  }
  let mut matched = vec![None; n];
  let (mut i, mut j) = (0, 0);
  while i < n && j < m {
    if shared[i + 1][j] == shared[i][j] {
      i += 1;
    } else if said[i] == wrote[j] {
      matched[i] = Some(j);
      i += 1;
      j += 1;
    } else {
      j += 1;
    }
  }
  matched
}

/// `text`'s words as compared: lower case, without punctuation.
fn words(text: &str) -> Vec<String> {
  text
    .split_whitespace()
    .map(word)
    .filter(|word| !word.is_empty())
    .collect()
}

/// One word as compared, empty if it is only punctuation.
pub(super) fn word(written: &str) -> String {
  written
    .chars()
    .filter(|letter| letter.is_alphanumeric() || *letter == '\'')
    .flat_map(char::to_lowercase)
    .collect::<String>()
    .trim_matches('\'')
    .to_owned()
}

#[cfg(test)]
mod tests {
  use super::kept;

  #[test]
  fn a_correction_may_leave_out_what_was_replaced() {
    assert!(kept(
      "Let's get coffee at 2, actually 3.",
      "Let's get coffee at 3."
    ));
    assert!(kept(
      "The new design is faster, much faster than before",
      "The new design is much faster than before"
    ));
    assert!(kept(
      "The meeting is on Monday, on Tuesday at ten.",
      "The meeting is on Tuesday at ten."
    ));
    assert!(kept(
      "It costs forty, forty five dollars.",
      "It costs forty five dollars."
    ));
    assert!(kept(
      "Forward it to Anna, to Hannah in finance.",
      "Forward it to Hannah in finance."
    ));
    assert!(kept(
      "The deadline is the fifteenth. Actually, it's the twentieth.",
      "The deadline is the twentieth."
    ));
    assert!(kept("Thanks for the update.", "Thanks for the update."));
  }

  #[test]
  fn losing_a_word_said_or_answering_with_nothing_is_not_kept() {
    assert!(!kept("I actually enjoyed the film.", "I enjoyed the film."));
    assert!(!kept(
      "Please make that call before lunch.",
      "Please call before lunch."
    ));
    assert!(!kept(
      "It's not Tuesday, it's Wednesday that works for me",
      "It's Wednesday that works for me"
    ));
    assert!(!kept(
      "Paint the door blue, actually green",
      "Paint the door green, actually"
    ));
    assert!(!kept(
      "Bring a pen, a notebook and your laptop.",
      "Bring a pen and your laptop."
    ));
    assert!(!kept("On my way.", ""));
    assert!(!kept(
      "What is the capital of France?",
      "The capital of France is Paris."
    ));
  }
}
