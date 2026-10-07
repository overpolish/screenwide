// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use screenwide_transcriber::{Reply, Request, Transcript, SAMPLE_RATE};

/// The transcriber program, started for a run of jobs and kept for the rest
/// of them, since each start loads the model again. Dropping it closes its
/// input, which is what ends it, so its memory is all given back.
pub(super) struct Transcriber {
  child: Child,
  input: Option<ChildStdin>,
  output: BufReader<ChildStdout>,
  next_id: u64,
}

impl Transcriber {
  pub(super) fn start() -> Result<Self, String> {
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
    self.next_id += 1;
    let request = Request {
      audio: audio.0.clone(),
      id: self.next_id,
      language,
      model,
      model_id: model_id.to_owned(),
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
    if self.output.read_line(&mut reply).map_err(|_| stopped())? == 0 {
      return Err(stopped());
    }
    match serde_json::from_str::<Reply>(&reply).map_err(|error| error.to_string())? {
      Reply::Done { id, transcript } if id == request.id => Ok(transcript),
      Reply::Failed { id, message } if id == request.id => Err(message),
      _ => Err("The transcriber answered a different request".to_owned()),
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
pub(super) struct Samples(PathBuf);

impl Drop for Samples {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

impl Samples {
  /// The samples themselves, to look at before they are transcribed.
  pub(super) fn read(&self) -> Result<Vec<f32>, String> {
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
  static NEXT: AtomicU64 = AtomicU64::new(0);
  let samples = Samples(std::env::temp_dir().join(format!(
    "screenwide-transcription-{}-{}.f32",
    std::process::id(),
    NEXT.fetch_add(1, Ordering::Relaxed)
  )));
  let output = crate::editor::ffmpeg_command()
    .args(["-nostdin", "-loglevel", "error", "-y", "-i"])
    .arg(source)
    .args(["-ac", "1", "-ar", &SAMPLE_RATE.to_string(), "-f", "f32le"])
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
