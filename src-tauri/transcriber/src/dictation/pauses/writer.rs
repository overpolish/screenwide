// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each stretch of a dictation written out once a pause ends it. The speech
//! since the last fixed point is written again as a whole each time, so a
//! sentence broken by a pause reads as one and a short stretch, such as a
//! spoken command, is heard in context; it is fixed once it runs past
//! [`FIX_AFTER`]. What was written is shared out among the stretches by
//! when each word, or Whisper's each segment, was said, so a command said
//! on its own between pauses stays on its own.

use std::sync::mpsc::Receiver;
use std::sync::{Mutex, MutexGuard, PoisonError};

use screenwide_transcriber::{DictatedText, DictatedWord, SAMPLE_RATE};
use transcribe_cpp::{RunOptions, Session, TimestampKind, Transcript};

use super::stretches::Stretch;

/// Audio after a stretch's speech written with it, as the voice fades past
/// where the voice activity model put its end: 300 ms.
const TAIL: usize = 4_800;
/// Speech written together past this is fixed, so each pass writes at most
/// about this much again: 20 s, within Whisper's 30 s window.
const FIX_AFTER: usize = 320_000;
/// How much of the text before the speech written is given as its context.
const CONTEXT_CHARS: usize = 240;

/// What a dictation has written so far, shared with the thread that answers
/// the app.
#[derive(Debug, Default)]
pub struct Written {
  fixed: Vec<Piece>,
  /// The speech since the last fixed point, a piece for each stretch.
  open: Vec<Piece>,
  open_stretches: Vec<Stretch>,
  /// Where the speech not yet fixed begins, in samples.
  open_from: usize,
  pub failed: Option<String>,
}

/// What one stretch of speech was written as.
#[derive(Debug)]
struct Piece {
  text: String,
  stretch: Stretch,
}

/// How each stretch is written.
pub struct Writing {
  pub run: RunOptions,
  /// Whether the model takes the text before as context: Whisper does.
  pub takes_context: bool,
}

/// Writes each stretch `stretches` brings, from `audio`, into `written`,
/// until `stretches` closes or writing fails.
pub fn write(
  session: &mut Session,
  stretches: &Receiver<Stretch>,
  audio: &Mutex<Vec<f32>>,
  written: &Mutex<Written>,
  writing: &Writing,
) {
  for stretch in stretches {
    if let Err(error) = write_stretch(session, stretch, audio, written, writing) {
      lock(written).failed = Some(error);
      return;
    }
  }
}

/// Writes the speech since the last fixed point again, `stretch` with it.
fn write_stretch(
  session: &mut Session,
  stretch: Stretch,
  audio: &Mutex<Vec<f32>>,
  written: &Mutex<Written>,
  writing: &Writing,
) -> Result<(), String> {
  let (open_from, mut stretches, context) = {
    let written = lock(written);
    (
      written.open_from,
      written.open_stretches.clone(),
      context(&written),
    )
  };
  stretches.push(stretch);
  // Copied out, so the microphone's samples keep arriving while it is
  // written.
  let pcm = {
    let audio = lock(audio);
    audio[open_from..(stretch.end + TAIL).min(audio.len())].to_vec()
  };
  let mut run = writing.run.clone();
  run.timestamps = TimestampKind::Auto;
  run.prompt = Some(context).filter(|context| writing.takes_context && !context.is_empty());
  let transcript = session
    .run(&pcm, &run)
    .map_err(|error| format!("Could not write the dictation: {error}"))?;
  let open = shared_out(&transcript, open_from, &stretches);

  let mut written = lock(written);
  written.open = open;
  written.open_stretches = stretches;
  if pcm.len() > FIX_AFTER {
    let open = std::mem::take(&mut written.open);
    written.fixed.extend(open);
    written.open_stretches.clear();
    written.open_from += pcm.len();
  }
  Ok(())
}

/// `transcript`, of the audio from sample `from`, shared out among
/// `stretches`: each word, or each segment where the model times no words,
/// goes to the stretch its time overlaps most, or the nearest where it
/// overlaps none. Whisper's segments run on into the pauses round them, so
/// where one starts says little; how much of it is spent in each stretch
/// says where it was spoken.
fn shared_out(transcript: &Transcript, from: usize, stretches: &[Stretch]) -> Vec<Piece> {
  let timed: Vec<(i64, i64, &str)> = if transcript.words.is_empty() {
    transcript
      .segments
      .iter()
      .map(|segment| (segment.t0_ms, segment.t1_ms, segment.text.as_str()))
      .collect()
  } else {
    transcript
      .words
      .iter()
      .map(|word| (word.t0_ms, word.t1_ms, word.text.as_str()))
      .collect()
  };
  let samples_per_ms = (SAMPLE_RATE / 1_000) as usize;
  let at = |ms: i64| from + usize::try_from(ms).unwrap_or(0) * samples_per_ms;
  let mut texts = vec![String::new(); stretches.len()];
  for (t0_ms, t1_ms, text) in timed {
    let (start, end) = (at(t0_ms), at(t1_ms).max(at(t0_ms)));
    let Some(index) = stretch_of((start, end), stretches) else {
      continue;
    };
    let text = text.trim();
    if !text.is_empty() {
      if !texts[index].is_empty() {
        texts[index].push(' ');
      }
      texts[index].push_str(text);
    }
  }
  texts
    .into_iter()
    .zip(stretches)
    .filter(|(text, _)| !text.is_empty())
    .map(|(text, &stretch)| Piece { text, stretch })
    .collect()
}

/// Which of `stretches` the span `[start, end)` was spoken in: the one it
/// overlaps most, or the nearest.
fn stretch_of((start, end): (usize, usize), stretches: &[Stretch]) -> Option<usize> {
  let overlap = |stretch: &Stretch| {
    end
      .min(stretch.end)
      .saturating_sub(start.max(stretch.start))
  };
  let distance = |stretch: &Stretch| {
    stretch
      .start
      .saturating_sub(end)
      .max(start.saturating_sub(stretch.end))
  };
  stretches
    .iter()
    .enumerate()
    .max_by(|(_, a), (_, b)| {
      overlap(a)
        .cmp(&overlap(b))
        .then_with(|| distance(b).cmp(&distance(a)))
    })
    .map(|(index, _)| index)
}

/// The end of the text already fixed, given as the next pass's context.
fn context(written: &Written) -> String {
  let text = written
    .fixed
    .iter()
    .map(|piece| piece.text.as_str())
    .collect::<Vec<_>>()
    .join(" ");
  let start = text
    .char_indices()
    .rev()
    .nth(CONTEXT_CHARS)
    .map_or(0, |(at, _)| at);
  text[start..].to_owned()
}

/// The text written so far, word by word: the words of each stretch share
/// its time, so a pause falls only between stretches, where one was heard.
pub fn dictated(written: &Written, heard: usize, finished: bool) -> DictatedText {
  let ms = |samples: usize| samples as u64 * 1_000 / u64::from(SAMPLE_RATE);
  let pieces: Vec<&Piece> = written.fixed.iter().chain(&written.open).collect();
  let words = pieces
    .iter()
    .flat_map(|piece| {
      piece.text.split_whitespace().map(|word| DictatedWord {
        text: word.to_owned(),
        start_ms: ms(piece.stretch.start),
        end_ms: ms(piece.stretch.end),
        settled: true,
      })
    })
    .collect();
  DictatedText {
    settled: pieces
      .iter()
      .map(|piece| piece.text.as_str())
      .collect::<Vec<_>>()
      .join(" "),
    unsettled: String::new(),
    words,
    heard_ms: ms(heard),
    finished,
  }
}

pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
  mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
