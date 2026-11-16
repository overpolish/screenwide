// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation: speech written out as it is spoken, by a streaming model run
//! through transcribe.cpp. A dictation takes the helper's input over from
//! [`Job::BeginDictation`] to [`Job::FinishDictation`], answering each request
//! itself, as its stream borrows the model's session for that long.

use std::io::{BufRead, Lines};
use std::path::{Path, PathBuf};

use screenwide_transcriber::{DictatedText, DictatedWord, Job, Reply, Request};
use transcribe_cpp::{
  CommitPolicy, Model, ParakeetBufferedStreamOptions, RunOptions, Session, Stream, StreamExtension,
  StreamOptions, TimestampKind, VadSession,
};

/// Parakeet Unified's buffered stream re-reads this much of what came before
/// each chunk, the only left context it was trained with.
const LEFT_MS: i32 = 5_600;

/// The model last loaded and a session on it, kept for the dictations after:
/// loading takes far longer than the first words. And the voice activity
/// model, for dictations written at each pause, and the model that writes
/// each stretch again for one that streams and is then written again.
#[derive(Default)]
pub struct Dictation {
  loaded: Option<(PathBuf, Session)>,
  vad: Option<(PathBuf, VadSession)>,
  rewriter: Option<(PathBuf, Session)>,
}

impl Dictation {
  /// Runs the dictation `begin` asked for, writing `words` where the audio
  /// fits them better than what was heard, reading its requests from `lines`
  /// and sending every reply, the one to `begin` included, through `send`.
  /// False once the app has stopped listening.
  pub fn run<B: BufRead>(
    &mut self,
    begin: u64,
    (model, words): (&Path, Vec<String>),
    lines: &mut Lines<B>,
    send: &mut dyn FnMut(&Reply) -> bool,
  ) -> bool {
    let session = match self.session(model) {
      Ok(session) => session,
      Err(message) => return send(&Reply::Failed { id: begin, message }),
    };
    let mut stream = match start(session, words) {
      Ok(stream) => stream,
      Err(message) => return send(&Reply::Failed { id: begin, message }),
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
            Ok(update) => Reply::Dictated {
              id,
              text: text(
                &stream,
                Some(u64::try_from(update.input_received_ms).unwrap_or(0)),
              ),
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
              text: text(&stream, None),
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

  /// A session on the model at `model`, and one on the voice activity model
  /// at `vad`, each loaded unless it is already.
  pub fn sessions(
    &mut self,
    model: &Path,
    vad: &Path,
  ) -> Result<(&mut Session, &mut VadSession), String> {
    self.session(model)?;
    if !matches!(&self.vad, Some((loaded, _)) if loaded == vad) {
      self.vad = None;
      let session = Model::load(vad)
        .and_then(|model| model.vad_session())
        .map_err(|error| format!("Could not load the voice activity model: {error}"))?;
      self.vad = Some((vad.to_owned(), session));
    }
    match (&mut self.loaded, &mut self.vad) {
      (Some((_, session)), Some((_, vad))) => Ok((session, vad)),
      _ => Err("The dictation models are not loaded".to_owned()),
    }
  }

  /// Sessions on the streaming model at `model`, on the model at `rewriter`
  /// that writes each stretch again, and on the voice activity model at
  /// `vad`, each loaded unless it is already.
  pub fn rewriting_sessions(
    &mut self,
    (model, rewriter, vad): (&Path, &Path, &Path),
  ) -> Result<(&mut Session, &mut Session, &mut VadSession), String> {
    self.sessions(model, vad)?;
    if !matches!(&self.rewriter, Some((loaded, _)) if loaded == rewriter) {
      self.rewriter = None;
      let session = Model::load(rewriter)
        .and_then(|model| model.session())
        .map_err(|error| format!("Could not load the dictation model: {error}"))?;
      self.rewriter = Some((rewriter.to_owned(), session));
    }
    match (&mut self.loaded, &mut self.rewriter, &mut self.vad) {
      (Some((_, session)), Some((_, rewriter)), Some((_, vad))) => Ok((session, rewriter, vad)),
      _ => Err("The dictation models are not loaded".to_owned()),
    }
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

/// A stream on `session`, writing `words` where the audio fits them better
/// than what was heard.
pub fn start(session: &mut Session, words: Vec<String>) -> Result<Stream<'_>, String> {
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
  let run = RunOptions {
    timestamps: TimestampKind::Word,
    vocabulary: words,
    ..RunOptions::default()
  };
  session
    .stream(&run, &options)
    .map_err(|error| format!("Could not start dictating: {error}"))
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

/// What the stream has heard, and when each word of it was said. A stream
/// times its tokens alone, so words are put together from them: a token
/// starting with a space starts a word, as does the token after a space
/// alone, which is how the model writes the space before a number. A word
/// ends with its last letter or digit: the model writes a full stop only
/// once the pause after it has passed. A word is settled while its place in
/// the text falls within the settled part. `heard_ms` is how much has been
/// heard; none once the stream is finished. The text is the stream's whole
/// hypothesis, which writes a listed word in place of the words heard as
/// soon as the audio fits it better: the stream's committed text only ever
/// grows, so it cannot show that.
pub fn text(stream: &Stream<'_>, heard_ms: Option<u64>) -> DictatedText {
  let mut heard = stream.text();
  let finished = heard_ms.is_none();
  heard.committed = heard.full.trim().to_owned();
  heard.tentative.clear();
  let mut words: Vec<DictatedWord> = Vec::new();
  let mut after_space = false;
  for token in stream.snapshot().tokens {
    let starts = after_space || token.text.starts_with([' ', '\u{2581}']);
    let piece = token.text.trim_start_matches([' ', '\u{2581}']);
    if piece.is_empty() {
      after_space = starts;
      continue;
    }
    after_space = false;
    let start_ms = u64::try_from(token.t0_ms).unwrap_or(0);
    let end_ms = u64::try_from(token.t1_ms).unwrap_or(0);
    match words.last_mut() {
      Some(word) if !starts => {
        word.text.push_str(piece);
        if piece.chars().any(char::is_alphanumeric) {
          word.end_ms = end_ms;
        }
      }
      _ => words.push(DictatedWord {
        text: piece.to_owned(),
        start_ms,
        end_ms,
        settled: false,
      }),
    }
  }
  let mut place = 0;
  for word in &mut words {
    if let Some(at) = heard.committed[place..].find(&word.text) {
      place += at + word.text.len();
      word.settled = true;
    }
  }
  DictatedText {
    settled: heard.committed,
    unsettled: heard.tentative,
    heard_ms: heard_ms.unwrap_or_else(|| words.last().map_or(0, |word| word.end_ms)),
    words,
    finished,
    paused_after: false,
  }
}
