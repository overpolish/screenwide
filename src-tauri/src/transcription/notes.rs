// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Voice notes waiting to be transcribed, done one at a time on a thread of
//! their own. A note is queued when its recording stops and whenever the
//! Editor reads its moments, so a note missed while there was no model, or
//! while the app was closed, is picked up the next time it is looked at.

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, MutexGuard};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::catalogue::{self, Purpose};
use super::runner::{decode, Transcriber};
use super::{language, models};

/// Any note's transcript has moved on: queued, started, finished, or failed.
/// Whoever shows notes reads them again.
const NOTES_CHANGED_EVENT: &str = "transcription://notes-changed";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Job {
  moments: PathBuf,
  moment: usize,
}

#[derive(Default)]
struct Queue {
  waiting: VecDeque<Job>,
  current: Option<Job>,
  /// Notes that could not be transcribed, left alone until the app is next
  /// started rather than tried again each time they are looked at.
  failed: HashSet<Job>,
  working: bool,
}

static QUEUE: LazyLock<Mutex<Queue>> = LazyLock::new(|| Mutex::new(Queue::default()));

fn queue() -> MutexGuard<'static, Queue> {
  QUEUE
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Where a voice note's transcript is, as the Editor shows it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "status"
)]
pub(crate) enum NoteTranscript {
  Failed,
  /// Names the model that would transcribe it, to offer to download.
  NoModel {
    model: &'static str,
  },
  Queued,
  Ready {
    text: String,
  },
  Transcribing,
}

pub(crate) fn changed(app: &AppHandle) {
  let _ = app.emit(NOTES_CHANGED_EVENT, ());
}

/// The transcript of the `moment`th moment's note in the moments file at
/// `moments`, `text` once it has one. A note not yet transcribed is queued
/// if it can be. Nothing while notes are not to be transcribed.
pub(crate) fn status(
  app: &AppHandle,
  moments: &Path,
  moment: usize,
  text: Option<&str>,
) -> Option<NoteTranscript> {
  if let Some(text) = text {
    return Some(NoteTranscript::Ready {
      text: text.to_owned(),
    });
  }
  if !crate::moments::settings::current().transcribe_notes {
    return None;
  }
  let job = Job {
    moment,
    moments: moments.to_owned(),
  };
  {
    let queue = queue();
    if queue.current.as_ref() == Some(&job) {
      return Some(NoteTranscript::Transcribing);
    }
    if queue.waiting.contains(&job) {
      return Some(NoteTranscript::Queued);
    }
    if queue.failed.contains(&job) {
      return Some(NoteTranscript::Failed);
    }
  }
  if models::usable(app, Purpose::Moments).is_none() {
    return catalogue::for_purpose(Purpose::Moments)
      .map(|model| NoteTranscript::NoModel { model: model.id });
  }
  enqueue(app, job);
  Some(NoteTranscript::Queued)
}

/// Queues every untranscribed note of the recording project at `project`.
pub(crate) fn queue_project(app: &AppHandle, project: &Path) {
  let Some(moments) = crate::moments::for_project(project) else {
    return;
  };
  let Ok(recorded) = crate::moments::read(&moments) else {
    return;
  };
  for (moment, recorded) in recorded.iter().enumerate() {
    if recorded.note_duration_ms.is_some() {
      status(app, &moments, moment, recorded.transcript.as_deref());
    }
  }
}

fn enqueue(app: &AppHandle, job: Job) {
  let mut queue = queue();
  if queue.current.as_ref() == Some(&job) || queue.waiting.contains(&job) {
    return;
  }
  queue.waiting.push_back(job);
  if queue.working {
    return;
  }
  let worker_app = app.clone();
  let started = std::thread::Builder::new()
    .name("transcription".to_owned())
    .spawn(move || work(&worker_app));
  queue.working = started.is_ok();
  if let Err(error) = started {
    eprintln!("Could not start transcribing notes: {error}");
  }
  drop(queue);
  changed(app);
}

fn work(app: &AppHandle) {
  let mut transcriber: Option<Transcriber> = None;
  loop {
    let job = {
      let mut queue = queue();
      let Some(job) = queue.waiting.pop_front() else {
        queue.current = None;
        queue.working = false;
        break;
      };
      queue.current = Some(job.clone());
      job
    };
    changed(app);
    let result = transcribe(app, &job, &mut transcriber);
    {
      let mut queue = queue();
      queue.current = None;
      if let Err(error) = &result {
        eprintln!("Could not transcribe a voice note: {error}");
        queue.failed.insert(job);
      }
    }
    // A failure may have left the program in any state; the next note
    // starts it afresh.
    if result.is_err() {
      transcriber = None;
    }
    changed(app);
  }
}

/// Transcribes `job` into its moments file. A note that no longer needs it,
/// or has nothing to be transcribed with now, is passed over: looking at it
/// again queues it once it can be.
fn transcribe(
  app: &AppHandle,
  job: &Job,
  transcriber: &mut Option<Transcriber>,
) -> Result<(), String> {
  if !crate::moments::settings::current().transcribe_notes {
    return Ok(());
  }
  let needed = crate::moments::read(&job.moments)?
    .get(job.moment)
    .is_some_and(|recorded| recorded.note_duration_ms.is_some() && recorded.transcript.is_none());
  let Some(model) = models::usable(app, Purpose::Moments) else {
    return Ok(());
  };
  if !needed {
    return Ok(());
  }
  let folder = job.moments.parent().unwrap_or(Path::new(""));
  let samples = decode(&crate::moments::note_path(folder, job.moment))?;
  let running = match transcriber {
    Some(running) => running,
    None => transcriber.insert(Transcriber::start()?),
  };
  let transcript = running.transcribe(
    models::path(app, model)?,
    model.id,
    &samples,
    language::resolved(&language::current()),
  )?;
  crate::moments::add_transcript(&job.moments, job.moment, &transcript.text)
}
