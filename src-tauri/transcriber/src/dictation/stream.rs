// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation: speech written out as it is spoken, by a streaming model run
//! through transcribe.cpp. A dictation takes the helper's input over from
//! [`Job::BeginDictation`] to [`Job::FinishDictation`], answering each request
//! itself, as its stream borrows the model's session for that long.

use std::io::{BufRead, Lines};
use std::path::{Path, PathBuf};

use screenwide_transcriber::{DictatedText, Job, Reply, Request};
use transcribe_cpp::{
  CommitPolicy, Model, ParakeetBufferedStreamOptions, RunOptions, Session, StreamExtension,
  StreamOptions, StreamText,
};

/// Parakeet Unified's buffered stream re-reads this much of what came before
/// each chunk, the only left context it was trained with.
const LEFT_MS: i32 = 5_600;

/// The model last loaded and a session on it, kept for the dictations after:
/// loading takes far longer than the first words.
#[derive(Default)]
pub struct Dictation {
  loaded: Option<(PathBuf, Session)>,
}

impl Dictation {
  /// Runs the dictation `begin` asked for, reading its requests from `lines`
  /// and sending every reply, the one to `begin` included, through `send`.
  /// False once the app has stopped listening.
  pub fn run<B: BufRead>(
    &mut self,
    begin: u64,
    model: &Path,
    lines: &mut Lines<B>,
    send: &mut dyn FnMut(&Reply) -> bool,
  ) -> bool {
    let session = match self.session(model) {
      Ok(session) => session,
      Err(message) => return send(&Reply::Failed { id: begin, message }),
    };
    let chunk_ms = chunk_ms(&session.model());
    let options = StreamOptions {
      commit_policy: CommitPolicy::Auto,
      family: Some(StreamExtension::ParakeetBuffered(
        ParakeetBufferedStreamOptions {
          chunk_ms: Some(chunk_ms),
          left_ms: Some(LEFT_MS),
          right_ms: Some(chunk_ms),
        },
      )),
      ..StreamOptions::default()
    };
    let mut stream = match session.stream(&RunOptions::default(), &options) {
      Ok(stream) => stream,
      Err(error) => {
        return send(&Reply::Failed {
          id: begin,
          message: format!("Could not start dictating: {error}"),
        })
      }
    };
    if !send(&Reply::Dictated {
      id: begin,
      text: DictatedText::default(),
    }) {
      return false;
    }
    let mut pcm = Vec::new();
    for line in lines {
      let Ok(line) = line else { return false };
      if line.trim().is_empty() {
        continue;
      }
      let Ok(Request { id, job }) = serde_json::from_str::<Request>(&line) else {
        return false;
      };
      let (reply, finished) = match job {
        Job::FeedDictation { samples } => {
          pcm.clear();
          pcm.extend(samples.iter().map(|&sample| f32::from(sample) / 32_768.0));
          let reply = match stream.feed(&pcm) {
            Ok(_) => Reply::Dictated {
              id,
              text: text(stream.text()),
            },
            Err(error) => Reply::Failed {
              id,
              message: format!("Could not hear the dictation: {error}"),
            },
          };
          (reply, false)
        }
        Job::FinishDictation => {
          let reply = match stream.finalize() {
            Ok(_) => Reply::Dictated {
              id,
              text: text(stream.text()),
            },
            Err(error) => Reply::Failed {
              id,
              message: format!("Could not finish the dictation: {error}"),
            },
          };
          (reply, true)
        }
        _ => (
          Reply::Failed {
            id,
            message: "A dictation is under way".to_owned(),
          },
          false,
        ),
      };
      if !send(&reply) {
        return false;
      }
      if finished {
        return true;
      }
    }
    false
  }

  fn session(&mut self, path: &Path) -> Result<&mut Session, String> {
    if !matches!(&self.loaded, Some((loaded, _)) if loaded == path) {
      self.loaded = None;
      let session = Model::load(path)
        .and_then(|model| model.session())
        .map_err(|error| format!("Could not load the dictation model: {error}"))?;
      self.loaded = Some((path.to_owned(), session));
    }
    self
      .loaded
      .as_mut()
      .map(|(_, session)| session)
      .ok_or_else(|| "The dictation model is not loaded".to_owned())
  }
}

/// How much audio each step hears, and so how far the words lag behind the
/// voice: twice this. A GPU keeps up with the shortest step that stays
/// accurate, 160 ms; on the CPU each step re-reads the left context, so it
/// takes 560 ms steps to stay ahead of the voice.
fn chunk_ms(model: &Model) -> i32 {
  if model.backend().eq_ignore_ascii_case("cpu") {
    560
  } else {
    160
  }
}

fn text(text: StreamText) -> DictatedText {
  DictatedText {
    settled: text.committed,
    unsettled: text.tentative,
  }
}
