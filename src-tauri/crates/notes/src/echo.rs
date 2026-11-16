// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The call heard twice. Played through the computer's speakers rather than
//! headphones, the other people reach the microphone too, and their words
//! would be written again as the note taker's. A microphone line said while
//! the computer's audio was speaking, mostly in its words, is that echo.

use screenwide_speech::runner::notes::{NoteLine, NoteSource};

/// How far apart in time a line and the speech it echoes may be, in
/// milliseconds: each is cut at its own pauses.
const SLACK_MS: u64 = 1_000;
/// The share of a microphone line's words that, found in what the computer
/// played meanwhile, make it an echo.
const ECHOED: f32 = 0.6;

/// `lines` without the microphone's lines that echo the computer's audio.
pub fn without_echo(lines: &[NoteLine]) -> Vec<&NoteLine> {
  let played: Vec<&NoteLine> = lines
    .iter()
    .filter(|line| line.source == NoteSource::System)
    .collect();
  lines
    .iter()
    .filter(|line| line.source == NoteSource::System || !echoes(line, &played))
    .collect()
}

/// Whether the microphone's `line` is mostly the words of what was played
/// while it was said.
fn echoes(line: &NoteLine, played: &[&NoteLine]) -> bool {
  let said = words(&line.text);
  if said.is_empty() {
    return false;
  }
  let meanwhile: Vec<String> = played
    .iter()
    .filter(|other| {
      other.start_ms <= line.end_ms + SLACK_MS && line.start_ms <= other.end_ms + SLACK_MS
    })
    .flat_map(|other| words(&other.text))
    .collect();
  let found = said.iter().filter(|word| meanwhile.contains(word)).count();
  found as f32 >= ECHOED * said.len() as f32
}

fn words(text: &str) -> Vec<String> {
  text
    .split_whitespace()
    .map(|word| {
      word
        .chars()
        .filter(|letter| letter.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>()
    })
    .filter(|word| !word.is_empty())
    .collect()
}

#[cfg(test)]
mod tests {
  use screenwide_speech::runner::notes::{NoteLine, NoteSource};

  use super::without_echo;

  fn line(source: NoteSource, (start_ms, end_ms): (u64, u64), text: &str) -> NoteLine {
    NoteLine {
      source,
      start_ms,
      end_ms,
      text: text.to_owned(),
      words: Vec::new(),
    }
  }

  #[test]
  fn what_the_speakers_played_is_not_written_again_as_the_note_takers() {
    let lines = [
      line(
        NoteSource::System,
        (0, 3_000),
        "We should ship the beta next week.",
      ),
      line(
        NoteSource::Microphone,
        (100, 3_100),
        "we should ship the beta next",
      ),
      line(NoteSource::Microphone, (4_000, 6_000), "That works for me."),
    ];
    let kept: Vec<&str> = without_echo(&lines)
      .iter()
      .map(|line| line.text.as_str())
      .collect();
    assert_eq!(
      kept,
      ["We should ship the beta next week.", "That works for me."]
    );
  }

  #[test]
  fn the_same_words_said_at_another_time_are_kept() {
    let lines = [
      line(NoteSource::System, (0, 3_000), "We should ship the beta."),
      line(
        NoteSource::Microphone,
        (10_000, 12_000),
        "We should ship the beta.",
      ),
    ];
    assert_eq!(without_echo(&lines).len(), 2);
  }
}
