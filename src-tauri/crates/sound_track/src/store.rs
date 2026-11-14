// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sound library: every song or sound effect someone has added, kept
//! once in the app's data folder and named by a hash of the file it was
//! added from, with what was measured of it in a file of the same name
//! beside it. A sound effect is kept with its dead space cut (`trim`). A
//! project carries a copy of each sound its clips play (`project`).

mod adding;
mod kind;
mod measure;
pub mod music;
mod naming;
mod project;
mod removed;
pub mod singing;
mod sung;
mod trim;

pub use adding::{import, import_as, import_found};
pub use kind::{remember_kinds, set_kind};
pub use naming::{is_name, is_sound_file, EXTENSIONS};
pub use project::{adopt, carry, keep_from_project, prune, resolve};
pub use singing::Separated;
pub use sung::{is_sung, keep_heard_vocals};

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use naming::{content_hash, extension, is_stem};

/// Where a project keeps the sounds its clips play, beside its other media.
pub const PROJECT_FOLDER: &str = "media/audio";
/// The make of the file kept beside each sound.
const FORMAT_VERSION: u16 = 1;
/// How a sound is measured, raised whenever what is measured changes: a
/// sound measured before is measured again, once, when next read.
const MEASURED: u16 = 3;
/// The kind of file a sound effect cut down is kept as: lossless, so the
/// cut costs nothing of it.
const CUT_EXTENSION: &str = "flac";

static DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

/// A sound in the library, as measured when it was added.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Sound {
  /// Its name in the library: the hash of the file it was added from and
  /// the extension of what is kept, which is a sound effect cut down.
  pub name: String,
  /// Its title tag, or else the name of the file it was added from.
  pub title: String,
  pub duration_ms: u64,
  /// How loud it is over its whole length, in LUFS, which a song is levelled
  /// by; none where it is silent.
  pub loudness_lufs: Option<f64>,
  /// How loud it is at its loudest moment, in LUFS (EBU R128's momentary
  /// loudness over 400 ms), which a sound effect is levelled by, as that is
  /// how loud it is heard; none where it is silent.
  #[serde(default)]
  pub loudest_lufs: Option<f64>,
  /// How loud it is from 0 to 1, as root mean square, evenly along it, ten
  /// a second, which the clip's island draws its waveform from.
  #[serde(default)]
  pub levels: Vec<f32>,
  /// When it was added, in milliseconds since the Unix epoch.
  pub added_ms: u64,
  /// Its beat, which songs that touch are joined by, and whether it is
  /// sung, once heard.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub music: Option<music::Music>,
  /// How it was measured (`MEASURED`); 0 for a sound kept before that was
  /// told.
  #[serde(default)]
  #[ts(skip)]
  pub measured: u16,
  /// Who made it and under which licence, for a sound found on Openverse;
  /// none for one added from a file.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub credit: Option<screenwide_credits::Credit>,
  /// What Openverse says of it as music, for a song found there.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub tags: Option<music::MusicTags>,
  /// What it was last used as: added under it, or laid on its track
  /// (`kind`). A sound never placed has none, and its length says.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub kind: Option<crate::AudioClipKind>,
}

impl Sound {
  fn from_measured(
    name: String,
    title: String,
    added_ms: u64,
    measured: measure::Measured,
  ) -> Self {
    Self {
      name,
      title,
      duration_ms: measured.duration_ms,
      loudness_lufs: measured.loudness_lufs,
      loudest_lufs: measured.loudest_lufs,
      levels: measured.levels,
      added_ms,
      music: Some(measured.music),
      measured: MEASURED,
      credit: None,
      tags: None,
      kind: None,
    }
  }
}

#[derive(Deserialize, Serialize)]
struct Kept {
  version: u16,
  sound: Sound,
}

/// Settles where the library lives. Runs at launch, before any editor opens.
pub fn initialize(data: &Path) {
  let _ = DIRECTORY.set(data.join("sounds"));
}

fn directory() -> Result<&'static Path, String> {
  DIRECTORY
    .get()
    .map(PathBuf::as_path)
    .ok_or_else(|| "The sound library is not ready yet".to_owned())
}

/// The file kept beside the sound `name` in `folder`.
fn kept_path(folder: &Path, name: &str) -> PathBuf {
  let stem = name.split_once('.').map_or(name, |(stem, _)| stem);
  folder.join(format!("{stem}.json"))
}

fn read_kept(folder: &Path, name: &str) -> Option<Sound> {
  let bytes = std::fs::read(kept_path(folder, name)).ok()?;
  let kept: Kept = serde_json::from_slice(&bytes).ok()?;
  let file = folder.join(name);
  if kept.version != FORMAT_VERSION || kept.sound.name != name || !file.is_file() {
    return None;
  }
  // A sound measured before what is measured last changed is measured
  // again, once, and kept as measured now. Its length and name stay.
  if kept.sound.measured < MEASURED && kept.sound.duration_ms > 0 {
    if let Ok(measured) = measure::measure(&file) {
      let sound = Sound {
        credit: kept.sound.credit.clone(),
        tags: kept.sound.tags.clone(),
        kind: kept.sound.kind,
        ..Sound::from_measured(
          kept.sound.name.clone(),
          kept.sound.title.clone(),
          kept.sound.added_ms,
          measured,
        )
      };
      let _ = write_kept(folder, &sound);
      return Some(sound);
    }
  }
  Some(kept.sound)
}

/// Writes `bytes` to `path` whole: beside it first, then moved over it, so
/// work cut short never leaves a file that looks finished.
fn write_whole(path: &Path, bytes: &[u8]) -> Result<(), String> {
  let mut partial = path.as_os_str().to_owned();
  partial.push(".part");
  let partial = PathBuf::from(partial);
  std::fs::write(&partial, bytes)
    .and_then(|()| std::fs::rename(&partial, path))
    .map_err(|error| {
      let _ = std::fs::remove_file(&partial);
      format!("The sound could not be kept: {error}")
    })
}

fn write_kept(folder: &Path, sound: &Sound) -> Result<(), String> {
  let kept = Kept {
    version: FORMAT_VERSION,
    sound: sound.clone(),
  };
  let bytes = serde_json::to_vec(&kept).map_err(|error| error.to_string())?;
  write_whole(&kept_path(folder, &sound.name), &bytes)
}

/// Every sound in the library, the most recently added first.
pub fn sounds() -> Vec<Sound> {
  let Ok(folder) = directory() else {
    return Vec::new();
  };
  let Ok(entries) = std::fs::read_dir(folder) else {
    return Vec::new();
  };
  let mut sounds: Vec<Sound> = entries
    .flatten()
    .filter_map(|entry| {
      let name = entry.file_name().into_string().ok()?;
      is_name(&name).then(|| read_kept(folder, &name)).flatten()
    })
    .collect();
  sounds.sort_by(|a, b| b.added_ms.cmp(&a.added_ms).then(a.name.cmp(&b.name)));
  sounds
}

/// Takes the sound `name` out of the library. Projects that play it keep
/// their own copy, and do not hand it back (`removed`).
pub fn remove(name: &str) -> Result<(), String> {
  if !is_name(name) {
    return Err("There is no such sound".to_owned());
  }
  let folder = directory()?;
  let sound = folder.join(name);
  let [singing, rest, record] = singing::separated_files(&sound);
  for path in [singing, rest, record, kept_path(folder, name), sound] {
    match std::fs::remove_file(&path) {
      Ok(()) => {}
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(error) => return Err(format!("The sound could not be removed: {error}")),
    }
  }
  removed::note(folder, name);
  Ok(())
}

/// Gives the sound `name` the title `title`, and says what it is now.
pub fn rename(name: &str, title: &str) -> Result<Sound, String> {
  let title = title.trim();
  if title.is_empty() {
    return Err("A sound needs a title".to_owned());
  }
  let folder = directory()?;
  let sound = is_name(name)
    .then(|| read_kept(folder, name))
    .flatten()
    .ok_or_else(|| "There is no such sound".to_owned())?;
  let sound = Sound {
    title: title.to_owned(),
    ..sound
  };
  write_kept(folder, &sound)?;
  Ok(sound)
}

#[cfg(test)]
mod tests;
