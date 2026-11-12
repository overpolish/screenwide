// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The singing separated from the songs clips play, a piece at a time and
//! only where they play it: it gets out of the voice's way entirely while
//! the rest of the song only comes down, and is taken out of an instrumental
//! song. The work runs in the background, one song at a time, as the edit is
//! saved; each window hears how far a song has got (`SINGING_EVENT`), and
//! the preview plays the singing from the next play once it is there. An
//! export separates whatever it still needs first.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_sound_track::store::singing::{self, missing, PIECE_MS};
use screenwide_sound_track::store::Separated;
use screenwide_sound_track::{AudioClipKind, RecordingAudioClip, SoundTrack};
use screenwide_transcriber::{SEPARATE_CHUNK, SEPARATE_SAMPLE_RATE};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::{ducks, file, microphone, project_sounds};

/// How many frames of a song each piece of its singing holds.
const PIECE_FRAMES: u64 = PIECE_MS * SEPARATE_SAMPLE_RATE as u64 / 1_000;
// A piece is exactly what the model hears at once, so pieces separated apart
// sit together as if separated whole.
const _: () = assert!(PIECE_FRAMES == SEPARATE_CHUNK as u64);

/// The event each window hears as a song's singing is separated.
const SINGING_EVENT: &str = "editor://singing";

/// How far the separation of a song's singing has got.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SingingProgress<'a> {
  sound: &'a str,
  /// From 0 to 1; none once the work is done or given up.
  fraction: Option<f32>,
}

/// The pieces each song wants separated, by name, with the file they are
/// separated from: the latest edit's, so a worker already under way picks up
/// what an edit saved meanwhile adds.
static WANTED: Mutex<BTreeMap<String, (PathBuf, BTreeSet<u64>)>> = Mutex::new(BTreeMap::new());
/// One separation at a time: each runs the music separation model.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The pieces of each song `clips` play, crossfades included, from the files
/// the project whose manifest is `project` plays them from: songs played
/// instrumental, and every song while they make way for the voice.
fn wanted(
  app: &AppHandle,
  project: &Path,
  clips: &[RecordingAudioClip],
) -> Vec<(String, PathBuf, BTreeSet<u64>)> {
  let ducking = microphone(project).is_some_and(|microphone| ducks(app, project, microphone));
  let songs: Vec<RecordingAudioClip> = clips
    .iter()
    .filter(|clip| clip.kind == AudioClipKind::Music && (clip.is_instrumental || ducking))
    .cloned()
    .collect();
  let folder = project_sounds(project);
  let track = SoundTrack {
    clips: songs,
    ..SoundTrack::default()
  }
  .with_files(|name| file(folder.as_deref(), name));
  let mut wanted: Vec<(String, PathBuf, BTreeSet<u64>)> = track
    .files
    .iter()
    .map(|(name, file)| (name.clone(), file.path.clone(), BTreeSet::new()))
    .collect();
  for piece in track.pieces(u64::MAX) {
    if let Some((_, _, pieces)) = wanted.iter_mut().find(|(_, path, _)| *path == piece.path) {
      let file_end_ms = piece.file_start_ms + (piece.end_ms - piece.start_ms);
      pieces.extend(singing::pieces_between(piece.file_start_ms, file_end_ms));
    }
  }
  wanted.retain(|(_, _, pieces)| !pieces.is_empty());
  wanted
}

/// Separates, in the background, the pieces of each song `clips` play that
/// are not separated yet, once Studio ducking's model is on this computer,
/// and carries them into the project and the library. Until then a song
/// plays, and makes way, as a whole.
pub(crate) fn separate_singing(app: &AppHandle, project: &Path, clips: &[RecordingAudioClip]) {
  let Some(model) = crate::editor::speech::studio_ducking::model(app) else {
    return;
  };
  let folder = project_sounds(project);
  for (name, path, pieces) in wanted(app, project, clips) {
    if missing(Separated::read(&path).as_ref(), &pieces).is_empty() {
      continue;
    }
    let is_new = {
      let Ok(mut wanted) = WANTED.lock() else {
        continue;
      };
      wanted.insert(name.clone(), (path, pieces)).is_none()
    };
    if !is_new {
      continue;
    }
    let (app, model, folder) = (app.clone(), model.clone(), folder.clone());
    std::thread::spawn(move || {
      let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      // Until no edit wants more: one saved while this worked may have.
      while let Some((path, pieces)) = WANTED
        .lock()
        .ok()
        .and_then(|wanted| wanted.get(&name).cloned())
      {
        let runs = missing(Separated::read(&path).as_ref(), &pieces);
        if runs.is_empty() {
          // Done, unless an edit saved meanwhile asked for more.
          let Ok(mut wanted) = WANTED.lock() else {
            break;
          };
          if wanted.get(&name).is_some_and(|(_, now)| *now == pieces) {
            wanted.remove(&name);
            break;
          }
          continue;
        }
        let told = |fraction: Option<f32>| {
          let _ = app.emit(
            SINGING_EVENT,
            SingingProgress {
              sound: &name,
              fraction,
            },
          );
        };
        if let Err(error) = separate_runs(&path, &runs, model.clone(), &mut |fraction| {
          told(Some(fraction));
        }) {
          eprintln!("Could not separate the singing in {name}: {error}");
          if let Ok(mut wanted) = WANTED.lock() {
            wanted.remove(&name);
          }
          told(None);
          break;
        }
        // Made beside whichever copy was found first, it is handed to the
        // other, as a sound is.
        if let Some(folder) = &folder {
          screenwide_sound_track::store::carry(folder, [name.as_str()]);
          screenwide_sound_track::store::adopt(folder);
        }
        told(None);
      }
    });
  }
}

/// Separates the pieces of each song `clips` play that are not separated
/// yet, before an export, so it hears instrumental songs without their
/// singing and the singing out of the voice's way, as the preview does once
/// the work is done. Without the model the songs play as a whole.
pub(crate) fn ready_for_export(
  app: &AppHandle,
  project: &Path,
  clips: &[RecordingAudioClip],
) -> Result<(), String> {
  let Some(model) = crate::editor::speech::studio_ducking::model(app) else {
    return Ok(());
  };
  for (_, path, pieces) in wanted(app, project, clips) {
    let _one = ONE_AT_A_TIME
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let runs = missing(Separated::read(&path).as_ref(), &pieces);
    separate_runs(&path, &runs, model.clone(), &mut |_| {})?;
  }
  Ok(())
}

/// Separates each run of pieces of the song `sound` into its two parts and
/// records them there, telling `progress` how far it has got across them
/// all.
fn separate_runs(
  sound: &Path,
  runs: &[std::ops::Range<u64>],
  model: PathBuf,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  singing::forget_unrecorded(sound);
  let total: u64 = runs.iter().map(|run| run.end - run.start).sum();
  let mut done = 0;
  for run in runs {
    let length = run.end - run.start;
    #[expect(clippy::cast_precision_loss, reason = "a fraction")]
    let share = |pieces: u64| pieces as f32 / total.max(1) as f32;
    screenwide_speech::duck::separate_singing(
      sound,
      model.clone(),
      (run.start * PIECE_FRAMES, run.end * PIECE_FRAMES),
      (
        &singing::singing_path(sound),
        &singing::rest_path(sound),
        singing::PART_LEVEL,
      ),
      &mut |fraction| progress(share(done) + fraction * share(length)),
    )?;
    singing::record(sound, run.clone())?;
    done += length;
  }
  Ok(())
}
