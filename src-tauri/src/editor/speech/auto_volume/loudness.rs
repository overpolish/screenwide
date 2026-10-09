// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How loud a track is, as EBU R128 measures it: FFmpeg's `ebur128` filter,
//! run through the same filters the preview and export play it through, so
//! the measure is of exactly what is heard.

use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::Stdio;

/// The integrated loudness of audio stream `stream` of `source`, in LUFS,
/// through `filters` first when there are any, telling `progress` how far
/// through the `duration_ms` long track it has got.
pub(super) fn integrated(
  source: &Path,
  stream: usize,
  filters: Option<&str>,
  duration_ms: u64,
  progress: &mut dyn FnMut(f32),
) -> Result<f64, String> {
  let chain = filters.map_or_else(
    || "ebur128=framelog=quiet".to_owned(),
    |filters| format!("{filters},ebur128=framelog=quiet"),
  );
  let mut child = crate::editor::ffmpeg_command()
    .args(["-hide_banner", "-nostdin", "-nostats", "-loglevel", "info"])
    .args(["-progress", "pipe:1", "-i"])
    .arg(source)
    .args(["-map", &format!("0:a:{stream}"), "-vn", "-af", &chain])
    .args(["-f", "null", "-"])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  // Read beside the progress, so neither pipe fills while the other is read.
  let mut stderr = child
    .stderr
    .take()
    .ok_or_else(|| "FFmpeg did not say how loud the microphone is".to_owned())?;
  let said = std::thread::spawn(move || {
    let mut text = String::new();
    let _ = stderr.read_to_string(&mut text);
    text
  });
  if let Some(stdout) = child.stdout.take() {
    let total_us = duration_ms.max(1) as f64 * 1_000.0;
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
      if let Some(us) = line
        .strip_prefix("out_time_us=")
        .and_then(|value| value.parse::<f64>().ok())
      {
        progress((us / total_us).clamp(0.0, 1.0) as f32);
      }
    }
  }
  let status = child
    .wait()
    .map_err(|error| format!("FFmpeg did not finish: {error}"))?;
  let said = said.join().unwrap_or_default();
  if !status.success() {
    return Err(format!(
      "Could not measure the microphone's volume: {}",
      said.lines().last().unwrap_or_default()
    ));
  }
  summary(&said).ok_or_else(|| "FFmpeg did not say how loud the microphone is".to_owned())
}

/// The integrated loudness in the summary `ebur128` logs when it ends: its
/// last line of the form `I: -23.0 LUFS`.
fn summary(said: &str) -> Option<f64> {
  said.lines().rev().find_map(|line| {
    line
      .trim()
      .strip_prefix("I:")?
      .trim()
      .strip_suffix("LUFS")?
      .trim()
      .parse()
      .ok()
  })
}
