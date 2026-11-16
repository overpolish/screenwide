// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A correction's line breaks: the ones the speaker said and no others. The
//! cleanup model lays text out as it sees fit, a list's points a line or
//! two apart from one stretch to the next, where what was heard already
//! holds every break meant: each said as "new line" or "new paragraph", and
//! each before a list's point (`commands`). The heard breaks are put back
//! before the words they came before, and a break the model added is a
//! space again.

use super::kept::{lined_up, word};

/// `corrected` with the line breaks of `heard`, and none of its own. A word
/// the model started a line with, where none was said, takes the case it
/// was heard with ("Dear Sam, thanks", not "Dear Sam, Thanks").
pub fn said_lines(heard: &str, corrected: &str) -> String {
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
    if lines > 0 {
      text.extend(std::iter::repeat_n('\n', lines));
    } else if at > 0 && gap.contains('\n') {
      text.push(' ');
      let lower = heard.is_some_and(|(_, said)| said.starts_with(char::is_lowercase));
      let mut letters = token.chars();
      if let (true, Some(first)) = (lower, letters.next()) {
        text.extend(first.to_lowercase());
        text.push_str(letters.as_str());
        continue;
      }
    } else {
      text.push_str(gap);
    }
    text.push_str(token);
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
  use super::said_lines;

  #[test]
  fn a_break_the_model_added_is_a_space_again() {
    assert_eq!(
      said_lines(
        "Dear Sam, thanks for coming to the review",
        "Dear Sam,\n\nThanks for coming to the review"
      ),
      "Dear Sam, thanks for coming to the review"
    );
  }

  #[test]
  fn breaks_the_speaker_said_stay_as_said() {
    assert_eq!(
      said_lines(
        "1. At 2, actually 3.\n2. Lunch.\n\nThanks",
        "1. At 3.\n\n2. Lunch.\nThanks"
      ),
      "1. At 3.\n2. Lunch.\n\nThanks"
    );
  }
}
