// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The work the models do on the recording open in the editor, listed in one
//! place for the island that shows it: each tool's work, how far it has got
//! and how long it has left, and a Cancel for each. Work is listed from the
//! first time it tells how far it has got until it ends. Cancel sets the
//! work's stop flag, which the work looks at as it goes, and takes it off
//! the list at once; turning the tool off, so the work is not asked for
//! again, is the editor's, as the switch would.
//!
//! Two pieces of work for the same tool at once, as a switch turned while
//! the recording is readied on opening can start, are one entry: they share
//! its flag and its line.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

/// The event that tells the editor what is under way.
const EVENT: &str = "editor://model-work";
/// How much of the work, and how long, is seen before the time left is
/// told: the first moments of most work are setup, slower or faster than
/// the rest.
const SETTLED_SHARE: f32 = 0.03;
const SETTLED_TIME: Duration = Duration::from_secs(5);
/// How often progress is told at most; work starting or ending is told at
/// once.
const TOLD_EVERY: Duration = Duration::from_millis(250);

/// Which tool's work it is.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ModelWork {
  AutoVolume,
  CameraObjects,
  Enhance,
  FollowFace,
  ReduceNoise,
  RemoveBackground,
  RestoreVoice,
  SmartDucking,
  Transcript,
  VocalCleanup,
}

/// One tool's work under way.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ModelJob {
  pub work: ModelWork,
  /// How far it has got, 0 to 1.
  pub fraction: f32,
  /// How long it has left, from how fast it has gone, once that has settled.
  pub seconds_left: Option<f32>,
}

/// The work under way on a recording, in the order it started.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ModelWorkList {
  pub artifact_id: u64,
  pub jobs: Vec<ModelJob>,
}

struct Entry {
  id: u64,
  artifact_id: u64,
  work: ModelWork,
  stop: Arc<AtomicBool>,
  /// The [`Job`]s for it still alive.
  holders: usize,
  told: Option<Told>,
}

impl Entry {
  fn is_cancelled(&self) -> bool {
    self.stop.load(Ordering::Acquire)
  }

  fn job(&self, now: Instant) -> Option<ModelJob> {
    let told = self.told.as_ref().filter(|_| !self.is_cancelled())?;
    let done = told.fraction - told.from;
    let elapsed = now.duration_since(told.began);
    let seconds_left = (done >= SETTLED_SHARE && elapsed >= SETTLED_TIME)
      .then(|| elapsed.as_secs_f32() * (1.0 - told.fraction).max(0.0) / done);
    Some(ModelJob {
      work: self.work,
      fraction: told.fraction,
      seconds_left,
    })
  }
}

/// How far the work has got, since it first told.
struct Told {
  began: Instant,
  from: f32,
  fraction: f32,
}

struct Registry {
  next_id: u64,
  entries: Vec<Entry>,
  last_told: Option<Instant>,
}

static REGISTRY: Mutex<Registry> = Mutex::new(Registry {
  next_id: 0,
  entries: Vec::new(),
  last_told: None,
});

fn registry() -> MutexGuard<'static, Registry> {
  REGISTRY
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Registry {
  fn list(&self, artifact_id: u64) -> ModelWorkList {
    let now = Instant::now();
    ModelWorkList {
      artifact_id,
      jobs: self
        .entries
        .iter()
        .filter(|entry| entry.artifact_id == artifact_id)
        .filter_map(|entry| entry.job(now))
        .collect(),
    }
  }

  /// The list for `artifact_id` to tell, unless it was told less than
  /// [`TOLD_EVERY`] ago and `at_once` is false.
  fn due(&mut self, artifact_id: u64, at_once: bool) -> Option<ModelWorkList> {
    let now = Instant::now();
    if !at_once
      && self
        .last_told
        .is_some_and(|last| now.duration_since(last) < TOLD_EVERY)
    {
      return None;
    }
    self.last_told = Some(now);
    Some(self.list(artifact_id))
  }
}

fn tell(app: &AppHandle, list: Option<ModelWorkList>) {
  if let Some(list) = list {
    let _ = app.emit(EVENT, list);
  }
}

/// A tool's work, listed once it tells how far it has got and until it is
/// dropped.
pub(crate) struct Job {
  app: AppHandle,
  id: u64,
  artifact_id: u64,
  stop: Arc<AtomicBool>,
}

impl Job {
  /// Joins `work` under way on the recording `artifact_id`, or starts it.
  pub(crate) fn begin(app: &AppHandle, artifact_id: u64, work: ModelWork) -> Self {
    let mut registry = registry();
    if let Some(entry) = registry
      .entries
      .iter_mut()
      .find(|entry| entry.artifact_id == artifact_id && entry.work == work && !entry.is_cancelled())
    {
      entry.holders += 1;
      return Self {
        app: app.clone(),
        id: entry.id,
        artifact_id,
        stop: Arc::clone(&entry.stop),
      };
    }
    registry.next_id += 1;
    let id = registry.next_id;
    let stop = Arc::new(AtomicBool::new(false));
    registry.entries.push(Entry {
      id,
      artifact_id,
      work,
      stop: Arc::clone(&stop),
      holders: 1,
      told: None,
    });
    Self {
      app: app.clone(),
      id,
      artifact_id,
      stop,
    }
  }

  /// Set once the work is cancelled.
  pub(crate) fn stop(&self) -> Arc<AtomicBool> {
    Arc::clone(&self.stop)
  }

  /// Tells how far the work has got, 0 to 1.
  pub(crate) fn tell(&self, fraction: f32) {
    let list = {
      let mut registry = registry();
      let Some(entry) = registry
        .entries
        .iter_mut()
        .find(|entry| entry.id == self.id)
        .filter(|entry| !entry.is_cancelled())
      else {
        return;
      };
      let first = entry.told.is_none();
      match &mut entry.told {
        Some(told) => told.fraction = fraction,
        None => {
          entry.told = Some(Told {
            began: Instant::now(),
            from: fraction,
            fraction,
          });
        }
      }
      registry.due(self.artifact_id, first)
    };
    tell(&self.app, list);
  }
}

impl Drop for Job {
  fn drop(&mut self) {
    let list = {
      let mut registry = registry();
      let Some(at) = registry
        .entries
        .iter()
        .position(|entry| entry.id == self.id)
      else {
        return;
      };
      let entry = &mut registry.entries[at];
      entry.holders -= 1;
      if entry.holders > 0 {
        return;
      }
      let shown = entry.told.is_some() && !entry.is_cancelled();
      registry.entries.remove(at);
      shown.then(|| registry.list(self.artifact_id))
    };
    tell(&self.app, list);
  }
}

/// The work under way on the recording `artifact_id`.
#[tauri::command]
pub fn get_model_work(artifact_id: u64) -> ModelWorkList {
  registry().list(artifact_id)
}

/// Stops `work` under way on the recording `artifact_id` and takes it off
/// the list. Turning its tool off is the editor's.
#[tauri::command]
pub fn cancel_model_work(app: AppHandle, artifact_id: u64, work: ModelWork) {
  let list = {
    let mut registry = registry();
    for entry in registry
      .entries
      .iter()
      .filter(|entry| entry.artifact_id == artifact_id && entry.work == work)
    {
      entry.stop.store(true, Ordering::Release);
    }
    registry.due(artifact_id, true)
  };
  tell(&app, list);
}
