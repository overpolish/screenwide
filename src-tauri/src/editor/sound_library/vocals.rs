// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether each song in the library is sung, so the Library island can
//! narrow songs to the instrumental or the sung: heard in the background,
//! one song at a time, by separating a few pieces of it with Studio
//! ducking's model, once that is on this computer, and kept with the song.
//! The pieces are the ones the singing is separated in anyway, so a song
//! later played instrumental already has them. A song found on Openverse
//! says whether it is sung, and is not heard.

use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use screenwide_sound_track::clip::SONG_MS;
use screenwide_sound_track::store::music::Vocals;
use screenwide_sound_track::store::singing::PIECE_MS;
use screenwide_sound_track::store::{self, Sound};
use screenwide_sound_track::AudioClipKind;
use tauri::{AppHandle, Emitter};

use super::singing::separate_pieces;
use super::LIBRARY_EVENT;

/// Where in a song the pieces heard fall, as shares of its length: away
/// from an intro or outro without singing, spread so a song sung only in
/// its verses is still caught.
const HEARD_AT: [f64; 3] = [0.3, 0.5, 0.7];

/// Whether a song is being heard now.
static HEARING: AtomicBool = AtomicBool::new(false);
/// Songs that could not be heard, by name, not tried again until the app
/// starts again.
static FAILED: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Whether `sound` is a song whose singing is not known.
fn is_unheard(sound: &Sound) -> bool {
  let is_song = sound.kind.map_or(sound.duration_ms >= SONG_MS, |kind| {
    kind == AudioClipKind::Music
  });
  let is_known = sound
    .tags
    .as_ref()
    .is_some_and(|tags| tags.vocals.is_some())
    || sound.music.is_some_and(|music| music.vocals.is_some());
  is_song && !is_known && !has_failed(&sound.name)
}

fn has_failed(name: &str) -> bool {
  FAILED
    .lock()
    .ok()
    .is_some_and(|failed| failed.as_ref().is_some_and(|failed| failed.contains(name)))
}

/// The pieces of a song `duration_ms` long that are heard.
fn heard_pieces(duration_ms: u64) -> BTreeSet<u64> {
  let last = duration_ms.saturating_sub(1) / PIECE_MS;
  HEARD_AT
    .iter()
    .map(|share| {
      #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a piece of a song"
      )]
      let piece = (duration_ms as f64 * share) as u64 / PIECE_MS;
      piece.min(last)
    })
    .collect()
}

/// Hears, in the background, each of `sounds` that is a song whose singing
/// is not known, once the separation model is on this computer, and tells
/// every window as each is heard. Asked again while it works, it carries on
/// with whatever the library holds then.
pub(super) fn hear(app: &AppHandle, sounds: &[Sound]) {
  if !sounds.iter().any(is_unheard) {
    return;
  }
  let Some(model) = crate::editor::speech::studio_ducking::model(app) else {
    return;
  };
  if HEARING.swap(true, Ordering::AcqRel) {
    return;
  }
  let app = app.clone();
  std::thread::spawn(move || {
    while let Some(sound) = store::sounds().into_iter().find(is_unheard) {
      match hear_one(&sound, model.clone()) {
        Ok(vocals) => {
          if let Err(error) = store::keep_heard_vocals(&sound.name, vocals) {
            eprintln!("Could not keep whether {} is sung: {error}", sound.name);
            fail(&sound.name);
          }
          let _ = app.emit(LIBRARY_EVENT, ());
        }
        Err(error) => {
          eprintln!("Could not hear whether {} is sung: {error}", sound.name);
          fail(&sound.name);
        }
      }
    }
    HEARING.store(false, Ordering::Release);
  });
}

fn hear_one(sound: &Sound, model: PathBuf) -> Result<Vocals, String> {
  let (path, _) =
    store::resolve(None, &sound.name).ok_or_else(|| "It is no longer in the library".to_owned())?;
  let pieces = heard_pieces(sound.duration_ms);
  separate_pieces(&path, &pieces, model)?;
  Ok(if store::is_sung(&path, &pieces)? {
    Vocals::Vocal
  } else {
    Vocals::Instrumental
  })
}

fn fail(name: &str) {
  if let Ok(mut failed) = FAILED.lock() {
    failed.get_or_insert_default().insert(name.to_owned());
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_song_is_heard_through_its_middle_and_a_short_one_where_it_can_be() {
    assert_eq!(heard_pieces(230_000), BTreeSet::from([8, 14, 20]));
    assert_eq!(heard_pieces(12_000), BTreeSet::from([0, 1]));
  }
}
