// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reframing a scene's pane with the crop tool. While it is in hand the
//! preview draws that pane so the crop window the webview lays over it shows
//! what the scene picks from. A preset's pane is drawn unzoomed, the screen at
//! the scene's box showing all of its crop and the camera as its whole picture
//! covering its box. A custom scene's pane is cropped the way the recording's
//! own composition is, so its picture is drawn where it already sits, uncut by
//! its box. The screen is drawn upright either way, since the crop window is
//! never turned, and the webview eases the scene's turn of it out before the
//! reframe and back in after. Only the paused frame is drawn this way; the
//! clip itself is not changed.

use super::*;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::{RecordingSceneClip, SceneRotation};

#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SceneReframe {
  Screen,
  Camera,
}

/// What the crop tool asks of the scenes: the pane it reframes, if any, and
/// how much of a scene's turn of the screen is drawn, from upright at zero
/// to as turned at one, while that turn eases out or back in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SceneCrop {
  pub(super) pane: Option<SceneReframe>,
  pub(super) screen_turn: f64,
}

impl Default for SceneCrop {
  fn default() -> Self {
    Self {
      pane: None,
      screen_turn: 1.0,
    }
  }
}

impl SceneCrop {
  /// How much of the screen's turn is drawn: none while the screen itself is
  /// reframed, whose crop window is never turned.
  pub(super) fn screen_turn(self) -> f64 {
    if self.pane == Some(SceneReframe::Screen) {
      0.0
    } else {
      self.screen_turn
    }
  }
}

/// Whether the pane `reframe` names has a box of its own in `clip`, which the
/// crop tool then crops in place rather than reframing.
pub(super) fn crops_in_place(clip: &RecordingSceneClip, reframe: SceneReframe) -> bool {
  clip.boxes.is_some_and(|boxes| match reframe {
    SceneReframe::Screen => true,
    SceneReframe::Camera => boxes.camera.is_some(),
  })
}

/// `clips` as the preview draws them while `reframe` is in hand: the screen's
/// framing of the clip `timeline_ms` into the edited video taken away, so
/// the screen shows whole, unless that screen is cropped in place.
pub(super) fn reframed_clips(
  clips: &[RecordingSceneClip],
  reframe: Option<SceneReframe>,
  timeline_ms: u64,
) -> Option<Vec<RecordingSceneClip>> {
  if reframe != Some(SceneReframe::Screen) {
    return None;
  }
  let mut clips = clips.to_vec();
  let clip = clips
    .iter_mut()
    .find(|clip| clip.start_ms <= timeline_ms && timeline_ms < clip.end_ms)?;
  if crops_in_place(clip, SceneReframe::Screen) {
    return None;
  }
  clip.screen = None;
  Some(clips)
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
/// is reframing, if any, and how much of the scene's turn of the screen to
/// draw, redrawing a paused frame they change.
#[tauri::command]
pub async fn set_recording_preview_scene_reframe(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  pane: Option<SceneReframe>,
  screen_turn: f64,
) -> Result<(), String> {
  if !(0.0..=1.0).contains(&screen_turn) {
    return Err("The screen's turn must be a share between 0 and 1".to_owned());
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
  let next = SceneCrop { pane, screen_turn };
  if *current == next {
    return Ok(());
  }
  *current = next;
  drop(current);
  manager.redraw_paused_frame()
}

#[cfg(test)]
mod tests;
