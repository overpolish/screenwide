// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A sound added to the library from a file: kept, measured and, where it
//! is a sound effect, cut down to what is heard of it (`trim`).

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
  content_hash, directory, extension, measure, read_kept, trim, write_kept, write_whole, Sound,
  CUT_EXTENSION,
};

/// Adds the sound in `file` to the library, measuring it, and says what it
/// is. A sound added before is found again by the file it came from, as it
/// was. A sound effect is kept with its dead space cut, as lossless FLAC;
/// where cutting it fails, it is kept as it came.
pub fn import(file: &Path) -> Result<Sound, String> {
  let extension = extension(file).ok_or_else(|| "That is not a sound file".to_owned())?;
  let bytes =
    std::fs::read(file).map_err(|error| format!("The sound could not be read: {error}"))?;
  let hash = content_hash(&bytes);
  let name = format!("{hash}.{extension}");
  let cut_name = format!("{hash}.{CUT_EXTENSION}");
  let folder = directory()?;
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
