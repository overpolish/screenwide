// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The singing separated from the songs that make way for the voice, so it
//! can get out of the way entirely while the rest of the song only comes
//! down.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use screenwide_sound_track::store;
use screenwide_sound_track::{AudioClipKind, RecordingAudioClip};
use tauri::AppHandle;

use super::project_sounds;

/// The songs whose singing is being separated, so a song is separated
/// once however often the edit is saved meanwhile.
static SEPARATING: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
/// One separation at a time: each runs the music separation model.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Separates, in the background, the singing in each song `clips` play that
/// makes way for the voice and has none separated yet, once Studio
/// ducking's model is on this computer, and carries it into the project
/// and the library. Until then the song makes way as a whole.
pub(crate) fn separate_singing(app: &AppHandle, project: &Path, clips: &[RecordingAudioClip]) {
  let Some(model) = crate::editor::speech::studio_ducking::model(app) else {
    return;
  };
  let folder = project_sounds(project);
  let songs: BTreeSet<&str> = clips
    .iter()
    .filter(|clip| clip.makes_way && clip.kind == AudioClipKind::Music)
    .map(|clip| clip.sound.as_str())
    .collect();
  for name in songs {
    let Some((path, _)) = store::resolve(folder.as_deref(), name) else {
      continue;
    };
    let singing = store::singing_path(&path);
    if singing.is_file() {
      continue;
    }
    let is_new = SEPARATING
      .lock()
      .is_ok_and(|mut separating| separating.insert(name.to_owned()));
    if !is_new {
      continue;
    }
    let (name, model, folder) = (name.to_owned(), model.clone(), folder.clone());
    std::thread::spawn(move || {
      let separated = {
        let _one = ONE_AT_A_TIME
          .lock()
          .unwrap_or_else(|poisoned| poisoned.into_inner());
        screenwide_speech::duck::separate_singing(&path, model, &singing, &mut |_| {})
      };
      match (separated, folder) {
        // Made beside whichever copy was found first, it is handed to the
        // other, as a sound is.
        (Ok(()), Some(folder)) => {
          store::carry(&folder, [name.as_str()]);
          store::adopt(&folder);
        }
        (Ok(()), None) => {}
        (Err(error), _) => eprintln!("Could not separate the singing in {name}: {error}"),
      }
      if let Ok(mut separating) = SEPARATING.lock() {
        separating.remove(&name);
      }
    });
  }
}
