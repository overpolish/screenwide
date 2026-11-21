// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures of the custom cursors a recording shows: the ones an app
//! draws itself rather than taking from the system, whose look the system's
//! artwork cannot stand in for. Each is kept once, as a PNG named for its
//! content in the folder beside the sidecar
//! (`screenwide_records::cursor::picture_path`), and the sidecar names it
//! where the cursor takes it on.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use screenwide_records::cursor::{picture_path, CursorRecord};

/// A custom cursor's picture, PNG-encoded, and the name the sidecar knows
/// it by.
#[derive(Clone, Debug)]
pub(crate) struct CursorPicture {
  pub name: String,
  pub png: Arc<[u8]>,
}

/// Two pictures are the same when their names are, which come from their
/// content; comparing the bytes on every pointer event would cost for
/// nothing.
impl PartialEq for CursorPicture {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

/// The pictures a stream has kept: written beside its sidecar as they come
/// for a recording, or held until a clip is written for the replay buffer.
pub(super) struct KeptPictures {
  sidecar: Option<PathBuf>,
  kept: HashMap<String, Arc<[u8]>>,
}

impl KeptPictures {
  /// Written beside the sidecar at `sidecar` as each first shows.
  pub(super) fn beside(sidecar: &Path) -> Self {
    Self {
      sidecar: Some(sidecar.to_owned()),
      kept: HashMap::new(),
    }
  }

  /// Held for clips written later.
  pub(super) fn held() -> Self {
    Self {
      sidecar: None,
      kept: HashMap::new(),
    }
  }

  pub(super) fn keep(&mut self, picture: &CursorPicture) -> Result<(), String> {
    if self.kept.contains_key(&picture.name) {
      return Ok(());
    }
    if let Some(sidecar) = &self.sidecar {
      write(sidecar, &picture.name, &picture.png)?;
    }
    self
      .kept
      .insert(picture.name.clone(), Arc::clone(&picture.png));
    Ok(())
  }

  /// Writes every picture `records` name beside the clip's sidecar at
  /// `sidecar`.
  pub(super) fn write_for(&self, records: &[CursorRecord], sidecar: &Path) -> Result<(), String> {
    for record in records {
      if let CursorRecord::Appearance {
        picture: Some(name),
        ..
      } = record
      {
        if let Some(png) = self.kept.get(name) {
          write(sidecar, name, png)?;
        }
      }
    }
    Ok(())
  }
}

fn write(sidecar: &Path, name: &str, png: &[u8]) -> Result<(), String> {
  let path = picture_path(sidecar, name).ok_or("A cursor picture's name is not one kept")?;
  if path.is_file() {
    return Ok(());
  }
  if let Some(folder) = path.parent() {
    std::fs::create_dir_all(folder)
      .map_err(|error| format!("The cursor pictures' folder could not be made: {error}"))?;
  }
  std::fs::write(&path, png)
    .map_err(|error| format!("A cursor's picture could not be kept: {error}"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_records::cursor::CursorStyle;

  fn picture(name: &str) -> CursorPicture {
    CursorPicture {
      name: name.to_owned(),
      png: Arc::from(name.as_bytes()),
    }
  }

  fn appearance(picture: Option<&str>) -> CursorRecord {
    CursorRecord::Appearance {
      height: 16.0,
      hotspot_x: 0.0,
      hotspot_y: 0.0,
      style: CursorStyle::Custom,
      timestamp_us: 0,
      width: 16.0,
      picture: picture.map(str::to_owned),
    }
  }

  fn folder(test: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
      "screenwide-cursor-pictures-{test}-{}",
      std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&folder);
    folder
  }

  #[test]
  fn a_recording_keeps_each_picture_beside_its_sidecar() {
    let sidecar = folder("recording").join("cursor.jsonl");
    let mut kept = KeptPictures::beside(&sidecar);
    kept.keep(&picture("ab12")).unwrap();
    kept.keep(&picture("ab12")).unwrap();
    let path = picture_path(&sidecar, "ab12").unwrap();
    assert_eq!(std::fs::read(path).unwrap(), b"ab12");
  }

  #[test]
  fn a_clip_takes_only_the_pictures_it_shows() {
    let sidecar = folder("clip").join("cursor.jsonl");
    let mut kept = KeptPictures::held();
    kept.keep(&picture("aa")).unwrap();
    kept.keep(&picture("bb")).unwrap();
    assert!(!picture_path(&sidecar, "aa").unwrap().exists());
    kept
      .write_for(&[appearance(None), appearance(Some("bb"))], &sidecar)
      .unwrap();
    assert!(!picture_path(&sidecar, "aa").unwrap().exists());
    assert_eq!(
      std::fs::read(picture_path(&sidecar, "bb").unwrap()).unwrap(),
      b"bb"
    );
  }
}
