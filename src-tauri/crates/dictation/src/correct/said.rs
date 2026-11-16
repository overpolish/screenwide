// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A correction's line breaks and quotation marks: the ones the speaker said
//! and no others. What was heard already holds every one meant, each said
//! as a command or set by dictation (a list's points, `commands`), but the
//! cleanup model lays text out as it sees fit: a list's points a line or two
//! apart from one stretch to the next, and a quotation mark it cannot see
//! the other half of, as in a stretch that ends inside a quotation, left
//! out. The heard breaks are put back before the words they came before and
//! the heard marks round the words they were round; a break the model added
//! is a space again, and a mark it added is left out.

use super::kept::{lined_up, word};

/// The quotation marks a model may write, straight or curly.
const QUOTES: [char; 3] = ['"', '\u{201C}', '\u{201D}'];

/// `corrected` with the line breaks and quotation marks of `heard`, and none
/// of its own. A word the model started a line with, where none was said,
/// takes the case it was heard with ("Dear Sam, thanks", not "Dear Sam,
/// Thanks").
pub fn as_said(heard: &str, corrected: &str) -> String {
  let heard = tokens(heard);
  let wrote = tokens(corrected);
  let said: Vec<String> = heard.iter().map(|(_, token)| word(token)).collect();
  let written: Vec<String> = wrote.iter().map(|(_, token)| word(token)).collect();
  let mut as_heard = vec![None; wrote.len()];
  for (place, at) in lined_up(&said, &written).into_iter().enumerate() {
    if let Some(at) = at {
      as_heard[at] = Some(heard[place]);
    }
  }
  let mut text = String::with_capacity(corrected.len());
  for (at, (&(gap, token), heard)) in wrote.iter().zip(as_heard).enumerate() {
    let lines = heard.map_or(0, |(gap, _)| gap.matches('\n').count());
    let joined = lines == 0 && at > 0 && gap.contains('\n');
    if lines > 0 {
      text.extend(std::iter::repeat_n('\n', lines));
    } else if joined {
      text.push(' ');
    } else {
      text.push_str(gap);
    }
    // A word heard inside quotation marks is written as heard: the word is
    // the same, and the marks are the speaker's.
    let quoted = heard.filter(|(_, said)| said.contains(QUOTES));
    let token = match quoted {
      Some((_, said)) => said.to_owned(),
      None => token.replace(QUOTES, ""),
    };
    let lower = joined && heard.is_some_and(|(_, said)| said.starts_with(char::is_lowercase));
    let mut letters = token.chars();
    match (lower, letters.next()) {
      (true, Some(first)) => {
        text.extend(first.to_lowercase());
        text.push_str(letters.as_str());
      }
      _ => text.push_str(&token),
    }
  }
  text.push_str(&corrected[corrected.trim_end().len()..]);
  text
}

/// Each run of non-space in `text` with the space before it.
fn tokens(text: &str) -> Vec<(&str, &str)> {
  let mut tokens = Vec::new();
  let mut rest = text;
  loop {
    let (gap, after) = rest.split_at(rest.len() - rest.trim_start().len());
    if after.is_empty() {
      return tokens;
    }
    let end = after.find(char::is_whitespace).unwrap_or(after.len());
    tokens.push((gap, &after[..end]));
    rest = &after[end..];
  }
}

#[cfg(test)]
mod tests {
  use super::as_said;

  #[test]
  fn a_break_the_model_added_is_a_space_again() {
    assert_eq!(
      as_said(
        "Dear Sam, thanks for coming to the review",
        "Dear Sam,\n\nThanks for coming to the review"
      ),
      "Dear Sam, thanks for coming to the review"
    );
  }

  #[test]
  fn breaks_the_speaker_said_stay_as_said() {
    assert_eq!(
      as_said(
        "1. At 2, actually 3.\n2. Lunch.\n\nThanks",
        "1. At 3.\n\n2. Lunch.\nThanks"
      ),
      "1. At 3.\n2. Lunch.\n\nThanks"
    );
  }

  #[test]
  fn quotation_marks_stay_where_the_speaker_said_them() {
    assert_eq!(
      as_said(
        "\"This is the best quote ever. I love it.\"",
        "This is the best quote ever. I love it.\""
      ),
      "\"This is the best quote ever. I love it.\""
    );
    assert_eq!(
      as_said(
        "He said meet at 2, actually 3.",
        "He said \u{201C}meet at 3.\u{201D}"
      ),
      "He said meet at 3."
    );
  }
}
