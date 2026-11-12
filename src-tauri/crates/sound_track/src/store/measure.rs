// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the library keeps of a sound, read in one pass of FFmpeg: its
//! loudness as EBU R128 measures it, its levels for the clip island's
//! waveform, its length, its title tag, and its beat and key.

use std::io::Read;
use std::path::Path;
use std::process::Stdio;

/// The rate the sound is read at for its levels and length, in Hz.
const READ_RATE: usize = 8_000;
/// How many levels a second of sound keeps.
const LEVELS_PER_SECOND: usize = 10;
/// A sound this quiet or quieter, in LUFS, holds nothing to level: the
/// absolute gate of EBU R128.
const SILENT_LUFS: f64 = -70.0;

pub(super) struct Measured {
  pub duration_ms: u64,
  pub loudness_lufs: Option<f64>,
  pub levels: Vec<f32>,
  pub title: Option<String>,
  pub music: super::music::Music,
}

pub(super) fn measure(file: &Path) -> Result<Measured, String> {
  let mut child = screenwide_sidecar::ffmpeg_command()
    .args([
      "-hide_banner",
      "-nostdin",
      "-nostats",
      "-loglevel",
      "info",
      "-i",
    ])
    .arg(file)
    .args(["-map", "0:a:0", "-vn", "-af"])
    .arg(format!(
      "ebur128=framelog=quiet,aresample={READ_RATE},aformat=sample_fmts=flt:channel_layouts=mono"
    ))
    .args(["-f", "f32le", "pipe:1"])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  // Read beside the samples, so neither pipe fills while the other is read.
  let mut stderr = child
    .stderr
    .take()
    .ok_or_else(|| "FFmpeg did not say what the sound is".to_owned())?;
  let said = std::thread::spawn(move || {
    let mut text = String::new();
    let _ = stderr.read_to_string(&mut text);
    text
  });
  // Each level is the root mean square of its stretch rather than its
  // peak: music is mastered close to full scale throughout, so its peaks
  // barely change, where how loud it is does.
  let per_level = READ_RATE / LEVELS_PER_SECOND;
  let mut levels = Vec::new();
  let mut squares = 0.0_f32;
  let mut in_level = 0;
  let mut samples = Vec::new();
  if let Some(mut stdout) = child.stdout.take() {
    let mut bytes = vec![0_u8; 64 * 1_024];
    let mut carried = Vec::with_capacity(4);
    loop {
      let count = match stdout.read(&mut bytes) {
        Ok(0) | Err(_) => break,
        Ok(count) => count,
      };
      carried.extend_from_slice(&bytes[..count]);
      let whole = carried.len() - carried.len() % 4;
      for chunk in carried[..whole].as_chunks::<4>().0 {
        let sample = f32::from_le_bytes(*chunk);
        squares += sample.powi(2);
        in_level += 1;
        samples.push(sample);
        if in_level == per_level {
          levels.push(rounded((squares / in_level as f32).sqrt()));
          squares = 0.0;
          in_level = 0;
        }
      }
      carried.drain(..whole);
    }
  }
  if in_level > 0 {
    levels.push(rounded((squares / in_level as f32).sqrt()));
  }
  let status = child
    .wait()
    .map_err(|error| format!("FFmpeg did not finish: {error}"))?;
  let said = said.join().unwrap_or_default();
  if !status.success() || samples.is_empty() {
    return Err("That file holds no sound Screenwide can play".to_owned());
  }
  Ok(Measured {
    duration_ms: (samples.len() as u64 * 1_000) / READ_RATE as u64,
    loudness_lufs: loudness(&said).filter(|lufs| *lufs > SILENT_LUFS),
    levels,
    title: title(&said),
    music: super::music::analyse(&samples, READ_RATE),
  })
}

/// A level kept to three places, which is all a waveform shows, so the
/// file kept beside the sound stays small.
fn rounded(level: f32) -> f32 {
  (level.min(1.0) * 1_000.0).round() / 1_000.0
}

/// The integrated loudness in the summary `ebur128` logs when it ends: its
/// last line of the form `I: -23.0 LUFS`.
fn loudness(said: &str) -> Option<f64> {
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

/// The first title tag FFmpeg lists of the input, if it has one.
fn title(said: &str) -> Option<String> {
  said.lines().find_map(|line| {
    let (key, value) = line.split_once(':')?;
    (key.trim() == "title")
      .then(|| value.trim().to_owned())
      .filter(|value| !value.is_empty())
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reads_the_title_and_loudness_ffmpeg_logs() {
    let said = "Input #0, mp3, from 'a.mp3':\n  Metadata:\n    title           : Morning: Light\n  Duration: 00:00:06.03\n[Parsed_ebur128_0 @ 0x1]   Integrated loudness:\n    I:         -20.4 LUFS\n";
    assert_eq!(title(said).as_deref(), Some("Morning: Light"));
    assert_eq!(loudness(said), Some(-20.4));
  }
}
