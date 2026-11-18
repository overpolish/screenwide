// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation that streams, so the words appear as they are spoken, and
//! writes each stretch again once a pause ends it with a model hearing the
//! whole stretch. A streaming model has to settle on each word with little
//! audio after it, so a "three" said like "free" stays "free"; heard with
//! the words round it, it is "three". The text written again takes the place
//! of the streamed words it covers, and the streamed words after it show as
//! they come.

use std::io::{BufRead, Lines};
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};

use screenwide_transcriber::{DictatedText, DictatedWord, Job, Reply, Request, SAMPLE_RATE};
use transcribe_cpp::{Feature, RunOptions, Session, Stream, VadSession};

use super::found;
use super::stretches::{Pauses, Said};
use super::writer::{self, dictated, lock, Writing, Written};
use crate::stream::{self, Dictation};

/// What a dictation written again at pauses was begun with.
pub struct Rewritten {
  pub model: PathBuf,
  pub rewriter: PathBuf,
  pub vad: PathBuf,
  pub language: Option<String>,
  pub words: Vec<String>,
}

/// Runs the dictation `begin` asked for, as [`Dictation::run`] runs one that
/// only streams. False once the app has stopped listening.
pub fn run<B: BufRead>(
  dictation: &mut Dictation,
  begin: u64,
  rewritten: Rewritten,
  lines: &mut Lines<B>,
  send: &mut dyn FnMut(&Reply) -> bool,
) -> bool {
  let models = (
    rewritten.model.as_path(),
    rewritten.rewriter.as_path(),
    rewritten.vad.as_path(),
  );
  let (session, rewriter, vad) = match dictation.rewriting_sessions(models) {
    Ok(sessions) => sessions,
    Err(message) => return send(&Reply::Failed { id: begin, message }),
  };
  vad.stream_reset();
  let writing = Writing {
    takes_context: rewriter.model().supports(Feature::ContextPrompt),
    run: RunOptions {
      language: rewritten.language,
      vocabulary: rewritten.words.clone(),
      ..RunOptions::default()
    },
  };
  let mut stream = match stream::start(session, rewritten.words) {
    Ok(stream) => stream,
    Err(message) => return send(&Reply::Failed { id: begin, message }),
  };
  if !send(&Reply::Dictated {
    id: begin,
    text: DictatedText::default(),
  }) {
    return false;
  }
  listen((&mut stream, rewriter, vad), &writing, lines, send)
}

fn listen<B: BufRead>(
  (stream, rewriter, vad): (&mut Stream<'_>, &mut Session, &mut VadSession),
  writing: &Writing,
  lines: &mut Lines<B>,
  send: &mut dyn FnMut(&Reply) -> bool,
) -> bool {
  let audio = Mutex::new(Vec::new());
  let written = Mutex::new(Written::default());
  std::thread::scope(|scope| {
    let (stretches, to_write) = mpsc::channel::<Said>();
    let (shared_audio, shared_written) = (&audio, &written);
    let writer = scope.spawn(move || {
      writer::write(rewriter, &to_write, shared_audio, shared_written, writing);
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
          let streamed = stream.feed(&pcm);
          let heard = vad.stream_feed(&pcm);
          match (streamed, heard, &stretches) {
            (Ok(update), Ok(heard), Some(stretches)) => {
              for stretch in found(&mut pauses, &heard) {
                let _ = stretches.send(stretch);
              }
              let heard_ms = u64::try_from(update.input_received_ms).unwrap_or(0);
              answer(id, &written, stream::text(stream, Some(heard_ms)))
            }
            (Err(error), _, _) | (_, Err(error), _) => {
              failed(id, format!("Could not hear the dictation: {error}"))
            }
            (Ok(_), Ok(_), None) => failed(id, "The dictation has finished".to_owned()),
          }
        }
        Job::FinishDictation => {
          let streamed = stream.finalize();
          let mut last: Vec<Said> = vad
            .stream_flush()
            .map(|heard| found(&mut pauses, &heard))
            .unwrap_or_default();
          last.extend(pauses.end().map(|stretch| pauses.said(stretch)));
          if let Some(stretches) = stretches.take() {
            for stretch in last {
              let _ = stretches.send(stretch);
            }
          }
          let _ = writer.join();
          let reply = match streamed {
            Ok(_) => answer(id, &written, stream::text(stream, None)),
            Err(error) => failed(id, format!("Could not finish the dictation: {error}")),
          };
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

fn answer(id: u64, written: &Mutex<Written>, streamed: DictatedText) -> Reply {
  let written = lock(written);
  if let Some(message) = &written.failed {
    return failed(id, message.clone());
  }
  let covered_ms = written.covered as u64 * 1_000 / u64::from(SAMPLE_RATE);
  let finished = streamed.finished;
  Reply::Dictated {
    id,
    text: combined(dictated(&written, 0, finished), streamed, covered_ms),
  }
}

/// The text written again, up to `covered_ms` into the dictation, then the
/// streamed words said after it.
fn combined(written: DictatedText, streamed: DictatedText, covered_ms: u64) -> DictatedText {
  let after: Vec<DictatedWord> = streamed
    .words
    .into_iter()
    .filter(|word| word.start_ms >= covered_ms)
    .collect();
  // The last word written again was written once the pause after it ended.
  let paused_after = after.is_empty() && !written.words.is_empty();
  let mut words = written.words;
  words.extend(after);
  DictatedText {
    settled: words
      .iter()
      .map(|word| word.text.as_str())
      .collect::<Vec<_>>()
      .join(" "),
    unsettled: String::new(),
    words,
    heard_ms: streamed.heard_ms,
    finished: streamed.finished,
    paused_after,
  }
}

fn failed(id: u64, message: String) -> Reply {
  Reply::Failed { id, message }
}

#[cfg(test)]
mod tests {
  use screenwide_transcriber::{DictatedText, DictatedWord};

  use super::combined;

  fn text(words: &[(&str, u64, bool)]) -> DictatedText {
    DictatedText {
      words: words
        .iter()
        .map(|&(text, start_ms, settled)| DictatedWord {
          text: text.to_owned(),
          start_ms,
          end_ms: start_ms + 200,
          settled,
        })
        .collect(),
      heard_ms: 4_000,
      ..DictatedText::default()
    }
  }

  #[test]
  fn the_words_written_again_take_the_streamed_words_place() {
    let written = text(&[("Coffee", 0, true), ("at", 0, true), ("three.", 0, true)]);
    let streamed = text(&[
      ("Coffee", 100, true),
      ("at", 400, true),
      ("free.", 700, true),
      ("Actually", 2_500, false),
    ]);
    let both = combined(written, streamed, 1_300);
    let said: Vec<&str> = both.words.iter().map(|word| word.text.as_str()).collect();
    assert_eq!(said, ["Coffee", "at", "three.", "Actually"]);
    assert!(!both.words[3].settled);
    assert!(!both.paused_after);
  }

  #[test]
  fn with_nothing_after_the_words_written_again_a_pause_follows_them() {
    let written = text(&[("Coffee", 0, true), ("at", 0, true), ("three.", 0, true)]);
    let streamed = text(&[
      ("Coffee", 100, true),
      ("at", 400, true),
      ("free.", 700, true),
    ]);
    let both = combined(written, streamed, 1_300);
    assert_eq!(both.words.len(), 3);
    assert!(both.paused_after);
  }

  #[test]
  fn before_any_pause_the_streamed_words_show() {
    let streamed = text(&[("Coffee", 100, true), ("at", 400, false)]);
    let both = combined(DictatedText::default(), streamed, 0);
    assert_eq!(both.words.len(), 2);
    assert!(!both.paused_after);
  }
}
