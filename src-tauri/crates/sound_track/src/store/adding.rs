// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sound added to the library from a file: kept, measured and, where it
//! is a sound effect, cut down to what is heard of it (`trim`).

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use screenwide_credits::Credit;

use super::kind::set_kind;
use super::music::MusicTags;
use super::{
  content_hash, directory, extension, measure, read_kept, removed, trim, write_kept, write_whole,
  Sound, CUT_EXTENSION,
};
use crate::AudioClipKind;

/// Adds the sound in `file` to the library, measuring it, and says what it
/// is. A sound added before is found again by the file it came from, as it
/// was, or by the library's own file for it, which a sound dragged out of
/// the library brings back: a sound effect's is cut, so its contents no
/// longer match what it was added from. A sound effect is kept with its
/// dead space cut, as lossless FLAC; where cutting it fails, it is kept as
/// it came. One taken out of the library before is let back in.
pub fn import(file: &Path) -> Result<Sound, String> {
  let sound = add(file)?;
  removed::forget(directory()?, &sound.name);
  Ok(sound)
}

/// Adds the sound in `file` to the library as `import` does, listed as
/// `kind` where it was added under one, and says what it is.
pub fn import_as(file: &Path, kind: Option<AudioClipKind>) -> Result<Sound, String> {
  let sound = import(file)?;
  let Some(kind) = kind else {
    return Ok(sound);
  };
  set_kind(&sound.name, kind)?;
  Ok(Sound {
    kind: Some(kind),
    ..sound
  })
}

fn add(file: &Path) -> Result<Sound, String> {
  let folder = directory()?;
  if let Some(sound) = kept_from(folder, file) {
    return Ok(sound);
  }
  let extension = extension(file).ok_or_else(|| "That is not a sound file".to_owned())?;
  let bytes =
    std::fs::read(file).map_err(|error| format!("The sound could not be read: {error}"))?;
  let hash = content_hash(&bytes);
  let name = format!("{hash}.{extension}");
  let cut_name = format!("{hash}.{CUT_EXTENSION}");
  if let Some(sound) = [&name, &cut_name]
    .into_iter()
    .find_map(|each| read_kept(folder, each))
  {
    return Ok(sound);
  }
  std::fs::create_dir_all(folder)
    .map_err(|error| format!("The sound library could not be made: {error}"))?;
  let kept = folder.join(&name);
  if !kept.is_file() {
    write_whole(&kept, &bytes)?;
  }
  let measured = match measure::measure(&kept) {
    Ok(measured) => measured,
    Err(error) => {
      let _ = std::fs::remove_file(&kept);
      return Err(error);
    }
  };
  let title = measured.title.clone().unwrap_or_else(|| {
    file
      .file_stem()
      .and_then(|stem| stem.to_str())
      .unwrap_or(&name)
      .to_owned()
  });
  let (name, measured) = match trim::kept(measured.duration_ms, measured.heard_ms) {
    Some(stretch) => {
      cut_down(folder, &kept, &cut_name, stretch).map_or((name, measured), |cut| (cut_name, cut))
    }
    None => (name, measured),
  };
  let added_ms = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |since| since.as_millis() as u64);
  let sound = Sound::from_measured(name, title, added_ms, measured);
  write_kept(folder, &sound)?;
  Ok(sound)
}

/// Adds the sound in `file`, found on Openverse, to the library as `title`,
/// credited to who made it, with what Openverse says of it as music, and
/// says what it is. One the library already has, from a file, takes the
/// credit, title and tags; one already credited keeps its own.
pub fn import_found(
  file: &Path,
  title: &str,
  credit: Credit,
  tags: MusicTags,
) -> Result<Sound, String> {
  let sound = import(file)?;
  if sound.credit.is_some() {
    return Ok(sound);
  }
  let sound = Sound {
    title: title.to_owned(),
    credit: Some(credit),
    tags: Some(tags),
    ..sound
  };
  write_kept(directory()?, &sound)?;
  Ok(sound)
}

/// The sound `file` is, where it is one of the library's own files in
/// `folder`.
fn kept_from(folder: &Path, file: &Path) -> Option<Sound> {
  let same = |one: &Path, other: &Path| {
    one == other
      || std::fs::canonicalize(one)
        .ok()
        .is_some_and(|one| std::fs::canonicalize(other).ok() == Some(one))
  };
  if !same(file.parent()?, folder) {
    return None;
  }
  read_kept(folder, file.file_name()?.to_str()?)
}

/// The sound `kept` cut to `stretch` and kept as `cut_name` in `folder` in
/// its place, measured; none where cutting it failed, which leaves it as
/// it was.
fn cut_down(
  folder: &Path,
  kept: &Path,
  cut_name: &str,
  stretch: (u64, u64),
) -> Option<measure::Measured> {
  let partial = folder.join(format!("{cut_name}.part"));
  let cut = folder.join(cut_name);
  let measured = trim::cut(kept, &partial, stretch)
    .and_then(|()| measure::measure(&partial))
    .and_then(|measured| {
      std::fs::rename(&partial, &cut)
        .map(|()| measured)
        .map_err(|error| error.to_string())
    });
  let _ = std::fs::remove_file(&partial);
  let measured = measured.ok()?;
  if cut != kept {
    let _ = std::fs::remove_file(kept);
  }
  Some(measured)
}
