// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The store's pictures as a library: what an image showing each carries,
//! the name of the file it came from where it came from one, and when it
//! arrived, kept in a small file beside it. A picture kept before these
//! were is described from its file the first time it is listed: its kept
//! size, and the time its file was last written.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::super::animation::decode;
use super::{directory, is_name, removed, stored_file, IMAGE_PREFIX, KEPT_EXTENSIONS};
use screenwide_annotations::image::{ImageArt, ImagePlay};
use screenwide_credits::Credit;

/// The version of the file kept beside a picture.
const FORMAT_VERSION: u16 = 1;

/// A picture in the library.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Picture {
  /// Its name in the store.
  pub name: String,
  /// The name of the file it was added from, without its extension; none
  /// for one pasted or dropped as a picture rather than a file, or kept
  /// before names were.
  pub title: Option<String>,
  /// When it arrived, in ms since the Unix epoch.
  #[ts(type = "number")]
  pub added_ms: u64,
  /// What an image showing it carries.
  pub art: ImageArt,
  /// The file it is kept in, as the library reads it to show it; filled
  /// in as it is listed, since the store's folder can move.
  #[serde(default)]
  pub file: String,
  /// Who made it and under which licence, for a picture found on
  /// Openverse; none for one given an image otherwise.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub credit: Option<Credit>,
}

#[derive(Deserialize, Serialize)]
struct Kept {
  version: u16,
  picture: Picture,
}

pub(super) fn kept_path(folder: &Path, name: &str) -> std::path::PathBuf {
  folder.join(format!("{name}.json"))
}

fn now_ms() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |since| since.as_millis() as u64)
}

pub(super) fn read_kept(folder: &Path, name: &str) -> Option<Picture> {
  let bytes = std::fs::read(kept_path(folder, name)).ok()?;
  let kept: Kept = serde_json::from_slice(&bytes).ok()?;
  (kept.version == FORMAT_VERSION && kept.picture.name == name).then_some(kept.picture)
}

fn write_kept(folder: &Path, picture: &Picture) -> Result<(), String> {
  let kept = Kept {
    version: FORMAT_VERSION,
    picture: picture.clone(),
  };
  let bytes = serde_json::to_vec(&kept).map_err(|error| error.to_string())?;
  let path = kept_path(folder, &picture.name);
  let partial = folder.join(format!("{}.json.tmp", picture.name));
  std::fs::write(&partial, bytes)
    .and_then(|()| std::fs::rename(&partial, &path))
    .map_err(|error| format!("The picture could not be described: {error}"))
}

/// The store's name for the picture `art` shows, if it is one of the
/// store's.
fn name_of(art: &ImageArt) -> Option<&str> {
  art
    .asset
    .strip_prefix(IMAGE_PREFIX)
    .filter(|name| is_name(name))
}

/// Notes the picture `art` shows as arrived now, and, where it came from a
/// file, by that file's name. One already noted keeps when it arrived, and
/// gains a name if it had none.
pub(super) fn note(art: &ImageArt, title: Option<&str>) {
  let (Some(folder), Some(name)) = (directory(), name_of(art)) else {
    return;
  };
  removed::forget(folder, name);
  let picture = match read_kept(folder, name) {
    Some(known) if known.title.is_some() || title.is_none() => return,
    Some(known) => Picture {
      title: title.map(str::to_owned),
      ..known
    },
    None => Picture {
      name: name.to_owned(),
      title: title.map(str::to_owned),
      added_ms: now_ms(),
      art: art.clone(),
      file: String::new(),
      credit: None,
    },
  };
  if let Err(error) = write_kept(folder, &picture) {
    eprintln!("Could not note the picture {name}: {error}");
  }
}

/// Names the picture `art` shows, found on Openverse, `title`, and credits
/// it to who made it, unless it is credited already.
pub(super) fn credit_found(art: &ImageArt, title: &str, credit: Credit) {
  let (Some(folder), Some(name)) = (directory(), name_of(art)) else {
    return;
  };
  let Some(picture) = picture(folder, name) else {
    return;
  };
  if picture.credit.is_some() {
    return;
  }
  let credited = Picture {
    title: Some(title.to_owned()),
    credit: Some(credit),
    ..picture
  };
  if let Err(error) = write_kept(folder, &credited) {
    eprintln!("Could not credit the picture {name}: {error}");
  }
}

/// The picture kept in `file`, described from the file itself.
fn described(name: &str, file: &Path) -> Option<Picture> {
  let bytes = std::fs::read(file).ok()?;
  let art = match decode(&bytes) {
    Some(animation) => {
      let (width, height) = animation.canvas;
      ImageArt {
        asset: format!("{IMAGE_PREFIX}{name}"),
        aspect: f64::from(width) / f64::from(height.max(1)),
        pixels: Some(f64::from(width.max(height))),
        play: Some(ImagePlay {
          cycle_ms: f64::from(animation.cycle_ms),
          frames: animation.frames.len() as u32,
          frame: 0,
          once: false,
        }),
      }
    }
    None => {
      let (width, height) = image::load_from_memory(&bytes)
        .ok()?
        .into_rgba8()
        .dimensions();
      ImageArt {
        asset: format!("{IMAGE_PREFIX}{name}"),
        aspect: f64::from(width) / f64::from(height.max(1)),
        pixels: Some(f64::from(width.max(height))),
        play: None,
      }
    }
  };
  let added_ms = std::fs::metadata(file)
    .and_then(|metadata| metadata.modified())
    .ok()
    .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
    .map_or(0, |since| since.as_millis() as u64);
  Some(Picture {
    name: name.to_owned(),
    title: None,
    added_ms,
    art,
    file: String::new(),
    credit: None,
  })
}

/// The picture `name` as the library has it, described and noted from its
/// file where it was kept before pictures were noted.
fn picture(folder: &Path, name: &str) -> Option<Picture> {
  let file = stored_file(name)?;
  let picture = match read_kept(folder, name) {
    Some(picture) => picture,
    None => {
      let picture = described(name, &file)?;
      let _ = write_kept(folder, &picture);
      picture
    }
  };
  Some(Picture {
    file: file.to_string_lossy().into_owned(),
    ..picture
  })
}

/// Every picture in the library, the most recently added first.
pub fn pictures() -> Vec<Picture> {
  let Some(folder) = directory() else {
    return Vec::new();
  };
  let Ok(entries) = std::fs::read_dir(folder) else {
    return Vec::new();
  };
  let hidden = removed::names(folder);
  let mut pictures: Vec<Picture> = entries
    .flatten()
    .filter_map(|entry| {
      let path = entry.path();
      let extension = path.extension()?.to_str()?;
      let name = path.file_stem()?.to_str()?;
      (KEPT_EXTENSIONS.contains(&extension) && is_name(name) && !hidden.contains(name))
        .then(|| picture(folder, name))
        .flatten()
    })
    .collect();
  pictures.sort_by(|a, b| b.added_ms.cmp(&a.added_ms).then(a.name.cmp(&b.name)));
  pictures
}

/// What an image showing the picture in `file` carries, where `file` is one
/// of the store's own: a picture dragged out of the library and back in is
/// the same picture, not one named by a copy of it.
pub(super) fn kept_art(file: &Path) -> Option<ImageArt> {
  let folder = directory()?;
  let same = |one: &Path, other: &Path| {
    one == other
      || std::fs::canonicalize(one)
        .ok()
        .is_some_and(|one| std::fs::canonicalize(other).ok() == Some(one))
  };
  if !same(file.parent()?, folder) {
    return None;
  }
  let name = file.file_stem()?.to_str()?;
  is_name(name)
    .then(|| picture(folder, name))
    .flatten()
    .map(|picture| picture.art)
}

/// Takes the picture `name` out of the library. Projects that show it keep
/// their own copy, which they hand back when they are opened again, to draw
/// it, but it stays out of the library (`removed`).
pub fn remove(name: &str) -> Result<(), String> {
  if !is_name(name) {
    return Err("There is no such picture".to_owned());
  }
  let folder = directory().ok_or("Pictures cannot be kept yet")?;
  removed::note(folder, name);
  if let Some(file) = stored_file(name) {
    std::fs::remove_file(file)
      .map_err(|error| format!("The picture could not be removed: {error}"))?;
  }
  let _ = std::fs::remove_file(kept_path(folder, name));
  Ok(())
}

/// Gives the picture `name` the name `title`, in place of the name of the
/// file it came from, or of none.
pub fn rename(name: &str, title: &str) -> Result<(), String> {
  let title = title.trim();
  if title.is_empty() {
    return Err("A picture needs a name".to_owned());
  }
  let folder = directory().ok_or("Pictures cannot be kept yet")?;
  let picture = is_name(name)
    .then(|| picture(folder, name))
    .flatten()
    .ok_or_else(|| "There is no such picture".to_owned())?;
  write_kept(
    folder,
    &Picture {
      title: Some(title.to_owned()),
      ..picture
    },
  )
}

#[cfg(test)]
mod tests;
