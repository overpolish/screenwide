// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The text a run of steps comes to, words spaced and commands carried out.

use screenwide_speech::runner::dictation::DictatedWord;

use super::{Command, Step, WordChange};

/// Ends a sentence, as "scratch that" counts them.
const SENTENCE_ENDS: [char; 3] = ['.', '?', '!'];
/// What a mark said by name takes the place of at the end of the text.
const MARKS: [char; 6] = ['.', ',', '?', '!', ':', ';'];

pub(super) struct Rendered {
  pub text: String,
  pub changes: Vec<WordChange>,
}

/// The text of `steps` over `words`, taking only those within the first
/// `upto` words.
pub(super) fn render(words: &[DictatedWord], steps: &[Step], upto: usize) -> Rendered {
  let mut text = String::new();
  let mut changes = Vec::new();
  for step in steps {
    match step {
      Step::Word(at) if *at < upto => {
        if !text.is_empty() && !text.ends_with('\n') {
          text.push(' ');
        }
        text.push_str(&words[*at].text);
      }
      Step::Command {
        command, to, word, ..
      } if *to <= upto => match command {
        Command::ScratchThat => scratch_last_sentence(&mut text),
        Command::Break(lines) => {
          text.truncate(text.trim_end_matches(' ').len());
          if !text.is_empty() {
            text.push_str(lines);
          }
        }
        Command::Mark(mark) => {
          text.truncate(
            text
              .trim_end_matches(|c: char| c == ' ' || MARKS.contains(&c))
              .len(),
          );
          if !text.is_empty() && !text.ends_with('\n') {
            text.push(*mark);
          }
        }
        Command::AddWord => changes.extend(word.clone().map(WordChange::Add)),
        Command::RemoveWord => changes.extend(word.clone().map(WordChange::Remove)),
      },
      _ => {}
    }
  }
  Rendered { text, changes }
}

/// Takes the last sentence away, back to where the one before it ended or
/// the line began.
fn scratch_last_sentence(text: &mut String) {
  let body = text.trim_end_matches(' ');
  let unfinished = body.trim_end_matches(SENTENCE_ENDS);
  let start = unfinished
    .rfind(|c: char| SENTENCE_ENDS.contains(&c) || c == '\n')
    .map_or(0, |at| at + 1);
  text.truncate(start);
}
