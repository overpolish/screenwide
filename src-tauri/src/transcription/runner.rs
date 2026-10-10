// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use screenwide_transcriber::{Job, Reply, Request, Speech, Transcript};

use self::samples::Samples;

pub(crate) mod samples;

/// The transcriber program, started for a run of jobs and kept for the rest
/// of them, since each start loads the model again. Dropping it closes its
/// input, which is what ends it, so its memory is all given back.
pub(crate) struct Transcriber {
  child: Child,
  input: Option<ChildStdin>,
  output: BufReader<ChildStdout>,
  next_id: u64,
  /// Set to stop the job under way, which ends the program.
  stop: Option<Arc<AtomicBool>>,
}

impl Transcriber {
  pub(crate) fn start() -> Result<Self, String> {
    let mut child = crate::editor::transcriber_command()
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .map_err(|error| format!("Could not start the transcriber: {error}"))?;
    let input = child.stdin.take();
    let output = child
      .stdout
      .take()
      .map(BufReader::new)
      .ok_or_else(|| "The transcriber has no output".to_owned())?;
    Ok(Self {
      child,
      input,
      next_id: 0,
      output,
      stop: None,
    })
  }

  /// Ends the program, and with it the job under way, once `stop` is set.
  /// It is looked at as each reply arrives.
  pub(crate) fn stopped_by(mut self, stop: Arc<AtomicBool>) -> Self {
    self.stop = Some(stop);
    self
  }

  pub(super) fn transcribe(
    &mut self,
    model: PathBuf,
    model_id: &str,
    audio: &Samples,
    language: Option<String>,
  ) -> Result<Transcript, String> {
    match self.ask(
      Job::Transcribe {
        audio: audio.0.clone(),
        language,
        model,
        model_id: model_id.to_owned(),
      },
      &mut |_| {},
    )? {
      Reply::Done { transcript, .. } => Ok(transcript),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// How likely each window of `audio` is to be speech, by the voice
  /// activity model at `model`, telling `progress` how far it has got.
  pub(crate) fn detect_speech(
    &mut self,
    model: PathBuf,
    audio: &Samples,
    progress: &mut dyn FnMut(f32),
  ) -> Result<Speech, String> {
    match self.ask(
      Job::DetectSpeech {
        audio: audio.0.clone(),
        model,
      },
      progress,
    )? {
      Reply::Speech { speech, .. } => Ok(speech),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// `audio` with everything but the voice in it taken out, written to
  /// `output`, both at the protocol's cleaning rate, telling `progress` how
  /// far it has got. Answers with the gain that lifts the voice back to its
  /// recorded loudness.
  pub(crate) fn clean_speech(
    &mut self,
    audio: &Samples,
    output: &Samples,
    progress: &mut dyn FnMut(f32),
  ) -> Result<f32, String> {
    match self.ask(
      Job::CleanSpeech {
        audio: audio.0.clone(),
        output: output.0.clone(),
      },
      progress,
    )? {
      Reply::Cleaned { gain, .. } => Ok(gain),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// The `stretches` of `audio` rebuilt as a studio recording by the
  /// restoration model whose encoder is at `features` and vocoder at
  /// `decoder`, written to `output` at the protocol's cleaning rate with
  /// silence between them, telling `progress` how far it has got.
  pub(crate) fn restore_speech(
    &mut self,
    (features, decoder): (PathBuf, PathBuf),
    audio: &Samples,
    stretches: Vec<[u64; 2]>,
    output: &Samples,
    progress: &mut dyn FnMut(f32),
  ) -> Result<(), String> {
    match self.ask(
      Job::RestoreSpeech {
        audio: audio.0.clone(),
        decoder,
        features,
        output: output.0.clone(),
        stretches,
      },
      progress,
    )? {
      Reply::Restored { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// The singing in the stereo `audio`, at the protocol's separation rate,
  /// written to `output` in the same form by the separation model at
  /// `model`, telling `progress` how far it has got.
  pub(crate) fn separate_vocals(
    &mut self,
    model: PathBuf,
    audio: &Samples,
    output: &Samples,
    progress: &mut dyn FnMut(f32),
  ) -> Result<(), String> {
    match self.ask(
      Job::SeparateVocals {
        audio: audio.0.clone(),
        model,
        output: output.0.clone(),
      },
      progress,
    )? {
      Reply::Separated { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// Runs `job`, a [`Job::MatteCamera`], which writes the camera's matte and
  /// cutout into files of their own, telling `progress` how far it has got.
  pub(crate) fn matte_camera(
    &mut self,
    job: Job,
    progress: &mut dyn FnMut(f32),
  ) -> Result<(), String> {
    if !matches!(job, Job::MatteCamera { .. }) {
      return Err("Only a camera can be matted".to_owned());
    }
    match self.ask(job, progress)? {
      Reply::Matted { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// Runs `job`, a [`Job::FollowObjects`] or a [`Job::ComposeMatte`], which
  /// writes its files, telling `progress` how far it has got and, following
  /// objects, `halfway` once the masks after the click are written.
  pub(crate) fn camera_objects(
    &mut self,
    job: Job,
    progress: &mut dyn FnMut(f32),
    halfway: &mut dyn FnMut(),
  ) -> Result<(), String> {
    if !matches!(job, Job::FollowObjects { .. } | Job::ComposeMatte { .. }) {
      return Err("Only a camera's objects are asked for here".to_owned());
    }
    match self.ask_telling(job, progress, halfway)? {
      Reply::Tracked { .. } | Reply::Composed { .. } => Ok(()),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }

  /// Sends `job` and waits for the reply that ends it, handing any progress
  /// on the way to `progress`.
  fn ask(&mut self, job: Job, progress: &mut dyn FnMut(f32)) -> Result<Reply, String> {
    self.ask_telling(job, progress, &mut || {})
  }

  /// [`Self::ask`], also calling `halfway` where the job says it is.
  fn ask_telling(
    &mut self,
    job: Job,
    progress: &mut dyn FnMut(f32),
    halfway: &mut dyn FnMut(),
  ) -> Result<Reply, String> {
    self.next_id += 1;
    let request = Request {
      id: self.next_id,
      job,
    };
    let mut line = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
    line.push(b'\n');
    let stopped = || "The transcriber stopped unexpectedly".to_owned();
    let input = self.input.as_mut().ok_or_else(stopped)?;
    input
      .write_all(&line)
      .and_then(|()| input.flush())
      .map_err(|_| stopped())?;
    let mut reply = String::new();
    loop {
      reply.clear();
      if self.output.read_line(&mut reply).map_err(|_| stopped())? == 0 {
        return Err(stopped());
      }
      if self
        .stop
        .as_ref()
        .is_some_and(|stop| stop.load(Ordering::Acquire))
      {
        let _ = self.child.kill();
        return Err("The work was stopped".to_owned());
      }
      let answer = serde_json::from_str::<Reply>(&reply).map_err(|error| error.to_string())?;
      let (Reply::Done { id, .. }
      | Reply::Speech { id, .. }
      | Reply::Cleaned { id, .. }
      | Reply::Restored { id }
      | Reply::Separated { id }
      | Reply::Matted { id }
      | Reply::Tracked { id }
      | Reply::Composed { id }
      | Reply::Halfway { id }
      | Reply::Progress { id, .. }
      | Reply::Failed { id, .. }) = &answer;
      if *id != request.id {
        return Err("The transcriber answered a different request".to_owned());
      }
      match answer {
        Reply::Progress { fraction, .. } => progress(fraction),
        Reply::Halfway { .. } => halfway(),
        Reply::Failed { message, .. } => return Err(message),
        answer => return Ok(answer),
      }
    }
  }
}

impl Drop for Transcriber {
  fn drop(&mut self) {
    self.input = None;
    let _ = self.child.wait();
  }
}
