// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each switch's bar, which is also its line in the work the models do
//! (`crate::editor::model_work`) and carries the flag that line's Cancel
//! sets; and the readying of Auto volume and Smart ducking, which every
//! switch ends with.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tauri::AppHandle;

use super::super::super::smart_ducking;
use super::super::super::step::Step;
use super::super::super::{ready_auto_volume, ready_ducking, SpeechSource};
use super::super::{reporter, MicrophoneTool};
use crate::editor::model_work::{Job, ModelWork};

/// A tool's progress bar. Once anything has been told, the bar is told it is
/// full when this goes, whether the work finished or failed, so it never
/// hangs part-way.
pub(super) struct Bar<F: FnMut(f32)> {
  report: F,
  told: bool,
  job: Option<Job>,
}

impl<F: FnMut(f32)> Step for Bar<F> {
  fn tell(&mut self, fraction: f32) {
    self.told = true;
    (self.report)(fraction);
    if let Some(job) = &self.job {
      job.tell(fraction);
    }
  }

  fn stop(&self) -> Option<Arc<AtomicBool>> {
    self.job.as_ref().map(Job::stop)
  }
}

impl<F: FnMut(f32)> Drop for Bar<F> {
  fn drop(&mut self) {
    if self.told {
      (self.report)(1.0);
    }
  }
}

pub(super) fn bar(app: &AppHandle, artifact_id: u64, tool: MicrophoneTool) -> Bar<impl FnMut(f32)> {
  let work = match tool {
    MicrophoneTool::AutoVolume => Some(ModelWork::AutoVolume),
    MicrophoneTool::Noise => Some(ModelWork::ReduceNoise),
    MicrophoneTool::RestoreVoice => Some(ModelWork::RestoreVoice),
    MicrophoneTool::SmartDucking => Some(ModelWork::SmartDucking),
    MicrophoneTool::Voice => Some(ModelWork::VocalCleanup),
    MicrophoneTool::Silences => None,
  };
  Bar {
    report: reporter(app, artifact_id, tool),
    told: false,
    job: work.map(|work| Job::begin(app, artifact_id, work)),
  }
}

/// Smart ducking's part in readying the recording `source` reads: its
/// separation model, where the switch is on and the model is downloaded,
/// and its own bar for making the system audio make way.
pub(super) struct Singing<F: FnMut(f32)> {
  model: Option<PathBuf>,
  bar: Bar<F>,
}

pub(super) fn singing(
  app: &AppHandle,
  artifact_id: u64,
  source: &SpeechSource,
) -> Singing<impl FnMut(f32)> {
  Singing {
    model: smart_ducking::wanted(app, &source.project_folder, source.microphone),
    bar: bar(app, artifact_id, MicrophoneTool::SmartDucking),
  }
}

/// Readies Auto volume and Smart ducking for the track as it is heard now,
/// where each is on: measures it, and makes the system audio make way for
/// it with the voice activity model at `model` and `singing`'s separation
/// model. Work that fails or is cancelled leaves the track heard unleveled
/// rather than undoing the switch that called for it.
pub(super) fn level_heard(
  source: &SpeechSource,
  model: Option<PathBuf>,
  singing: &mut Singing<impl FnMut(f32)>,
  level: &mut dyn Step,
) {
  if let Err(error) = ready(source, model, singing, level) {
    if !screenwide_speech::stop::is_stopped(&error) {
      eprintln!("Could not ready the microphone's volume: {error}");
    }
  }
}

pub(super) fn ready(
  source: &SpeechSource,
  model: Option<PathBuf>,
  singing: &mut Singing<impl FnMut(f32)>,
  level: &mut dyn Step,
) -> Result<(), String> {
  let recording = (source.movie.as_path(), source.duration_ms);
  ready_auto_volume(
    &source.project_folder,
    (source.microphone, source.system),
    recording,
    level,
  )?;
  ready_ducking(
    &source.project_folder,
    (source.microphone, source.system),
    recording,
    model.zip(singing.model.clone()),
    &mut singing.bar,
  )
}
