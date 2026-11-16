// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The answers scored, and written out as `results.md`.

use std::fmt::Write as _;

/// A dictation: what it should become, and what was heard.
pub struct Case {
  pub id: String,
  pub category: String,
  pub expect: String,
  pub heard: String,
}

/// A model's answer for one dictation.
pub struct Answer {
  pub text: String,
  /// Whether it passes the lost-word check.
  pub kept: bool,
  pub milliseconds: u128,
}

/// A model's answers, in the dictations' order.
pub struct Scored {
  pub name: String,
  pub answers: Vec<Answer>,
}

/// `text`'s words as scored: lower case, without punctuation.
fn words(text: &str) -> Vec<String> {
  text
    .split_whitespace()
    .map(|word| {
      word
        .chars()
        .filter(|letter| letter.is_alphanumeric() || *letter == '\'')
        .flat_map(char::to_lowercase)
        .collect::<String>()
    })
    .filter(|word| !word.is_empty())
    .collect()
}

/// How many words both heard and kept by `expect` are missing from `text`,
/// so a word the recognizer got wrong is not counted against the model.
fn lost(heard: &str, expect: &str, text: &str) -> usize {
  let mut expected = words(expect);
  let mut left = words(text);
  let take = |from: &mut Vec<String>, word: &String| {
    from
      .iter()
      .position(|other| other == word)
      .map(|at| from.swap_remove(at))
      .is_some()
  };
  words(heard)
    .iter()
    .filter(|word| take(&mut expected, word) && !take(&mut left, word))
    .count()
}

#[derive(Default)]
struct Tally {
  applied: usize,
  to_apply: usize,
  unchanged: usize,
  exactly: usize,
  clean: usize,
  losing: usize,
  words_lost: usize,
}

fn tally(cases: &[Case], texts: impl Iterator<Item = String>) -> Tally {
  let mut tally = Tally::default();
  for (case, text) in cases.iter().zip(texts) {
    if words(&case.expect) == words(&case.heard) {
      tally.clean += 1;
      tally.unchanged += usize::from(words(&text) == words(&case.heard));
      tally.exactly += usize::from(text.trim() == case.heard.trim());
    } else {
      tally.to_apply += 1;
      tally.applied += usize::from(words(&text) == words(&case.expect));
    }
    let lost = lost(&case.heard, &case.expect, &text);
    tally.losing += usize::from(lost > 0);
    tally.words_lost += lost;
  }
  tally
}

/// The scores of every model, then each dictation's answers.
pub fn report(cases: &[Case], scored: &[Scored]) -> String {
  let mut out = String::from(
    "<!-- SPDX-FileCopyrightText: 2026 overpolish -->\n<!-- SPDX-License-Identifier: GPL-3.0-or-later -->\n\n# Cleanup models on the English dictations\n\nWritten by `cargo run --example evaluate_cleanup`. Words are compared ignoring case and punctuation. Applied counts the dictations needing a correction that came out as expected. Unchanged counts the dictations needing none whose words came back as heard, and exactly, character for character. Words lost counts the words both heard and expected that the answer left out. Checked is the answer after the lost-word check, which puts back the text as heard when the answer fails it.\n\n| Model | Applied | Unchanged | Exactly | Losing words | Words lost | Checked: applied | Checked: unchanged | Checked: losing words | Median ms | Slowest ms |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n",
  );
  for model in scored {
    let raw = tally(
      cases,
      model.answers.iter().map(|answer| answer.text.clone()),
    );
    let checked = tally(
      cases,
      model.answers.iter().zip(cases).map(|(answer, case)| {
        if answer.kept {
          answer.text.clone()
        } else {
          case.heard.clone()
        }
      }),
    );
    let mut times: Vec<u128> = model
      .answers
      .iter()
      .map(|answer| answer.milliseconds)
      .collect();
    times.sort_unstable();
    let _ = writeln!(
      out,
      "| {} | {}/{} | {}/{} | {}/{} | {} | {} | {}/{} | {}/{} | {} | {} | {} |",
      model.name,
      raw.applied,
      raw.to_apply,
      raw.unchanged,
      raw.clean,
      raw.exactly,
      raw.clean,
      raw.losing,
      raw.words_lost,
      checked.applied,
      checked.to_apply,
      checked.unchanged,
      checked.clean,
      checked.losing,
      times[times.len() / 2],
      times.last().copied().unwrap_or_default(),
    );
  }
  out.push_str(
    "\n## Each dictation\n\nAn answer the lost-word check turns down is struck through.\n",
  );
  for (index, case) in cases.iter().enumerate() {
    let _ = write!(
      out,
      "\n### {} ({})\n\n- Expected: {}\n",
      case.id,
      case.category,
      quoted(&case.expect)
    );
    for model in scored {
      let answer = &model.answers[index];
      let text = quoted(&answer.text);
      let shown = if answer.kept {
        text
      } else {
        format!("~~{text}~~")
      };
      let _ = writeln!(out, "- {}: {shown}", model.name);
    }
  }
  out
}

fn quoted(text: &str) -> String {
  format!("`{}`", text.replace('\n', "⏎"))
}
