// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation in a language no model streams accurately enough: the voice
//! activity model finds the pauses as the speech is fed, and each stretch a
//! pause ends is written on a thread of its own, so the app's feeds are
//! answered at once with whatever is written by then.

pub mod notes;
pub mod rewritten;
mod stretches;
mod writer;

use std::io::{BufRead, Lines};
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};

use screenwide_transcriber::{Job, Reply, Request};
use transcribe_cpp::{Feature, RunOptions, Session, VadResult, VadSession};

use self::stretches::{Pauses, Stretch};
use self::writer::{dictated, lock, Writing, Written};
use crate::stream::Dictation;

/// What a dictation at pauses was begun with.
pub struct AtPauses {
  pub model: PathBuf,
  pub vad: PathBuf,
  pub language: Option<String>,
  pub words: Vec<String>,
}

/// Runs the dictation `begin` asked for, as [`Dictation::run`] runs one that
/// streams. False once the app has stopped listening.
pub fn run<B: BufRead>(
  dictation: &mut Dictation,
  begin: u64,
  at_pauses: AtPauses,
  lines: &mut Lines<B>,
  send: &mut dyn FnMut(&Reply) -> bool,
) -> bool {
  let (session, vad) = match dictation.sessions(&at_pauses.model, &at_pauses.vad) {
    Ok(sessions) => sessions,
    Err(message) => return send(&Reply::Failed { id: begin, message }),
  };
  vad.stream_reset();
  let writing = Writing {
    takes_context: session.model().supports(Feature::ContextPrompt),
    run: RunOptions {
      language: at_pauses.language,
      vocabulary: at_pauses.words,
      ..RunOptions::default()
    },
  };
  if !send(&Reply::Dictated {
    id: begin,
    text: dictated(&Written::default(), 0, false),
  }) {
    return false;
  }
  listen(session, vad, &writing, lines, send)
}

fn listen<B: BufRead>(
  session: &mut Session,
  vad: &mut VadSession,
  writing: &Writing,
  lines: &mut Lines<B>,
  send: &mut dyn FnMut(&Reply) -> bool,
) -> bool {
  let audio = Mutex::new(Vec::new());
  let written = Mutex::new(Written::default());
  std::thread::scope(|scope| {
    let (stretches, to_write) = mpsc::channel::<Stretch>();
    let (shared_audio, shared_written) = (&audio, &written);
    let writer = scope.spawn(move || {
      writer::write(session, &to_write, shared_audio, shared_written, writing);
    });
    let mut pauses = Pauses::default();
    let mut stretches = Some(stretches);
    let mut pcm = Vec::new();
    for line in lines {
      let Ok(line) = line else { return false };
      if line.trim().is_empty() {
        continue;
      }
      let Ok(Request { id, job }) = serde_json::from_str::<Request>(&line) else {
        return false;
      };
      let reply = match job {
        Job::FeedDictation { samples } => {
          pcm.clear();
          pcm.extend(samples.iter().map(|&sample| f32::from(sample) / 32_768.0));
          lock(&audio).extend_from_slice(&pcm);
          let heard = vad.stream_feed(&pcm);
          let found = heard.map(|heard| found(&mut pauses, &heard));
          match (found, &stretches) {
            (Ok(found), Some(stretches)) => {
              for stretch in found {
                let _ = stretches.send(stretch);
              }
              answer(id, &written, lock(&audio).len(), false)
            }
            (Err(error), _) => failed(id, format!("Could not hear the dictation: {error}")),
            (Ok(_), None) => failed(id, "The dictation has finished".to_owned()),
          }
        }
        Job::FinishDictation => {
          let mut last: Vec<Stretch> = vad
            .stream_flush()
            .map(|heard| found(&mut pauses, &heard))
            .unwrap_or_default();
          last.extend(pauses.end());
          if let Some(stretches) = stretches.take() {
            for stretch in last {
              let _ = stretches.send(stretch);
            }
          }
          let _ = writer.join();
          let reply = answer(id, &written, lock(&audio).len(), true);
          return send(&reply);
        }
        _ => failed(id, "A dictation is under way".to_owned()),
      };
      if !send(&reply) {
        return false;
      }
    }
    false
  })
}

/// The stretches the frames in `heard` end.
fn found(pauses: &mut Pauses, heard: &VadResult) -> Vec<Stretch> {
  let first = usize::try_from(heard.first_frame).unwrap_or(0);
  heard
    .probs
    .iter()
    .enumerate()
    .filter_map(|(at, &speech)| pauses.hear(first + at, speech))
    .collect()
}

fn answer(id: u64, written: &Mutex<Written>, heard: usize, finished: bool) -> Reply {
  let written = lock(written);
  match &written.failed {
    Some(message) => failed(id, message.clone()),
    None => Reply::Dictated {
      id,
      text: dictated(&written, heard, finished),
    },
  }
}

fn failed(id: u64, message: String) -> Reply {
  Reply::Failed { id, message }
}
