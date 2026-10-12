// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The followed objects laid into the camera's matte, one matte at a time,
//! each under a name of its own so a drawing reading the one before is not
//! written over.

use std::path::Path;
use std::sync::Mutex;

use screenwide_transcriber::{Job, PickedMask};

use super::super::stop::{self, Work};
use super::super::{matte, CameraRecording};
use super::{change, mask_path, read, COMPOSED_MAKE, COMPOSED_PREFIX};
use screenwide_speech::runner::Transcriber;

/// One matte composed at a time.
static COMPOSING: Mutex<()> = Mutex::new(());

/// Lays the followed objects into the camera's matte, or goes back to the
/// matte alone where none are left, and takes away the mattes made before.
pub(in crate::editor::camera_effects) fn compose(
  camera: &CameraRecording,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let folder = camera.folder();
  // The newest picks are read once the lock is held, so a lay-in under way
  // would only be done over again.
  stop::stop(&folder, Work::Composing);
  let _composing = COMPOSING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let running = stop::begin(&folder, Work::Composing);
  let picks = read(&folder);
  // The masks from the click on, and from the start up to it once followed
  // that way too.
  let followed = picks
    .picks
    .iter()
    .flat_map(|pick| {
      let after = (pick.after).then(|| PickedMask {
        path: mask_path(&folder, pick.id, false),
        keep: pick.keep,
        first: pick.frame,
        clicked: pick.frame,
      });
      let before = (pick.followed).then(|| PickedMask {
        path: mask_path(&folder, pick.id, true),
        keep: pick.keep,
        first: 0,
        clicked: pick.frame,
      });
      [after, before]
    })
    .flatten()
    .filter(|mask| mask.path.is_file())
    .collect::<Vec<_>>();
  let name = match (followed.is_empty(), matte::timings(&folder)) {
    (false, Some(timings)) => {
      let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_millis());
      let name = format!("{COMPOSED_PREFIX}{stamp}.mp4");
      let (size, _) = matte::shape(&camera.path);
      let (width, height) = matte::matte_size(size);
      let made = Transcriber::start()?
        .stopped_by(running.stop())
        .camera_objects(
          Job::ComposeMatte {
            ffmpeg: screenwide_sidecar::ffmpeg_path(),
            camera: camera.path.clone(),
            matte: matte::path(&folder),
            width,
            height,
            frames: timings.frames_ms.len() as u64,
            frames_per_second: timings.frames_per_second,
            picks: followed,
            output: folder.join(&name),
          },
          progress,
          &mut || {},
        );
      if let Err(error) = made {
        // Stopped by a newer lay-in, that one replaces the matte. Stopped as
        // the background was kept, the matte laid in before may hold an
        // object taken out since, so it goes, and is laid in afresh once the
        // background is taken away again.
        if running.is_stopped() && !super::super::choice(&folder).removes_background() {
          let _ = change(&folder, |picks| picks.composed = None);
          remove_composed(&folder, None);
        }
        return Err(error);
      }
      Some(name)
    }
    _ => None,
  };
  change(&folder, |picks| {
    picks.composed.clone_from(&name);
    picks.composed_make = COMPOSED_MAKE;
  })?;
  remove_composed(&folder, name.as_deref());
  Ok(())
}

/// Takes away every matte with objects in but `keep`. One a drawing still
/// has open may stay until the next time.
pub(in crate::editor::camera_effects) fn remove_composed(folder: &Path, keep: Option<&str>) {
  let Ok(entries) = std::fs::read_dir(folder) else {
    return;
  };
  for path in entries.flatten().map(|entry| entry.path()) {
    let Some(name) = path
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
    else {
      continue;
    };
    if name.starts_with(COMPOSED_PREFIX) && Some(name.as_str()) != keep {
      let _ = std::fs::remove_file(&path);
    }
  }
}
