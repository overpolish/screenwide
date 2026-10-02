// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reframing a scene's pane with the crop tool. While it is in hand the
//! preview draws that pane so the crop window the webview lays over it shows
//! what the scene picks from. A preset's pane is drawn unzoomed, the screen at
//! the scene's box showing all of its crop and the camera as its whole picture
//! covering its box. A custom scene's pane is cropped the way the recording's
//! own composition is, so its picture is drawn where it already sits, uncut by
//! its box. Only the paused frame is drawn this way; the clip itself is not
//! changed.

use super::*;
use crate::editor::scenes::RecordingSceneClip;
use crate::screenshots::ScreenshotOutputSettings;

#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SceneReframe {
  Screen,
  Camera,
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
/// framing of the clip at `source_ms` taken away, so the screen shows whole,
/// unless that screen is cropped in place.
pub(super) fn reframed_clips(
  clips: &[RecordingSceneClip],
  reframe: Option<SceneReframe>,
  source_ms: u64,
) -> Option<Vec<RecordingSceneClip>> {
  if reframe != Some(SceneReframe::Screen) {
    return None;
  }
  let mut clips = clips.to_vec();
  let clip = clips
    .iter_mut()
    .find(|clip| clip.start_ms <= source_ms && source_ms < clip.end_ms)?;
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
/// is reframing, if any, redrawing a paused frame it changes.
#[tauri::command]
pub async fn set_recording_preview_scene_reframe(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  pane: Option<SceneReframe>,
) -> Result<(), String> {
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
    .reframe
    .write()
    .map_err(|_| "The scenes are unavailable".to_owned())?;
  if *current == pane {
    return Ok(());
  }
  *current = pane;
  drop(current);
  if !manager.is_playing {
    manager.restart(PlaybackMode::InteractiveStill)?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_whole_camera_overhangs_a_square_box_from_its_middle() {
    // The editor's `reframeWindow` pins the same picture for the same box.
    let mut overlay = CameraOverlaySettings {
      camera_x: 190.0,
      camera_y: 190.0,
      camera_width: 640.0,
      frame_height: 180.0,
      frame_width: 180.0,
      frame_x: 100.0,
      frame_y: 100.0,
      radius_percent: 50.0,
    };
    show_whole_camera(&mut overlay, (1_920, 1_080));
    assert_eq!(
      (
        overlay.frame_x,
        overlay.frame_y,
        overlay.frame_width,
        overlay.frame_height
      ),
      (30.0, 100.0, 320.0, 180.0)
    );
    assert_eq!(
      (overlay.camera_x, overlay.camera_y, overlay.camera_width),
      (190.0, 190.0, 320.0)
    );
    assert_eq!(overlay.radius_percent, 0.0);
  }

  #[test]
  fn a_custom_screen_cropped_in_place_shows_its_picture_where_it_sits() {
    // A 2:1 screen in a box at a tenth of a 1600 by 900 canvas, zoomed twice
    // past covering it. The editor's `pictureRect` pins the same rect.
    let clip: RecordingSceneClip = serde_json::from_value(serde_json::json!({
      "id": "c",
      "startMs": 0,
      "endMs": 10_000,
      "preset": "full",
      "screen": { "focusX": 0.5, "focusY": 0.5, "zoom": 2.0 },
      "boxes": { "screen": { "x": 0.1, "y": 0.1, "width": 0.5, "height": 0.5 } },
    }))
    .expect("a custom clip");
    assert!(crops_in_place(&clip, SceneReframe::Screen));
    assert!(!crops_in_place(&clip, SceneReframe::Camera));
    assert_eq!(
      reframed_clips(
        std::slice::from_ref(&clip),
        Some(SceneReframe::Screen),
        5_000
      ),
      None
    );
    let base = crate::screenshots::test_output_settings(1_600, 900);
    let mut output = base.clone();
    assert!(crate::editor::scenes::arrange(
      &[clip],
      &[],
      5_000,
      0.0,
      &mut output,
      None
    ));
    show_screen_picture(&mut output, &base);
    let shown = [
      output.crop_x,
      output.crop_y,
      output.crop_width,
      output.crop_height,
    ];
    for (shown, expected) in shown.into_iter().zip([-340.0, -135.0, 1_800.0, 900.0]) {
      assert!((shown - expected).abs() < 1e-6, "{shown} is not {expected}");
    }
  }
}
