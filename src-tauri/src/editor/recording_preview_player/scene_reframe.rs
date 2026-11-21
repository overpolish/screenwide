// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reframing a scene's pane with the crop tool. While it is in hand the
//! preview draws that pane so the crop window the webview lays over it shows
//! what the scene picks from: cropped the way the recording's own
//! composition is, its picture drawn where it already sits, uncut by its
//! box, so nothing moves as the crop tool takes it. A preset's crop keeps
//! its layout, which makes room for the crop's shape; only a custom scene's
//! camera with no box of its own, sitting in the recording's place, is drawn
//! as its whole picture covering that place instead. The screen is drawn
//! upright either way, since the crop window is never turned, and the
//! webview eases the scene's turn of it out before the reframe and back in
//! after. Only the paused frame is drawn this way; the clip itself is not
//! changed.

use super::*;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::{RecordingSceneClip, SceneRotation};

#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SceneReframe {
  Screen,
  Camera,
}

/// What the crop tool asks of the scenes: the pane it reframes, if any, how
/// far that pane is opened out around its crop, from as the scene lays it at
/// zero to its whole picture at one, and how much of a scene's turn of the
/// screen is drawn, from upright at zero to as turned at one. The webview
/// eases both as the tool comes and goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SceneCrop {
  pub(super) pane: Option<SceneReframe>,
  pub(super) open: f64,
  pub(super) screen_turn: f64,
}

impl Default for SceneCrop {
  fn default() -> Self {
    Self {
      pane: None,
      open: 0.0,
      screen_turn: 1.0,
    }
  }
}

/// Draws the pane `reframe` names in `composition` as the crop tool shows it
/// once open: cropped `in_place` its whole picture where it sits, else the
/// camera's whole picture covering its place. `camera` is the camera's
/// source size, `base` the screen as the recording composes it.
pub(super) fn open_pane(
  composition: &mut PreviewCompositionSettings,
  (reframe, in_place, camera): (SceneReframe, bool, Option<(u32, u32)>),
  base: &ScreenshotOutputSettings,
) {
  match (reframe, camera.filter(|_| composition.bake_camera)) {
    (SceneReframe::Screen, _) if in_place => {
      show_screen_picture(&mut composition.recording_output.primary, base)
    }
    (SceneReframe::Camera, Some(camera)) if in_place => {
      show_camera_picture(&mut composition.camera_overlay, camera)
    }
    (SceneReframe::Camera, Some(camera)) => {
      show_whole_camera(&mut composition.camera_overlay, camera)
    }
    _ => {}
  }
}

/// `laid_out` drawn `open` of the way to `opened`: the screen's window and
/// picture and the camera's box and picture each that part of the way, so
/// the crop tool opens a pane out of its place and closes it back into it.
pub(super) fn ease_open(
  laid_out: &mut PreviewCompositionSettings,
  opened: &PreviewCompositionSettings,
  open: f64,
) {
  let ease = |from: &mut f64, to: f64| *from += (to - *from) * open;
  let (screen, to) = (
    &mut laid_out.recording_output.primary,
    &opened.recording_output.primary,
  );
  ease(&mut screen.crop_x, to.crop_x);
  ease(&mut screen.crop_y, to.crop_y);
  ease(&mut screen.crop_width, to.crop_width);
  ease(&mut screen.crop_height, to.crop_height);
  ease(&mut screen.image_x, to.image_x);
  ease(&mut screen.image_y, to.image_y);
  ease(&mut screen.image_width, to.image_width);
  let (camera, to) = (&mut laid_out.camera_overlay, &opened.camera_overlay);
  ease(&mut camera.frame_x, to.frame_x);
  ease(&mut camera.frame_y, to.frame_y);
  ease(&mut camera.frame_width, to.frame_width);
  ease(&mut camera.frame_height, to.frame_height);
  ease(&mut camera.camera_x, to.camera_x);
  ease(&mut camera.camera_y, to.camera_y);
  ease(&mut camera.camera_width, to.camera_width);
  ease(&mut camera.radius_percent, to.radius_percent);
}

/// Whether the crop tool crops the pane `reframe` names in place in `clip`:
/// every pane but a custom scene's camera with no box of its own.
pub(super) fn crops_in_place(clip: &RecordingSceneClip, reframe: SceneReframe) -> bool {
  clip.boxes.is_none_or(|boxes| match reframe {
    SceneReframe::Screen => true,
    SceneReframe::Camera => boxes.camera.is_some(),
  })
}

/// The screen of an `output` a scene arranged from `base`, its box opened out
/// to the whole of the picture it shows: the part of the screen `base` crops
/// to, at the size and place the scene draws it. The crop window
/// `customCropSelection` lays over it in the editor assumes the same place.
pub(super) fn show_screen_picture(
  output: &mut ScreenshotOutputSettings,
  base: &ScreenshotOutputSettings,
) {
  let scale = output.image_width / base.image_width.max(f64::EPSILON);
  output.crop_x = output.image_x + (base.crop_x - base.image_x) * scale;
  output.crop_y = output.image_y + (base.crop_y - base.image_y) * scale;
  output.crop_width = base.crop_width * scale;
  output.crop_height = base.crop_height * scale;
}

/// The screen of `output` drawn with `share` of its turn, each angle that part
/// of the way from upright, and the turn its motion blur reads back through
/// with it, so a frame the shutter left open on blends nothing of the rest.
pub(super) fn ease_screen_turn(output: &mut ScreenshotOutputSettings, share: f64) {
  if share >= 1.0 {
    return;
  }
  let part = |rotation: SceneRotation| SceneRotation {
    x: rotation.x * share,
    y: rotation.y * share,
    z: rotation.z * share,
  };
  output.rotation = output
    .rotation
    .map(part)
    .filter(|rotation| !rotation.is_upright());
  if let Some(motion) = &mut output.scene_motion {
    motion.rotation = part(motion.rotation);
  }
}

/// A camera overlay a scene arranged, its box opened out to the whole camera
/// picture where the scene draws it, for a camera of `camera` source size.
pub(super) fn show_camera_picture(overlay: &mut CameraOverlaySettings, camera: (u32, u32)) {
  let aspect = f64::from(camera.0) / f64::from(camera.1.max(1));
  overlay.frame_width = overlay.camera_width;
  overlay.frame_height = overlay.camera_width / aspect;
  overlay.frame_x = overlay.camera_x - overlay.frame_width / 2.0;
  overlay.frame_y = overlay.camera_y - overlay.frame_height / 2.0;
  overlay.radius_percent = 0.0;
}

/// A camera overlay a scene arranged, drawn as its whole picture covering the
/// box the scene gave it, its middle on the box's middle, for a camera of
/// `camera` source size. The crop window `reframeWindow` lays over it in the
/// editor assumes the same place.
pub(super) fn show_whole_camera(overlay: &mut CameraOverlaySettings, camera: (u32, u32)) {
  let aspect = f64::from(camera.0) / f64::from(camera.1.max(1));
  let width = overlay.frame_width.max(overlay.frame_height * aspect);
  let height = width / aspect;
  let centre = (
    overlay.frame_x + overlay.frame_width / 2.0,
    overlay.frame_y + overlay.frame_height / 2.0,
  );
  overlay.camera_x = centre.0;
  overlay.camera_y = centre.1;
  overlay.camera_width = width;
  overlay.frame_x = centre.0 - width / 2.0;
  overlay.frame_y = centre.1 - height / 2.0;
  overlay.frame_width = width;
  overlay.frame_height = height;
  overlay.radius_percent = 0.0;
}

/// Tells the preview which pane of the scene under the playhead the crop tool
/// is reframing, if any, how far it is opened out, and how much of the
/// scene's turn of the screen to draw, redrawing a paused frame they change.
#[tauri::command]
pub async fn set_recording_preview_scene_reframe(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  pane: Option<SceneReframe>,
  open: f64,
  screen_turn: f64,
) -> Result<(), String> {
  if !(0.0..=1.0).contains(&screen_turn) || !(0.0..=1.0).contains(&open) {
    return Err(
      "The crop's opening and the screen's turn must be shares between 0 and 1".to_owned(),
    );
  }
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  let sources = manager
    .sources
    .as_ref()
    .ok_or_else(|| "The recording preview player is not open".to_owned())?;
  let mut current = sources
    .scenes
    .crop
    .write()
    .map_err(|_| "The scenes are unavailable".to_owned())?;
  let next = SceneCrop {
    pane,
    open,
    screen_turn,
  };
  if *current == next {
    return Ok(());
  }
  *current = next;
  drop(current);
  manager.redraw_paused_frame()
}

#[cfg(test)]
mod tests;
