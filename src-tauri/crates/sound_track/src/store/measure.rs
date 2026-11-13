// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the library keeps of a sound, read in one pass of FFmpeg: its
//! loudness as EBU R128 measures it, over its whole length and at its
//! loudest moment, its levels for the clip island's waveform, its length,
//! where what is heard of it starts and ends, its title tag, and its beat.

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
/// The stretches a sound is looked through for where it is heard, in
/// samples: 10 ms.
const HEARD_WINDOW: usize = READ_RATE / 100;
/// How far under its loudest stretch a stretch of a sound is still heard,
/// in dB, and how quiet one may be and still be heard at all, in dBFS:
/// below both, it is the dead space a recorded sound effect often starts
/// and ends with, a hiss at most.
const HEARD_UNDER_LOUDEST_DB: f32 = 50.0;
const QUIETEST_HEARD_DBFS: f32 = -70.0;

pub(super) struct Measured {
  pub duration_ms: u64,
  /// Its loudness over its whole length, and over its loudest 400 ms (EBU
  /// R128's momentary loudness), in LUFS.
  pub loudness_lufs: Option<f64>,
  pub loudest_lufs: Option<f64>,
  pub levels: Vec<f32>,
  /// Where what is heard of it starts and ends, in ms; none where nothing
  /// is.
  pub heard_ms: Option<(u64, u64)>,
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
      // Where `ebur128` logs its loudness ten times a second.
      "verbose",
      "-i",
    ])
    .arg(file)
    .args(["-map", "0:a:0", "-vn", "-af"])
    .arg(format!(
      "ebur128=framelog=verbose,aresample={READ_RATE},aformat=sample_fmts=flt:channel_layouts=mono"
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
    loudest_lufs: loudest(&said),
    levels,
    heard_ms: heard(&samples),
    title: title(&said),
    music: super::music::analyse(&samples, READ_RATE),
  })
}

/// The loudest momentary loudness among those `ebur128` logs ten times a
/// second, each of the 400 ms before, where any is above silence.
fn loudest(said: &str) -> Option<f64> {
  said
    .lines()
    .filter_map(|line| {
      let (_, after) = line.split_once("M:")?;
      after.split_whitespace().next()?.parse::<f64>().ok()
    })
    .filter(|lufs| *lufs > SILENT_LUFS)
    .reduce(f64::max)
}

/// Where what is heard of `samples` starts and ends, in ms, by the 10 ms
/// stretches loud enough to be heard.
fn heard(samples: &[f32]) -> Option<(u64, u64)> {
  let windows: Vec<f32> = samples
    .chunks(HEARD_WINDOW)
    .map(|window| {
      #[expect(clippy::cast_precision_loss, reason = "a window's length")]
      let power = window.iter().map(|sample| sample * sample).sum::<f32>() / window.len() as f32;
      10.0 * power.max(1e-12).log10()
    })
    .collect();
  let loudest = windows.iter().copied().reduce(f32::max)?;
  let floor = (loudest - HEARD_UNDER_LOUDEST_DB).max(QUIETEST_HEARD_DBFS);
  let first = windows.iter().position(|db| *db > floor)?;
  let last = windows.iter().rposition(|db| *db > floor)?;
  let window_ms = (HEARD_WINDOW * 1_000 / READ_RATE) as u64;
  Some((first as u64 * window_ms, (last as u64 + 1) * window_ms))
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

  #[test]
  fn the_loudest_moment_is_the_loudest_momentary_loudness_logged() {
    let said = "[Parsed_ebur128_0 @ 0x1] t: 0.1  TARGET:-23 LUFS    M:-120.7 S:-120.7     I: -70.0 LUFS\n\
                [Parsed_ebur128_0 @ 0x1] t: 0.2  TARGET:-23 LUFS    M: -12.7 S:-20.1     I: -13.7 LUFS\n\
                [Parsed_ebur128_0 @ 0x1] t: 0.3  TARGET:-23 LUFS    M: -15.2 S:-19.0     I: -13.7 LUFS\n";
    assert_eq!(loudest(said), Some(-12.7));
    assert_eq!(loudest("M:-120.7 S:-120.7\n"), None);
  }

  #[test]
  fn what_is_heard_starts_and_ends_where_it_rises_out_of_the_dead_space() {
    // A second of hiss far under it, half a second of tone, then two
    // seconds of digital silence.
    let mut samples = vec![0.000_01_f32; READ_RATE];
    #[expect(clippy::cast_precision_loss, reason = "a test signal")]
    samples.extend((0..READ_RATE / 2).map(|index| 0.5 * (index as f32 * 0.3).sin()));
    samples.extend(vec![0.0; 2 * READ_RATE]);
    assert_eq!(heard(&samples), Some((1_000, 1_500)));
    assert_eq!(heard(&vec![0.0; READ_RATE]), None);
  }
}
