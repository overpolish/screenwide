// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The only audio notes keep: a few seconds of each voice, beside the notes,
//! so whoever took them can hear who a speaker is and name them. A clip is
//! deleted once its speaker is named, when no turn in the notes is labelled
//! with the number it was saved under any more, and after a week whether
//! named or not. Each is a WAV file named after its speaker's number.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use screenwide_speech::runner::notes::VoiceClip;

/// The folder in a notes project the clips are kept in.
const FOLDER: &str = "voices";
/// How long a clip is kept at most.
const KEPT: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const RATE: u32 = 16_000;

/// Saves the clip of each voice in `clips` that has a speaker number in the
/// notes in `root`, `voices` being each number's voice from 1.
pub fn save(root: &Path, clips: &[VoiceClip], voices: &[u32]) -> std::io::Result<()> {
  let folder = root.join(FOLDER);
  for clip in clips {
    let Some(at) = voices.iter().position(|&voice| voice == clip.speaker) else {
      continue;
    };
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join(file_name(at as u32 + 1)), wav(&clip.samples))?;
  }
  Ok(())
}

/// The speaker numbers in the notes in `root` that still have a clip, in
/// order.
pub fn kept(root: &Path) -> Vec<u32> {
  let mut numbers: Vec<u32> = clips(root).into_iter().map(|(number, _)| number).collect();
  numbers.sort_unstable();
  numbers
}

/// The clip of speaker `number` in the notes in `root`, as a WAV file.
pub fn read(root: &Path, number: u32) -> std::io::Result<Vec<u8>> {
  std::fs::read(root.join(FOLDER).join(file_name(number)))
}

/// Deletes the clips of the speakers the `markdown` of the notes in `root`
/// no longer labels by their number: those named since.
pub fn forget_named(root: &Path, markdown: &str) {
  for (number, path) in clips(root) {
    if !markdown.contains(&format!("**Speaker {number}:**")) {
      let _ = std::fs::remove_file(path);
    }
  }
  let _ = std::fs::remove_dir(root.join(FOLDER));
}

/// Deletes the clips kept longer than a week in each project in `projects`.
pub fn expire(projects: &Path) {
  let Ok(entries) = std::fs::read_dir(projects) else {
    return;
  };
  let now = SystemTime::now();
  for root in entries.flatten().map(|entry| entry.path()) {
    for (_, path) in clips(&root) {
      let made = std::fs::metadata(&path).and_then(|metadata| metadata.modified());
      if made.is_ok_and(|made| now.duration_since(made).unwrap_or_default() > KEPT) {
        let _ = std::fs::remove_file(path);
      }
    }
    // Only an empty folder goes: one still holding clips stays.
    let _ = std::fs::remove_dir(root.join(FOLDER));
  }
}

fn file_name(number: u32) -> String {
  format!("speaker-{number}.wav")
}

/// Each clip in the notes in `root`, with its speaker's number.
fn clips(root: &Path) -> Vec<(u32, PathBuf)> {
  let Ok(entries) = std::fs::read_dir(root.join(FOLDER)) else {
    return Vec::new();
  };
  entries
    .flatten()
    .filter_map(|entry| {
      let name = entry.file_name();
      let number = name
        .to_str()?
        .strip_prefix("speaker-")?
        .strip_suffix(".wav")?
        .parse()
        .ok()?;
      Some((number, entry.path()))
    })
    .collect()
}

/// `samples` as a mono 16-bit WAV file at the speech models' rate.
fn wav(samples: &[i16]) -> Vec<u8> {
  let data = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
  let mut out = Vec::with_capacity(44 + samples.len() * 2);
  out.extend_from_slice(b"RIFF");
  out.extend_from_slice(&(36 + data).to_le_bytes());
  out.extend_from_slice(b"WAVEfmt ");
  out.extend_from_slice(&16u32.to_le_bytes());
  out.extend_from_slice(&1u16.to_le_bytes()); // PCM
  out.extend_from_slice(&1u16.to_le_bytes()); // mono
  out.extend_from_slice(&RATE.to_le_bytes());
  out.extend_from_slice(&(RATE * 2).to_le_bytes());
  out.extend_from_slice(&2u16.to_le_bytes());
  out.extend_from_slice(&16u16.to_le_bytes());
  out.extend_from_slice(b"data");
  out.extend_from_slice(&data.to_le_bytes());
  for sample in samples {
    out.extend_from_slice(&sample.to_le_bytes());
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_clip_is_kept_under_its_speaker_number_until_they_are_named() {
    let root = std::env::temp_dir().join(format!("screenwide-clips-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let clip = |speaker| VoiceClip {
      speaker,
      samples: vec![1; 16],
    };
    // The speaker model's voices 5 and 2 are speakers 1 and 2 in the notes;
    // voice 7 said nothing written down.
    save(&root, &[clip(2), clip(5), clip(7)], &[5, 2]).unwrap();
    assert_eq!(kept(&root), [1, 2]);
    assert_eq!(read(&root, 2).unwrap().len(), 44 + 32);
    forget_named(
      &root,
      "# Notes\n\n**Kevin:** Hi.\n\n**Speaker 2:** Hello.\n",
    );
    assert_eq!(kept(&root), [2]);
    forget_named(&root, "# Notes\n\n**Kevin:** Hi.\n\n**Dom:** Hello.\n");
    assert!(kept(&root).is_empty());
    assert!(!root.join(FOLDER).exists());
    std::fs::remove_dir_all(&root).unwrap();
  }
}
