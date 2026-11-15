// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One dictation, on a thread of its own: the microphone opened at once, so
//! nothing said while the helper starts or loads its model is lost, and fed
//! to the helper in short steps until the dictation is finished or
//! cancelled.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use screenwide_capture::microphone::{Buffer, Source};
use screenwide_speech::runner::dictation::DictatedText;
use screenwide_speech::runner::Transcriber;

use crate::resample::ToSpeech;

/// How much speech is gathered before the helper hears it: 80 ms, the
/// model's own step, so words follow the voice without the helper being
/// asked more often than it can hear.
const STEP_SAMPLES: usize = 1_280;
/// How often the microphone's level is told.
const LEVEL_EVERY: Duration = Duration::from_millis(33);
/// How long the thread waits for a stop before looking at the microphone.
const WAIT: Duration = Duration::from_millis(15);

/// What a dictation tells as it goes.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
  /// How loud the microphone is, in decibels, about 30 times a second.
  Level(f32),
  /// The text heard so far, whenever it changes.
  Heard(DictatedText),
  /// Listening cannot go on: the microphone, the helper or its model failed.
  /// [`Session::finish`] then says why.
  Failed,
}

/// How a dictation ended: everything heard, which is empty when it was
/// cancelled, or why it failed, in English for the log; and the helper, for
/// the next dictation, while it is still sound.
pub struct Ended {
  pub heard: Result<DictatedText, String>,
  pub helper: Option<Transcriber>,
}

enum Stop {
  Finish,
  Cancel,
}

enum Heard {
  Samples(Vec<f32>),
  Failed(String),
}

pub struct Session {
  stop: Sender<Stop>,
  worker: JoinHandle<Ended>,
}

/// What a dictation listens with.
pub struct Setup {
  /// The streaming model's file.
  pub model: PathBuf,
  /// The microphone's id, or none for the system's default, which also
  /// stands in for one that is gone.
  pub microphone: Option<String>,
  /// Names and terms to write where the audio fits them better than what
  /// was heard.
  pub words: Vec<String>,
}

impl Session {
  /// Starts listening as `setup` says, on `helper` if one is kept from an
  /// earlier dictation, telling `on_event` as it goes.
  pub fn start(
    setup: Setup,
    helper: Option<Transcriber>,
    on_event: impl Fn(Event) + Send + 'static,
  ) -> Self {
    let (stop, stops) = mpsc::channel();
    let worker = std::thread::Builder::new()
      .name("dictation".to_owned())
      .spawn(move || {
        let ended = listen(setup, helper, &stops, &on_event);
        if ended.heard.is_err() {
          on_event(Event::Failed);
        }
        ended
      })
      .unwrap_or_else(|error| {
        let message = format!("Could not start dictating: {error}");
        std::thread::spawn(move || Ended {
          heard: Err(message),
          helper: None,
        })
      });
    Self { stop, worker }
  }

  /// Stops listening and waits for the rest of what was said to be heard.
  pub fn finish(self) -> Ended {
    self.end(Stop::Finish)
  }

  /// Stops listening and drops what was heard.
  pub fn cancel(self) -> Ended {
    self.end(Stop::Cancel)
  }

  fn end(self, stop: Stop) -> Ended {
    let _ = self.stop.send(stop);
    self.worker.join().unwrap_or_else(|_| Ended {
      heard: Err("Dictation stopped unexpectedly".to_owned()),
      helper: None,
    })
  }
}

fn listen(
  Setup {
    model,
    microphone,
    words,
  }: Setup,
  helper: Option<Transcriber>,
  stops: &Receiver<Stop>,
  on_event: &dyn Fn(Event),
) -> Ended {
  let failed = |error: String, helper| Ended {
    heard: Err(error),
    helper,
  };
  let (sender, heard) = mpsc::channel();
  // A microphone chosen and since unplugged gives way to the default, as
  // one not chosen does.
  let microphone = match microphone
    .as_deref()
    .and_then(|id| Source::resolve(id).ok())
    .map_or_else(|| Source::resolve_or_default(None), Ok)
  {
    Ok(source) => source,
    Err(error) => return failed(format!("Could not open the microphone: {error}"), helper),
  };
  let format = microphone.format();
  let samples = sender.clone();
  let stream = match microphone.start(
    Arc::new(move |buffer: Buffer| {
      let _ = samples.send(Heard::Samples(buffer.samples));
    }),
    Arc::new(move |error| {
      let _ = sender.send(Heard::Failed(error));
    }),
  ) {
    Ok(stream) => stream,
    Err(error) => return failed(format!("Could not open the microphone: {error}"), helper),
  };
  let mut helper = match helper.map_or_else(Transcriber::start_dictation, Ok) {
    Ok(helper) => helper,
    Err(error) => return failed(error, None),
  };
  if let Err(error) = helper.begin_dictation(model, words) {
    return failed(error, None);
  }

  let mut to_speech = ToSpeech::new(format.channels, format.sample_rate);
  let mut pending = Vec::new();
  let mut peak = 0.0_f32;
  let mut told_level = Instant::now();
  let mut text = DictatedText::default();
  let stop = loop {
    let stop = match stops.recv_timeout(WAIT) {
      Ok(stop) => Some(stop),
      Err(RecvTimeoutError::Timeout) => None,
      Err(RecvTimeoutError::Disconnected) => Some(Stop::Cancel),
    };
    if let Err(error) = take(&heard, &mut to_speech, &mut pending, &mut peak) {
      return failed(error, Some(helper));
    }
    if told_level.elapsed() >= LEVEL_EVERY {
      on_event(Event::Level(20.0 * peak.max(1e-8).log10()));
      peak = 0.0;
      told_level = Instant::now();
    }
    if let Some(stop) = stop {
      break stop;
    }
    if pending.len() >= STEP_SAMPLES {
      match helper.feed_dictation(std::mem::take(&mut pending)) {
        Ok(now) if now != text => {
          text.clone_from(&now);
          on_event(Event::Heard(now));
        }
        Ok(_) => {}
        Err(error) => return failed(error, None),
      }
    }
  };

  // What the microphone delivered before it closed is part of what was said.
  drop(stream);
  let _ = take(&heard, &mut to_speech, &mut pending, &mut peak);
  if matches!(stop, Stop::Finish) && !pending.is_empty() {
    if let Err(error) = helper.feed_dictation(pending) {
      return failed(error, None);
    }
  }
  match helper.finish_dictation() {
    Ok(_) if matches!(stop, Stop::Cancel) => Ended {
      heard: Ok(DictatedText::default()),
      helper: Some(helper),
    },
    Ok(heard) => Ended {
      heard: Ok(heard),
      helper: Some(helper),
    },
    Err(error) => failed(error, None),
  }
}

/// Brings everything the microphone has delivered into `pending`, noting its
/// loudest sample in `peak`.
fn take(
  heard: &Receiver<Heard>,
  to_speech: &mut ToSpeech,
  pending: &mut Vec<i16>,
  peak: &mut f32,
) -> Result<(), String> {
  while let Ok(heard) = heard.try_recv() {
    match heard {
      Heard::Samples(samples) => {
        *peak = samples
          .iter()
          .fold(*peak, |peak, sample| peak.max(sample.abs()));
        to_speech.push(&samples, pending);
      }
      Heard::Failed(error) => return Err(format!("The microphone stopped: {error}")),
    }
  }
  Ok(())
}
