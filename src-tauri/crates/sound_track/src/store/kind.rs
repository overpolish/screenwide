// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether the library lists a sound as a song or a sound effect: what it
//! was last used as, added under that kind or laid on that track, so an
//! 11-second bell used as an effect stays with the effects.

use crate::AudioClipKind;

use super::{directory, is_name, read_kept, write_kept};

/// Notes that the sound `name` was used as `kind`, and says whether that
/// changed what the library lists it as.
pub fn set_kind(name: &str, kind: AudioClipKind) -> Result<bool, String> {
  let folder = directory()?;
  let Some(sound) = is_name(name).then(|| read_kept(folder, name)).flatten() else {
    return Ok(false);
  };
  if sound.kind == Some(kind) {
    return Ok(false);
  }
  write_kept(
    folder,
    &super::Sound {
      kind: Some(kind),
      ..sound
    },
  )?;
  Ok(true)
}

/// Notes what each of `placed`, a sound laid on a track, was used as, and
/// says whether the library lists any of them differently now.
pub fn remember_kinds<'a>(placed: impl IntoIterator<Item = (&'a str, AudioClipKind)>) -> bool {
  placed
    .into_iter()
    .fold(false, |changed, (name, kind)| match set_kind(name, kind) {
      Ok(now) => changed || now,
      Err(error) => {
        eprintln!("Could not note what {name} is used as: {error}");
        changed
      }
    })
}
