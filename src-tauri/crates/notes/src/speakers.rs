// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Who said what: the microphone is the note taker. In the computer's audio,
//! a stretch runs on across a change of voice when the next speaker answers
//! quickly, so once the speaker model has told the voices apart each word
//! goes to the voice heard most while it was said, and a line is shared out
//! where the voice changes. Voices are numbered in the order they first speak
//! in the notes. Until the call ends and the model has run, they are the
//! others, together.

use screenwide_speech::runner::notes::{NoteLine, NoteSource, SpeakerTurn};

/// Who said something.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Who {
  /// The person taking the notes.
  Me,
  /// Everyone else, before their voices are told apart.
  Others,
  /// One of the others, numbered from 1.
  Speaker(u32),
}

/// What was said, in order, each run of it with who said it, and the
/// speaker model's voice for each speaker number from 1.
#[derive(Debug, Default)]
pub struct Attributed {
  pub said: Vec<(Who, String)>,
  pub voices: Vec<u32>,
}

/// Who said what in `lines`, the voices in the computer's audio told apart
/// by `turns` once the speaker model has run.
pub fn attributed(lines: &[&NoteLine], turns: Option<&[SpeakerTurn]>) -> Attributed {
  let mut numbered: Vec<u32> = Vec::new();
  let mut said = Vec::new();
  for line in lines {
    let turns = match (line.source, turns) {
      (NoteSource::Microphone, _) => {
        said.push((Who::Me, line.text.clone()));
        continue;
      }
      (NoteSource::System, None) => {
        said.push((Who::Others, line.text.clone()));
        continue;
      }
      (NoteSource::System, Some(turns)) => turns,
    };
    let mut number = |span| match voice(span, turns) {
      Some(voice) => {
        let at = numbered
          .iter()
          .position(|&known| known == voice)
          .unwrap_or_else(|| {
            numbered.push(voice);
            numbered.len() - 1
          });
        Who::Speaker(u32::try_from(at).unwrap_or(0) + 1)
      }
      None => Who::Others,
    };
    if line.words.is_empty() {
      said.push((number((line.start_ms, line.end_ms)), line.text.clone()));
      continue;
    }
    let mut runs: Vec<(Who, String)> = Vec::new();
    for word in &line.words {
      let who = number((word.start_ms, word.end_ms));
      match runs.last_mut() {
        Some((speaking, text)) if *speaking == who => {
          text.push(' ');
          text.push_str(&word.text);
        }
        _ => runs.push((who, word.text.clone())),
      }
    }
    said.extend(runs);
  }
  Attributed {
    said,
    voices: numbered,
  }
}

/// The voice heard most during `(start_ms, end_ms)`, or the nearest where
/// none was heard during it.
fn voice((start_ms, end_ms): (u64, u64), turns: &[SpeakerTurn]) -> Option<u32> {
  let mut heard: Vec<(u32, u64)> = Vec::new();
  for turn in turns {
    let overlap = turn
      .end_ms
      .min(end_ms)
      .saturating_sub(turn.start_ms.max(start_ms));
    if overlap == 0 {
      continue;
    }
    match heard.iter_mut().find(|(voice, _)| *voice == turn.speaker) {
      Some((_, total)) => *total += overlap,
      None => heard.push((turn.speaker, overlap)),
    }
  }
  let most = heard
    .iter()
    .max_by_key(|(_, total)| *total)
    .map(|(voice, _)| *voice);
  most.or_else(|| {
    turns
      .iter()
      .min_by_key(|turn| {
        turn
          .start_ms
          .saturating_sub(end_ms)
          .max(start_ms.saturating_sub(turn.end_ms))
      })
      .map(|turn| turn.speaker)
  })
}

#[cfg(test)]
mod tests {
  use screenwide_speech::runner::notes::{NoteLine, NoteSource, NoteWord, SpeakerTurn};

  use super::{attributed, Who};

  fn turn(start_ms: u64, end_ms: u64, speaker: u32) -> SpeakerTurn {
    SpeakerTurn {
      start_ms,
      end_ms,
      speaker,
    }
  }

  fn word(text: &str, start_ms: u64) -> NoteWord {
    NoteWord {
      text: text.to_owned(),
      start_ms,
      end_ms: start_ms + 300,
    }
  }

  #[test]
  fn a_line_two_voices_spoke_in_is_shared_out_between_them() {
    let others = NoteLine {
      source: NoteSource::System,
      start_ms: 0,
      end_ms: 2_000,
      text: "Ready next week. Docs aren't.".to_owned(),
      words: vec![
        word("Ready", 0),
        word("next", 300),
        word("week.", 600),
        word("Docs", 1_200),
        word("aren't.", 1_500),
      ],
    };
    let me = NoteLine {
      source: NoteSource::Microphone,
      start_ms: 2_500,
      end_ms: 3_500,
      text: "Who takes them?".to_owned(),
      words: Vec::new(),
    };
    let lines = [&others, &me];
    let turns = [turn(0, 1_000, 4), turn(1_100, 2_000, 2)];
    let told = attributed(&lines, Some(&turns));
    assert_eq!(
      told.said,
      [
        (Who::Speaker(1), "Ready next week.".to_owned()),
        (Who::Speaker(2), "Docs aren't.".to_owned()),
        (Who::Me, "Who takes them?".to_owned()),
      ]
    );
    assert_eq!(told.voices, [4, 2]);
    assert_eq!(attributed(&lines, None).said[0].0, Who::Others);
  }
}
