// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Notes being taken, on a thread of their own: the microphone and the
//! computer's audio listened to, nothing recorded, each brought to what the
//! speech models hear and fed to dictation's helper a tenth of a second at a
//! time, the lines it writes saved to the Markdown file as they come. When
//! the notes finish, the voices are told apart over the whole call, the file
//! is written a last time and a clip of each voice is saved beside it.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use screenwide_capture::capture::{begin_listening_blocking, ListeningSession};
use screenwide_capture::listening::{Heard, HeardSource, Listener, Listening};
use screenwide_speech::resample::ToSpeech;
use screenwide_speech::runner::notes::{NoteSource, NotesModels};
use screenwide_speech::runner::Transcriber;

use super::Document;

/// How much of a source is gathered before the helper hears it: 100 ms at
/// the speech models' rate.
const FEED: usize = 1_600;
/// A run that starts this much after where its source's gathered samples
/// end follows a silence the source sent nothing for, in milliseconds.
const GAP_MS: u64 = 200;
/// How often the thread looks for a stop between runs.
const STOP_POLL: Duration = Duration::from_millis(100);

/// What notes are taken with, and where they are written.
pub struct Setup {
  pub models: NotesModels,
  /// A language code such as `en`, or none to detect it.
  pub language: Option<String>,
  pub words: Vec<String>,
  pub listening: Listening,
  /// The Markdown file the notes are saved to.
  pub path: PathBuf,
  /// What the notes are headed, such as when the call was.
  pub title: String,
}

/// Why notes stopped on their own.
pub enum Failure {
  /// Dictation's helper or its models could not be readied, so nothing was
  /// written.
  Readying(String),
  /// Taking them failed part way.
  Taking(String),
}

/// Notes under way.
pub struct Session {
  listening: Option<ListeningSession>,
  stop: Sender<Stop>,
  worker: JoinHandle<Result<(), String>>,
}

impl Session {
  /// Starts taking notes as `setup` says, telling `on_ready` once the
  /// models have loaded and `on_failure` why if they stop on their own.
  /// Listening begins before this returns; dictation's helper starts and
  /// loads its models afterwards, on the notes' own thread, which takes
  /// seconds, and what is heard meanwhile waits for it.
  pub fn start(
    setup: Setup,
    on_ready: impl FnOnce() + Send + 'static,
    on_failure: impl FnOnce(Failure) + Send + 'static,
  ) -> Result<Self, String> {
    let Setup {
      models,
      language,
      words,
      listening,
      path,
      title,
    } = setup;
    let (heard, runs) = mpsc::channel();
    let listening = begin_listening_blocking(listening, listener(heard))?;
    let (stop, stops) = mpsc::channel();
    let worker = std::thread::Builder::new()
      .name("screenwide-notes".to_owned())
      .spawn(move || {
        let mut helper = match ready(models, language, words) {
          Ok(helper) => helper,
          Err(error) => {
            on_failure(Failure::Readying(error.clone()));
            return Err(error);
          }
        };
        on_ready();
        let taken = take(&mut helper, &runs, &stops, (path, title));
        if let Err(error) = &taken {
          on_failure(Failure::Taking(error.clone()));
        }
        taken
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      listening: Some(listening),
      stop,
      worker,
    })
  }

  /// Stops listening, writes what is still to be written and tells the
  /// voices apart, the file written a last time.
  pub fn finish(self) -> Result<(), String> {
    self.stop_with(Stop::Finish)
  }

  /// Stops listening and leaves the file as it stands, for notes being
  /// thrown away.
  pub fn abandon(self) -> Result<(), String> {
    self.stop_with(Stop::Abandon)
  }

  fn stop_with(mut self, stop: Stop) -> Result<(), String> {
    if let Some(listening) = self.listening.take() {
      listening.stop();
    }
    let _ = self.stop.send(stop);
    self
      .worker
      .join()
      .map_err(|_| "Taking notes stopped unexpectedly".to_owned())?
  }
}

/// Dictation's helper, started with the notes' models loaded.
fn ready(
  models: NotesModels,
  language: Option<String>,
  words: Vec<String>,
) -> Result<Transcriber, String> {
  let mut helper = Transcriber::start_dictation()?;
  helper.begin_notes(models, language, words)?;
  Ok(helper)
}

/// A run of samples brought to the speech models' rate, and when, in
/// milliseconds from the notes' start, it was heard.
type Run = (NoteSource, u64, Vec<i16>);

/// What the capture hands each run to: it is brought to the speech models'
/// rate, each source by its own resampler, and sent on.
fn listener(runs: Sender<Run>) -> Listener {
  let resamplers: Arc<Mutex<[Option<ToSpeech>; 2]>> = Arc::default();
  Arc::new(move |heard: Heard| {
    let (source, at) = match heard.source {
      HeardSource::Microphone => (NoteSource::Microphone, 0),
      HeardSource::System => (NoteSource::System, 1),
    };
    let mut samples = Vec::new();
    {
      let mut resamplers = resamplers.lock().unwrap_or_else(PoisonError::into_inner);
      resamplers[at]
        .get_or_insert_with(|| ToSpeech::new(heard.channels, heard.sample_rate))
        .push(&heard.samples, &mut samples);
    }
    let at_ms = u64::try_from(heard.at.as_millis()).unwrap_or(u64::MAX);
    let _ = runs.send((source, at_ms, samples));
  })
}

/// How notes under way are stopped.
enum Stop {
  /// What is still gathered is written and the voices told apart.
  Finish,
  /// Left as they are, the helper let go at once: they are being thrown
  /// away.
  Abandon,
}

/// Feeds `runs` to `helper` until a stop comes, saving the lines as they are
/// written, then finishes the notes unless they are abandoned.
fn take(
  helper: &mut Transcriber,
  runs: &Receiver<Run>,
  stops: &Receiver<Stop>,
  (path, title): (PathBuf, String),
) -> Result<(), String> {
  let mut document = Document::new(title);
  document.save(&path).map_err(|error| error.to_string())?;
  let mut gathered: [Gathered; 2] = Default::default();
  let add = |document: &mut Document, lines: Vec<_>| -> Result<(), String> {
    if lines.is_empty() {
      return Ok(());
    }
    document.add(lines);
    document.save(&path).map_err(|error| error.to_string())
  };
  loop {
    let stopping = match stops.try_recv() {
      Ok(Stop::Abandon) => return Ok(()),
      Ok(Stop::Finish) => true,
      Err(_) => false,
    };
    match runs.recv_timeout(STOP_POLL) {
      Ok((source, at_ms, samples)) => {
        let pending = &mut gathered[index(source)];
        if let Some(full) = pending.add(at_ms, &samples) {
          let lines = helper.feed_notes(source, full.0, full.1)?;
          add(&mut document, lines)?;
        }
        if !stopping {
          continue;
        }
      }
      Err(RecvTimeoutError::Timeout) if !stopping => continue,
      Err(_) => {}
    }
    // Stopping, or the capture is gone: the rest of what is gathered, then
    // the end.
    while let Ok((source, at_ms, samples)) = runs.try_recv() {
      if let Some(full) = gathered[index(source)].add(at_ms, &samples) {
        let lines = helper.feed_notes(source, full.0, full.1)?;
        add(&mut document, lines)?;
      }
    }
    for source in [NoteSource::Microphone, NoteSource::System] {
      if let Some((at_ms, samples)) = gathered[index(source)].take() {
        let lines = helper.feed_notes(source, at_ms, samples)?;
        add(&mut document, lines)?;
      }
    }
    let finished = helper.finish_notes()?;
    document.finish(finished.lines, finished.turns);
    document.save(&path).map_err(|error| error.to_string())?;
    let root = path
      .parent()
      .ok_or_else(|| "The notes have no folder".to_owned())?;
    return super::clips::save(root, &finished.voices, &document.voices())
      .map_err(|error| error.to_string());
  }
}

fn index(source: NoteSource) -> usize {
  match source {
    NoteSource::Microphone => 0,
    NoteSource::System => 1,
  }
}

/// A source's samples gathered until there are enough to feed, and when the
/// first was heard.
#[derive(Default)]
struct Gathered {
  start_ms: u64,
  samples: Vec<i16>,
}

impl Gathered {
  /// Gathers `samples`, heard at `at_ms`, answering what is ready to feed:
  /// a full tenth of a second, or what was gathered before a silence the
  /// source sent nothing for.
  fn add(&mut self, at_ms: u64, samples: &[i16]) -> Option<(u64, Vec<i16>)> {
    let ends_ms = self.start_ms + self.samples.len() as u64 * 1_000 / 16_000;
    let ready = if !self.samples.is_empty() && at_ms > ends_ms + GAP_MS {
      self.take()
    } else {
      None
    };
    if self.samples.is_empty() {
      self.start_ms = at_ms;
    }
    self.samples.extend_from_slice(samples);
    ready.or_else(|| (self.samples.len() >= FEED).then(|| self.take()).flatten())
  }

  fn take(&mut self) -> Option<(u64, Vec<i16>)> {
    (!self.samples.is_empty()).then(|| (self.start_ms, std::mem::take(&mut self.samples)))
  }
}

#[cfg(test)]
mod tests {
  use super::Gathered;

  #[test]
  fn runs_are_fed_a_tenth_of_a_second_at_a_time_and_split_at_silences() {
    let mut gathered = Gathered::default();
    assert_eq!(gathered.add(0, &[1; 800]), None);
    let (at_ms, full) = gathered.add(50, &[1; 800]).expect("a tenth of a second");
    assert_eq!((at_ms, full.len()), (0, 1_600));
    assert_eq!(gathered.add(100, &[1; 160]), None);
    // A second of silence the source sent nothing for.
    let (at_ms, before) = gathered.add(1_200, &[2; 160]).expect("what came before");
    assert_eq!((at_ms, before.len()), (100, 160));
    assert_eq!(gathered.take(), Some((1_200, vec![2; 160])));
  }
}
