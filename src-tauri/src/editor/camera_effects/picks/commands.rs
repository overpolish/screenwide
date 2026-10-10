// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's objects as the editor picks them: picking on the preview,
//! the objects picked, and taking one away again. Objects are followed in
//! the background, from where they were clicked to the end and then to the
//! start, each part laid into the matte as soon as it is done, with the
//! progress told on the Objects bar.

use tauri::{AppHandle, Emitter};

use super::super::commands::{open, shown_changed, Bars};
use super::super::make::CameraTool;
use super::super::matte;
use super::following::{start_following, stop_unwanted};
use super::{change, compose, model, read, remove_masks, CameraPick};
use crate::editor::recording_preview_player::{
  position_ms, set_camera_picking, CameraPicking, PickedSpan,
};

/// The event that tells the editor the objects picked have changed.
pub(super) const CHANGED_EVENT: &str = "editor://camera-picks-changed";

/// The objects picked in the camera of the recording open in the editor.
/// Any not yet followed, as when the recording closed while one was, start
/// following again.
#[tauri::command]
pub fn get_recording_camera_picks(
  app: AppHandle,
  artifact_id: u64,
) -> Result<Vec<CameraPick>, String> {
  let camera = open(&app, artifact_id)?;
  start_following(&app, artifact_id, &camera);
  Ok(read(&camera.folder()).picks)
}

/// Picks objects in the camera on the preview, with the object `chosen`
/// marked out among them, or stops when `picking` is off.
#[tauri::command]
pub fn set_recording_camera_picking(
  app: AppHandle,
  artifact_id: u64,
  picking: bool,
  chosen: Option<u32>,
) -> Result<(), String> {
  let camera = open(&app, artifact_id)?;
  let folder = camera.folder();
  let picking = picking.then(|| CameraPicking {
    app: app.clone(),
    artifact_id,
    picked: matte::timings(&folder).map_or_else(Vec::new, |timings| {
      read(&folder)
        .picks
        .iter()
        .filter_map(|pick| {
          let (from_ms, to_ms) = frame_span(&timings.frames_ms, pick.frame as usize)?;
          Some(PickedSpan {
            id: pick.id,
            from_ms,
            to_ms,
            x: pick.x,
            y: pick.y,
            line: pick.line.clone(),
            keep: pick.keep,
          })
        })
        .collect()
    }),
    chosen,
  });
  set_camera_picking(&app, artifact_id, picking);
  Ok(())
}

/// The camera's time frame `index` of `frames_ms` shows for: from its own
/// time to the next frame's, each read back up to 2 ms early.
fn frame_span(frames_ms: &[f64], index: usize) -> Option<(u64, u64)> {
  let at = |ms: f64| (ms - 2.0).max(0.0).ceil() as u64;
  let from = at(*frames_ms.get(index)?);
  let to = frames_ms.get(index + 1).map_or(u64::MAX, |next| at(*next));
  Some((from, to))
}

/// Picks the object at `points`, 0 to 1 across and down the camera frame
/// the preview shows, to keep with the person or take away with the room,
/// and starts following it, as a click on the camera, or a line painted
/// over it, does while objects are picked. Answers with its id.
pub(crate) fn add_pick(
  app: &AppHandle,
  artifact_id: u64,
  points: &[[f32; 2]],
  keep: bool,
) -> Result<u32, String> {
  let points = points
    .iter()
    .map(|point| point.map(|value| value.clamp(0.0, 1.0)))
    .collect::<Vec<_>>();
  let Some(&[x, y]) = points.first() else {
    return Err("An object was picked at no point".to_owned());
  };
  let camera = open(app, artifact_id)?;
  if model(app).is_none() {
    return Err("Download the object model first".to_owned());
  }
  let folder = camera.folder();
  let timings = matte::timings(&folder).ok_or_else(|| "Remove the background first".to_owned())?;
  let position = position_ms(app, artifact_id).unwrap_or(0);
  let ms = position.min(camera.duration_ms.saturating_sub(1)) as f64;
  // A frame shows from its own time, which a frame read back may be stamped
  // up to 2 ms after.
  let frame = timings
    .frames_ms
    .partition_point(|&frame_ms| frame_ms <= ms + 2.0)
    .saturating_sub(1);
  let frame_ms = *timings
    .frames_ms
    .get(frame)
    .ok_or_else(|| "That frame is not in the camera recording".to_owned())?;
  let mut id = 0;
  change(&folder, |picks| {
    id = picks.next_id;
    picks.picks.push(CameraPick {
      id: picks.next_id,
      keep,
      frame: frame as u64,
      frame_ms: frame_ms.max(0.0).round() as u64,
      x,
      y,
      line: if points.len() > 1 { points } else { Vec::new() },
      followed: false,
      after: false,
    });
    picks.next_id += 1;
  })?;
  let _ = app.emit(CHANGED_EVENT, artifact_id);
  start_following(app, artifact_id, &camera);
  Ok(id)
}

/// Takes the object `id` out of the picks, and out of the matte where it
/// was laid in. Answers with the objects picked now.
#[tauri::command]
pub async fn remove_recording_camera_pick(
  app: AppHandle,
  artifact_id: u64,
  id: u32,
) -> Result<Vec<CameraPick>, String> {
  let camera = open(&app, artifact_id)?;
  tauri::async_runtime::spawn_blocking(move || {
    let folder = camera.folder();
    let was_laid_in = read(&folder)
      .picks
      .iter()
      .any(|pick| pick.id == id && pick.after);
    let picks = change(&folder, |picks| picks.picks.retain(|pick| pick.id != id))?;
    // One still being followed is taken away when following ends, which
    // stops once nothing it follows is wanted.
    stop_unwanted(&folder);
    remove_masks(&folder, id);
    let _ = app.emit(CHANGED_EVENT, artifact_id);
    if was_laid_in {
      // A lay-in for a later change overtakes this one, which then has
      // nothing to show.
      let mut bars = Bars::new(&app, artifact_id);
      let laid = compose(&camera, &mut |fraction| {
        bars.tell(CameraTool::Objects, fraction);
      });
      drop(bars);
      match laid {
        Ok(()) => shown_changed(&app, artifact_id, &camera),
        Err(error) => eprintln!("Could not lay the camera's objects into its matte: {error}"),
      }
    }
    Ok(picks.picks)
  })
  .await
  .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
  use super::frame_span;

  #[test]
  fn a_pick_is_marked_for_as_long_as_its_frame_shows() {
    let frames = [0.0, 33.333, 66.667];
    // A frame read back is stamped up to 2 ms after its time, so each span
    // starts and ends 2 ms early, and the last runs on to the end.
    assert_eq!(frame_span(&frames, 0), Some((0, 32)));
    assert_eq!(frame_span(&frames, 1), Some((32, 65)));
    assert_eq!(frame_span(&frames, 2), Some((65, u64::MAX)));
    assert_eq!(frame_span(&frames, 3), None);
  }
}
