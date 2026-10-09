// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use screenwide_transcriber::{Job, Reply, Request, Speech, Transcript, SAMPLE_RATE};

/// The transcriber program, started for a run of jobs and kept for the rest
/// of them, since each start loads the model again. Dropping it closes its
/// input, which is what ends it, so its memory is all given back.
pub(crate) struct Transcriber {
  child: Child,
  input: Option<ChildStdin>,
  output: BufReader<ChildStdout>,
  next_id: u64,
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
    })
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

  /// Sends `job` and waits for the reply that ends it, handing any progress
  /// on the way to `progress`.
  fn ask(&mut self, job: Job, progress: &mut dyn FnMut(f32)) -> Result<Reply, String> {
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
      let answer = serde_json::from_str::<Reply>(&reply).map_err(|error| error.to_string())?;
      let (Reply::Done { id, .. }
      | Reply::Speech { id, .. }
      | Reply::Cleaned { id, .. }
      | Reply::Restored { id }
      | Reply::Progress { id, .. }
      | Reply::Failed { id, .. }) = &answer;
      if *id != request.id {
        return Err("The transcriber answered a different request".to_owned());
      }
      match answer {
        Reply::Progress { fraction, .. } => progress(fraction),
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

/// Audio as the transcriber takes it, in a scratch file removed with this.
pub(crate) struct Samples(PathBuf);

impl Drop for Samples {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

impl Samples {
  /// A scratch file for the transcriber to write into.
  pub(crate) fn scratch() -> Self {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    Self(std::env::temp_dir().join(format!(
      "screenwide-transcription-{}-{}.f32",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    )))
  }

  pub(crate) fn path(&self) -> &Path {
    &self.0
  }

  /// The samples themselves, to look at before they are transcribed.
  pub(crate) fn read(&self) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(&self.0)
      .map_err(|error| format!("Could not read the audio to transcribe: {error}"))?;
    Ok(
      bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect(),
    )
  }
}

/// `source` converted by FFmpeg into what the transcriber reads: mono 32-bit
/// float samples at its rate, in little-endian order, which is the native
/// one on every computer Screenwide runs on.
pub(super) fn decode(source: &Path) -> Result<Samples, String> {
  decode_with(source, &[], SAMPLE_RATE)
}

/// The `stream`th audio track of `source`, converted as [`decode`] converts
/// a voice note but at `rate`, and padded or trimmed to start where the
/// recording starts so a moment in it is the same moment in the picture.
pub(crate) fn decode_audio_stream(
  source: &Path,
  stream: usize,
  rate: u32,
) -> Result<Samples, String> {
  decode_with(
    source,
    &[
      "-map",
      &format!("0:a:{stream}"),
      "-af",
      "aresample=async=1:first_pts=0",
    ],
    rate,
  )
}

fn decode_with(source: &Path, map: &[&str], rate: u32) -> Result<Samples, String> {
  let samples = Samples::scratch();
  let output = crate::editor::ffmpeg_command()
    .args(["-nostdin", "-loglevel", "error", "-y", "-i"])
    .arg(source)
    .args(map)
    .args(["-ac", "1", "-ar", &rate.to_string(), "-f", "f32le"])
    .arg(&samples.0)
    .output()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  if !output.status.success() {
    return Err(format!(
      "Could not read the audio to transcribe: {}",
      String::from_utf8_lossy(&output.stderr).trim()
    ));
  }
  Ok(samples)
}
