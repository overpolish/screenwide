// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each stretch of a call written out once a pause ends it, on a thread of
//! its own, so the app's feeds are answered at once. A stretch is written
//! once and kept: notes are read afterwards, not watched being written.

use std::sync::mpsc::Receiver;
use std::sync::Mutex;

use screenwide_transcriber::{NoteLine, NoteSource, NoteWord, SAMPLE_RATE};
use transcribe_cpp::{RunOptions, Session, TimestampKind};

use super::super::stretches::Stretch;
use super::super::writer::lock;
use super::bleed::without_bleed;
use super::index;
use super::track::Audio;

/// Audio after a stretch's speech written with it, as the voice fades past
/// where the voice activity model put its end: 300 ms.
const TAIL: usize = 4_800;

/// The lines written since the app last asked, or why writing stopped.
#[derive(Default)]
pub struct Heard {
  pub lines: Vec<NoteLine>,
  pub failed: Option<String>,
}

/// Writes each stretch `stretches` brings, from its track in `audio`, into
/// `heard`, until `stretches` closes or writing fails.
pub fn write(
  session: &mut Session,
  run: &RunOptions,
  stretches: &Receiver<(NoteSource, Stretch)>,
  audio: &[Audio; 2],
  heard: &Mutex<Heard>,
) {
  for (source, stretch) in stretches {
    let mut pcm: Vec<f32> = {
      let audio = lock(&audio[index(source)]);
      let end = (stretch.end + TAIL).min(audio.len());
      audio[stretch.start.min(end)..end]
        .iter()
        .map(|&sample| f32::from(sample) / 32_768.0)
        .collect()
    };
    if source == NoteSource::Microphone {
      let system = lock(&audio[index(NoteSource::System)]);
      if !without_bleed(&mut pcm, &system, stretch.start) {
        continue;
      }
    }
    // The computer's audio is written word by word, so a stretch two voices
    // spoke in can be shared out between them once they are told apart.
    let mut run = run.clone();
    if source == NoteSource::System {
      run.timestamps = TimestampKind::Word;
    }
    match session.run(&pcm, &run) {
      Ok(transcript) => {
        let text = transcript.text.trim().to_owned();
        let offset = ms(stretch.start);
        let words = transcript
          .words
          .iter()
          .map(|word| NoteWord {
            text: word.text.trim().to_owned(),
            start_ms: offset + u64::try_from(word.t0_ms).unwrap_or(0),
            end_ms: offset + u64::try_from(word.t1_ms).unwrap_or(0),
          })
          .filter(|word| !word.text.is_empty())
          .collect();
        if !text.is_empty() {
          lock(heard).lines.push(NoteLine {
            source,
            start_ms: offset,
            end_ms: ms(stretch.end),
            text,
            words,
          });
        }
      }
      Err(error) => {
        lock(heard).failed = Some(format!("Could not write the notes: {error}"));
        return;
      }
    }
  }
}

fn ms(samples: usize) -> u64 {
  samples as u64 * 1_000 / u64::from(SAMPLE_RATE)
}
