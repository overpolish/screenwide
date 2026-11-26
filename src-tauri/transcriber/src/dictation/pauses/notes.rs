// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Notes from a call: the microphone and the computer's audio, each its own
//! track, written a stretch at a time once a pause ends it (`track`,
//! `writing`). The audio stays in memory for as long as the notes are taken,
//! never on disk, so the voices in the computer's audio can be told apart
//! over the whole call once it ends (`speakers`): the speaker model reads a
//! whole recording at once. All that is kept of it is a few seconds of each
//! voice (`alone`), until whoever took the notes names its speaker.

mod alone;
mod bleed;
mod clip_window;
mod speakers;
mod track;
mod voices;
mod writing;

use std::io::{BufRead, Lines};
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};

use screenwide_transcriber::{Job, NoteSource, Reply, Request};
use transcribe_cpp::RunOptions;

use self::track::{Audio, Track};
use self::writing::Heard;
use super::stretches::Stretch;
use super::writer::lock;
use crate::stream::Dictation;

/// What notes were begun with.
pub struct Notes {
  pub model: PathBuf,
  pub vad: PathBuf,
  pub speakers: PathBuf,
  pub voices: PathBuf,
  pub language: Option<String>,
  pub words: Vec<String>,
}

/// Takes the notes `begin` asked for, as [`Dictation::run`] runs a
/// dictation. False once the app has stopped listening.
pub fn run<B: BufRead>(
  dictation: &mut Dictation,
  begin: u64,
  notes: Notes,
  lines: &mut Lines<B>,
  send: &mut dyn FnMut(&Reply) -> bool,
) -> bool {
  let loaded = Track::new(&notes.vad).and_then(|microphone| {
    let system = Track::new(&notes.vad)?;
    let speakers = speakers::Speakers::load(&notes.speakers, &notes.voices)?;
    Ok(([microphone, system], speakers))
  });
  let ((mut tracks, mut speakers), session) =
    match loaded.and_then(|loaded| Ok((loaded, dictation.sessions(&notes.model, &notes.vad)?.0))) {
      Ok(begun) => begun,
      Err(message) => return send(&Reply::Failed { id: begin, message }),
    };
  if !send(&Reply::Noted {
    id: begin,
    lines: Vec::new(),
  }) {
    return false;
  }
  let run = RunOptions {
    language: notes.language,
    vocabulary: notes.words,
    ..RunOptions::default()
  };
  let audio: [Audio; 2] = Default::default();
  let heard = Mutex::new(Heard::default());
  std::thread::scope(|scope| {
    let (stretches, to_write) = mpsc::channel::<(NoteSource, Stretch)>();
    let (shared_audio, shared_heard, run) = (&audio, &heard, &run);
    let writer =
      scope.spawn(move || writing::write(session, run, &to_write, shared_audio, shared_heard));
    let mut stretches = Some(stretches);
    for line in lines {
      let Ok(line) = line else { return false };
      if line.trim().is_empty() {
        continue;
      }
      let Ok(Request { id, job }) = serde_json::from_str::<Request>(&line) else {
        return false;
      };
      let reply = match job {
        Job::FeedNotes {
          source,
          at_ms,
          samples,
        } => {
          let at = index(source);
          match tracks[at].feed(&audio[at], at_ms, &samples) {
            Ok(found) => {
              if let Some(stretches) = &stretches {
                for stretch in found {
                  let _ = stretches.send((source, stretch));
                }
              }
              noted(id, &heard)
            }
            Err(message) => failed(id, message),
          }
        }
        Job::FinishNotes => {
          if let Some(stretches) = stretches.take() {
            for source in [NoteSource::Microphone, NoteSource::System] {
              for stretch in tracks[index(source)].finish() {
                let _ = stretches.send((source, stretch));
              }
            }
          }
          let _ = writer.join();
          let mut written = lock(&heard);
          let system = lock(&audio[index(NoteSource::System)]);
          let reply = match written.failed.take() {
            Some(message) => failed(id, message),
            None => match speakers.turns(&system) {
              Ok(turns) => Reply::NotesFinished {
                id,
                lines: std::mem::take(&mut written.lines),
                voices: alone::clips(&system, &turns),
                turns,
              },
              Err(message) => failed(id, message),
            },
          };
          return send(&reply);
        }
        _ => failed(id, "Notes are being taken".to_owned()),
      };
      if !send(&reply) {
        return false;
      }
    }
    false
  })
}

fn index(source: NoteSource) -> usize {
  match source {
    NoteSource::Microphone => 0,
    NoteSource::System => 1,
  }
}

/// The lines written since the last answer, or why writing stopped.
fn noted(id: u64, heard: &Mutex<Heard>) -> Reply {
  let mut heard = lock(heard);
  match &heard.failed {
    Some(message) => failed(id, message.clone()),
    None => Reply::Noted {
      id,
      lines: std::mem::take(&mut heard.lines),
    },
  }
}

fn failed(id: u64, message: String) -> Reply {
  Reply::Failed { id, message }
}
