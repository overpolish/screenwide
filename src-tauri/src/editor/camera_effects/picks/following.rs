// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The project's picked objects followed in the background, one worker to a
//! project, each lot laid into the matte as it is followed.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter};

use super::super::commands::shown_changed;
use super::super::stop::{self, Running, Work};
use super::super::{choice, CameraRecording};
use super::commands::CHANGED_EVENT;
use super::{change, compose, follow, model, read, remove_masks};

/// How far following the objects `ids` has got, 0 to 1, as
/// `{ artifactId, ids, fraction }`.
const FOLLOWING_EVENT: &str = "editor://camera-objects-following";

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Following<'a> {
  artifact_id: u64,
  ids: &'a [u32],
  fraction: f32,
}

/// The projects whose objects are being followed, so each has one worker,
/// with the ids of the lot it is following.
static FOLLOWING: Mutex<Vec<(PathBuf, Vec<u32>)>> = Mutex::new(Vec::new());

fn following() -> std::sync::MutexGuard<'static, Vec<(PathBuf, Vec<u32>)>> {
  FOLLOWING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Starts following the project's objects not yet followed, unless that is
/// under way, the background is kept, or the model is not downloaded.
pub(super) fn start_following(app: &AppHandle, artifact_id: u64, camera: &CameraRecording) {
  let folder = camera.folder();
  if model(app).is_none()
    || !choice(&folder).removes_background()
    || !read(&folder).picks.iter().any(|pick| !pick.followed)
  {
    return;
  }
  {
    let mut following = following();
    if following.iter().any(|(at, _)| *at == folder) {
      return;
    }
    following.push((folder.clone(), Vec::new()));
  }
  let (app, camera) = (app.clone(), camera.clone());
  tauri::async_runtime::spawn_blocking(move || {
    follow_all(&app, artifact_id, &camera);
    following().retain(|(at, _)| *at != folder);
  });
}

/// Stops following the lot under way in `folder` once none of its objects
/// is left in the picks.
pub(super) fn stop_unwanted(folder: &Path) {
  let picks = read(folder).picks;
  let following = following();
  let unwanted = following.iter().any(|(at, ids)| {
    at == folder && !ids.is_empty() && ids.iter().all(|id| picks.iter().all(|pick| pick.id != *id))
  });
  if unwanted {
    stop::stop(folder, Work::Following);
  }
}

/// Marks `ids` as the lot followed in `folder` and starts it, together, so
/// [`stop_unwanted`] never sees the lot without the work it can stop.
fn begin_lot(folder: &Path, ids: &[u32]) -> Running {
  let mut following = following();
  if let Some((_, lot)) = following.iter_mut().find(|(at, _)| at == folder) {
    lot.clone_from(&ids.to_vec());
  }
  stop::begin(folder, Work::Following)
}

/// Follows the project's objects until none is left, those clicked in the
/// same frame together, laying them into the matte from the click on as soon
/// as that much is followed, and again once they are followed to the start.
/// Those that cannot be followed are taken out of the picks. Keeping the
/// background stops it, leaving the rest to follow once it is taken away
/// again.
fn follow_all(app: &AppHandle, artifact_id: u64, camera: &CameraRecording) {
  let folder = camera.folder();
  loop {
    if !choice(&folder).removes_background() {
      return;
    }
    let waiting = read(&folder)
      .picks
      .into_iter()
      .filter(|pick| !pick.followed)
      .collect::<Vec<_>>();
    let Some(frame) = waiting.first().map(|pick| pick.frame) else {
      return;
    };
    let lot = waiting
      .into_iter()
      .filter(|pick| pick.frame == frame)
      .collect::<Vec<_>>();
    let Some(files) = model(app) else {
      return;
    };
    let ids = lot.iter().map(|pick| pick.id).collect::<Vec<_>>();
    let running = begin_lot(&folder, &ids);
    // Each object shows how far it has got; those waiting their turn do not.
    let tell = |fraction: f32| {
      let _ = app.emit(
        FOLLOWING_EVENT,
        Following {
          artifact_id,
          ids: &ids,
          fraction,
        },
      );
    };
    tell(0.0);
    // Laid in on a thread of its own, so following goes on meanwhile; the
    // matte is laid in once more when following ends, after this.
    let mut laying = None;
    let followed = follow(
      camera,
      (files, running.stop()),
      &lot,
      &mut |fraction| tell(fraction),
      &mut || {
        let (app, camera, ids) = (app.clone(), camera.clone(), ids.clone());
        laying = Some(std::thread::spawn(move || {
          let marked = change(&camera.folder(), |picks| {
            for each in picks.picks.iter_mut().filter(|each| ids.contains(&each.id)) {
              each.after = true;
            }
          });
          if marked.is_ok() {
            lay_in(&app, artifact_id, &camera);
          }
        }));
      },
    );
    if let Some(laying) = laying {
      let _ = laying.join();
    }
    let stopped = running.is_stopped();
    drop(running);
    let kept = match followed {
      Ok(()) => change(&folder, |picks| {
        for each in picks.picks.iter_mut().filter(|each| ids.contains(&each.id)) {
          each.after = true;
          each.followed = true;
        }
      })
      .map(|picks| picks.picks.iter().map(|each| each.id).collect::<Vec<_>>()),
      // Stopped, the lot is either out of the picks already or waits for
      // the background to be taken away again.
      Err(_) if stopped => Ok(read(&folder).picks.iter().map(|each| each.id).collect()),
      Err(error) => {
        eprintln!("Could not follow the camera's objects: {error}");
        let _ = change(&folder, |picks| {
          picks.picks.retain(|each| !ids.contains(&each.id))
        });
        Ok(Vec::new())
      }
    }
    .unwrap_or_default();
    // One taken out of the picks while it was followed goes now.
    for id in ids.iter().filter(|id| !kept.contains(id)) {
      remove_masks(&folder, *id);
    }
    lay_in(app, artifact_id, camera);
  }
}

/// Tells the editor the picks changed, lays the followed objects into the
/// matte where the background is taken away, and shows it.
fn lay_in(app: &AppHandle, artifact_id: u64, camera: &CameraRecording) {
  let _ = app.emit(CHANGED_EVENT, artifact_id);
  if !choice(&camera.folder()).removes_background() {
    return;
  }
  match compose(camera, &mut |_| {}) {
    Ok(()) => shown_changed(app, artifact_id, camera),
    Err(error) => eprintln!("Could not lay the camera's objects into its matte: {error}"),
  }
}
