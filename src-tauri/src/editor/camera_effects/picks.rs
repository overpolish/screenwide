// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Objects picked in the camera where the matte has them wrong: one to keep
//! with the person, such as a shoulder the matte drops, or one to take away
//! with the room, such as a sofa it keeps. Each is clicked in one frame and
//! followed through the whole recording by SAM 2.1 and Cutie in the
//! transcriber program, then laid into the matte: the drawings read the
//! matte with the objects in, so nothing else changes for them.
//!
//! The matte with the objects in is written under a new name each time,
//! since a drawing may still be reading the one before.

pub(crate) mod commands;
mod following;
mod laying;

pub(super) use laying::{compose, remove_composed};

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use screenwide_transcriber::{ClickedObject, Job};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use ts_rs::TS;

use super::{matte, CameraRecording};
use crate::editor::speech::choice_file;
use crate::transcription::catalogue::{self, Purpose};
use crate::transcription::models;
use crate::transcription::runner::Transcriber;

/// The make of the picks file. Picks of the first make were followed in a
/// form the matte no longer reads.
const FORMAT_VERSION: u16 = 2;
const COMPOSED_PREFIX: &str = "camera-matte-picked-";
/// Changes whenever the objects are laid into the matte differently, so a
/// matte laid the old way is made again.
const COMPOSED_MAKE: u32 = 4;

/// One object picked in the camera.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CameraPick {
  pub id: u32,
  /// Kept with the person, or taken away with the room.
  pub keep: bool,
  /// The camera frame it was picked in, by its place in the matte's frames,
  /// and when that frame shows.
  pub frame: u64,
  pub frame_ms: u64,
  /// Where it was clicked, 0 to 1 across and down the frame: the click, or
  /// where a line painted over it starts.
  pub x: f32,
  pub y: f32,
  /// The line painted over it, 0 to 1 across and down the frame, or none
  /// where it was clicked.
  #[serde(default)]
  pub line: Vec<[f32; 2]>,
  /// Whether it has been followed through the whole recording.
  pub followed: bool,
  /// Whether it has been followed from where it was clicked to the end,
  /// which is done first.
  #[ts(skip)]
  #[serde(default)]
  pub(super) after: bool,
}

/// The objects picked in a project, as it keeps them.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Picks {
  pub picks: Vec<CameraPick>,
  pub next_id: u32,
  /// The file name of the matte with the followed objects in, once made,
  /// and how it was made.
  pub composed: Option<String>,
  #[serde(default)]
  pub composed_make: u32,
}

/// One change to the picks at a time, as following and picking both write
/// them.
static CHANGING: Mutex<()> = Mutex::new(());

fn picks_path(folder: &Path) -> PathBuf {
  folder.join("camera-picks.json")
}

pub(super) fn masks_folder(folder: &Path) -> PathBuf {
  folder.join("camera-picks")
}

/// Where the masks of the object `id` go: those from where it was clicked to
/// the end, or `before`, those from the start to where it was clicked.
fn mask_path(folder: &Path, id: u32, before: bool) -> PathBuf {
  let side = if before { "before" } else { "after" };
  masks_folder(folder).join(format!("{id}-{side}.mp4"))
}

/// Takes away the masks of the object `id`.
pub(super) fn remove_masks(folder: &Path, id: u32) {
  for before in [false, true] {
    let _ = std::fs::remove_file(mask_path(folder, id, before));
  }
}

pub(super) fn read(folder: &Path) -> Picks {
  choice_file::read(&picks_path(folder), FORMAT_VERSION).unwrap_or_default()
}

/// Changes the project's picks with `change`, answering what they are now.
pub(super) fn change(folder: &Path, change: impl FnOnce(&mut Picks)) -> Result<Picks, String> {
  let _changing = CHANGING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let mut picks = read(folder);
  change(&mut picks);
  choice_file::write(&picks_path(folder), FORMAT_VERSION, &picks)
    .map_err(|error| format!("Could not keep the camera's objects: {error}"))?;
  Ok(picks)
}

/// The matte with the objects in, where there are any and it is made.
pub(crate) fn composed(folder: &Path) -> Option<PathBuf> {
  let picks = read(folder);
  if !picks.picks.iter().any(|pick| pick.after) {
    return None;
  }
  if picks.composed_make != COMPOSED_MAKE {
    return None;
  }
  picks
    .composed
    .map(|name| folder.join(name))
    .filter(|path| path.is_file())
}

/// Whether objects are followed whose matte is not made, as after the
/// matte was made again.
pub(super) fn needs_composing(folder: &Path) -> bool {
  matte::is_made(folder)
    && read(folder).picks.iter().any(|pick| pick.after)
    && composed(folder).is_none()
}

/// How many points along a painted line show the model the object: enough
/// to say a line across it is all one thing, few enough that it is not read
/// as a dozen separate clicks.
const LINE_PROMPTS: usize = 8;

/// Where the model is shown the object: the click, or points spread evenly
/// along the line painted over it.
fn prompts(pick: &CameraPick) -> Vec<[f32; 2]> {
  let line = &pick.line;
  if line.is_empty() {
    return vec![[pick.x, pick.y]];
  }
  let count = line.len().min(LINE_PROMPTS);
  (0..count)
    .map(|step| line[step * (line.len() - 1) / (count - 1).max(1)])
    .collect()
}

/// The object following model's files, where it is downloaded.
pub(super) fn model(app: &AppHandle) -> Option<Vec<PathBuf>> {
  let model = catalogue::for_purpose(Purpose::CameraObjects)?;
  if !models::is_downloaded(app, model) {
    return None;
  }
  models::paths(app, model).ok()
}

/// Follows `picks`, all clicked in the same frame, through `camera` together
/// with the model's `files`, telling `progress` how far it has got: from
/// there to the end, their masks kept and `halfway` told, then to the start.
/// Answers once all their masks are kept, or once `stop` is set.
pub(super) fn follow(
  camera: &CameraRecording,
  (files, stop): (Vec<PathBuf>, Arc<AtomicBool>),
  picks: &[CameraPick],
  progress: &mut dyn FnMut(f32),
  halfway: &mut dyn FnMut(),
) -> Result<(), String> {
  let Some(frame) = picks.first().map(|pick| pick.frame) else {
    return Ok(());
  };
  let folder = camera.folder();
  std::fs::create_dir_all(masks_folder(&folder))
    .map_err(|error| format!("Could not keep the camera's objects: {error}"))?;
  let partial = |pick: &CameraPick, before: bool| {
    mask_path(&folder, pick.id, before).with_extension("partial.mp4")
  };
  let keep = |before: bool| {
    picks.iter().try_for_each(|pick| {
      std::fs::rename(partial(pick, before), mask_path(&folder, pick.id, before))
        .map_err(|error| format!("Could not keep the camera's objects: {error}"))
    })
  };
  let made = Transcriber::start()?.stopped_by(stop).camera_objects(
    Job::FollowObjects {
      model: files,
      ffmpeg: crate::editor::media_preview::ffmpeg_path(),
      camera: camera.path.clone(),
      timings: matte::timings_path(&folder),
      frame,
      objects: picks
        .iter()
        .map(|pick| ClickedObject {
          points: prompts(pick),
          after: partial(pick, false),
          before: partial(pick, true),
        })
        .collect(),
    },
    progress,
    &mut || match keep(false) {
      Ok(()) => halfway(),
      Err(error) => eprintln!("{error}"),
    },
  );
  let kept = made.and_then(|()| keep(true));
  if kept.is_err() {
    // With the masks, the frames a run stopped part-way held for them.
    for pick in picks {
      for before in [false, true] {
        let _ = std::fs::remove_file(partial(pick, before));
        let _ = std::fs::remove_file(partial(pick, before).with_extension("slots"));
      }
    }
  }
  kept
}
