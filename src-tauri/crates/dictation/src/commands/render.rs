// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The text a run of steps comes to, words spaced and commands carried out,
//! the numbers starting a list's points written as digits, each point on a
//! line of its own (`points`), "hashtag" said before a word written as the
//! mark (`hashtags`), and a pause before a word that corrects what was just
//! said ("at 3 … actually 2") marked with the comma the speech model leaves
//! out.

mod hashtags;
mod points;

use std::fmt::Write;

use screenwide_speech::runner::dictation::DictatedWord;

use super::{gap, plain, Command, Phrases, Step, WordChange, PAUSE_MS};

/// Ends a sentence, as "scratch that" counts them, full width as Chinese
/// and Japanese write them too.
const SENTENCE_ENDS: [char; 6] = ['.', '?', '!', '。', '？', '！'];
/// What a mark said by name takes the place of at the end of the text.
const MARKS: [char; 13] = [
  '.', ',', '?', '!', ':', ';', '。', '，', '、', '？', '！', '：', '；',
];

pub(super) struct Rendered {
  pub text: String,
  pub changes: Vec<WordChange>,
}

/// The text of `steps` over `words`, taking only those within the first
/// `upto` words, written as `phrases` writes them.
pub(super) fn render(
  words: &[DictatedWord],
  steps: &[Step],
  (upto, phrases): (usize, &Phrases),
) -> Rendered {
  let mut text = String::new();
  let mut changes = Vec::new();
  let mut last_word = None;
  let points = points::points(words, steps, (upto, phrases));
  let hashtags = hashtags::hashtags(words, steps, (upto, phrases));
  let mut after_point = false;
  let mut after_hashtag = false;
  for step in steps {
    match step {
      Step::Word(at) if *at < upto => {
        if hashtags.contains(at) {
          after_hashtag = true;
          continue;
        }
        let unpunctuated = text.ends_with(char::is_alphanumeric);
        let set_apart = unpunctuated
          && last_word.is_some_and(|before| corrects_after_pause(words, (before, *at), phrases));
        if set_apart {
          text.push(',');
        }
        let point = points
          .iter()
          .find(|(place, _)| place == at)
          .map(|&(_, number)| number);
        let line_start = text.is_empty() || text.ends_with('\n');
        if point.is_some() && !line_start {
          // Each point of a list starts a line of its own.
          text.push('\n');
        } else if phrases.spaced && !line_start {
          text.push(' ');
        }
        if std::mem::take(&mut after_hashtag) {
          text.push('#');
        }
        let word = &words[*at].text;
        let follows_point = std::mem::take(&mut after_point);
        if let Some(number) = point {
          let _ = write!(text, "{number}.");
          after_point = true;
        } else if set_apart && plain(word) != "i" {
          // The speech model starts what follows a pause as a sentence;
          // after the comma it goes on the sentence, so it loses its
          // capital. "I" keeps its own.
          push_recased(&mut text, word, char::to_lowercase);
        } else if follows_point {
          // A point's number set apart with a comma leaves its point
          // starting in lower case, but the full stop starts a sentence.
          push_recased(&mut text, word, char::to_uppercase);
        } else {
          text.push_str(word);
        }
        last_word = Some(*at);
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

/// Pushes `word` with its first letter recased.
fn push_recased<I: Iterator<Item = char>>(text: &mut String, word: &str, recase: fn(char) -> I) {
  let mut letters = word.chars();
  text.extend(letters.next().into_iter().flat_map(recase));
  text.push_str(letters.as_str());
}

/// Whether the speaker paused between the words at `before` and `at`, and
/// what they say from `at` corrects what they said before it.
fn corrects_after_pause(
  words: &[DictatedWord],
  (before, at): (usize, usize),
  phrases: &Phrases,
) -> bool {
  gap(&words[before], &words[at]) >= PAUSE_MS
    && phrases.corrections.iter().any(|said| {
      let mut heard = words[at..].iter().map(|word| plain(&word.text));
      said
        .split(' ')
        .all(|part| heard.next().is_some_and(|word| word == part))
    })
}

/// Takes the last sentence away, back to where the one before it ended or
/// the line began.
fn scratch_last_sentence(text: &mut String) {
  let body = text.trim_end_matches(' ');
  let unfinished = body.trim_end_matches(SENTENCE_ENDS);
  let start = unfinished
    .char_indices()
    .rfind(|&(_, c)| SENTENCE_ENDS.contains(&c) || c == '\n')
    .map_or(0, |(at, c)| at + c.len_utf8());
  text.truncate(start);
}
