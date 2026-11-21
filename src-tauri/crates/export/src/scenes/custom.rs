// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A preset scene made custom where it stands, so a drag on its screen can
//! move the boxes from there. The twin of `customRecordingSceneClip` in
//! `recording-scene-custom.ts`.

use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::geometry::Rect;
use screenwide_scenes::model::{RecordingScenePreset, SceneBox, SceneBoxes, SceneRadius};
use screenwide_scenes::{CameraOverlaySettings, RecordingSceneClip, SceneFraming};

use super::target::{fitted, Base};

/// `clip` made custom: its panes kept in the boxes its preset puts them in
/// on the screen's `output`, and showing what they show there, so the change
/// alone moves nothing. `camera` is the camera drawn into the picture, with
/// its source size, where there is one; a full scene's camera then gets the
/// recording's own box. A camera-only scene keeps the screen hidden in the
/// recording's own place, and a screen-only one the camera. `None` for a
/// clip already custom, for a blank scene, which has no pane to place, and
/// for an output with no placement to measure from.
pub fn made_custom(
  clip: &RecordingSceneClip,
  output: &ScreenshotOutputSettings,
  camera: Option<(&CameraOverlaySettings, (u32, u32))>,
) -> Option<RecordingSceneClip> {
  if clip.boxes.is_some() || clip.preset == RecordingScenePreset::Blank || !output.has_placement() {
    return None;
  }
  let camera = camera
    .filter(|(_, (width, height))| *width > 0 && *height > 0)
    .map(|(overlay, (width, height))| (overlay, f64::from(width) / f64::from(height)));
  let base = Base::of(output, camera, None);
  let crop = base.placement.screen;
  // Where the scene crops a pane to a shape of its own, its box has that
  // shape already (`Base::target`), and keeps it.
  let shapes = clip.shapes.unwrap_or_default();
  let (screen, preset_camera) = match base.preset_panes(clip) {
    Some(panes) => (panes.screen.unwrap_or(crop), panes.camera),
    None => (
      shapes.screen.map_or(crop, |shape| fitted(crop, shape)),
      None,
    ),
  };
  let camera_box = preset_camera.or_else(|| {
    (camera.is_some() && clip.preset == RecordingScenePreset::Full).then_some(
      shapes.camera.map_or(base.placement.frame, |shape| {
        fitted(base.placement.frame, shape)
      }),
    )
  });
  // An unset camera framing means a different picture once the camera has
  // a box of its own, so the one the clip shows now is written down.
  let framing = match (camera, camera_box) {
    (Some((overlay, aspect)), Some(frame)) => Some(
      clip
        .camera
        .unwrap_or_else(|| {
          if clip.needs_camera() {
            SceneFraming::WHOLE
          } else {
            SceneFraming::of_camera(overlay, aspect)
          }
        })
        .within(frame, aspect),
    ),
    _ => clip.camera,
  };
  let share = |rect: Rect| {
    SceneBox {
      x: rect.x / base.canvas.0.max(1.0),
      y: rect.y / base.canvas.1.max(1.0),
      width: rect.width / base.canvas.0.max(1.0),
      height: rect.height / base.canvas.1.max(1.0),
    }
    .moved((0.0, 0.0), 1.0)
  };
  let camera_only = clip.preset == RecordingScenePreset::CameraOnly;
  // A camera filling the canvas is cut by the canvas's corners, not rounded
  // inside them; a box of its own would round it, so the square corners it
  // shows are written down.
  let radius = match clip.radius {
    Some(SceneRadius {
      camera: Some(_), ..
    }) => clip.radius,
    _ if camera_only => Some(SceneRadius {
      camera: Some(0.0),
      ..clip.radius.unwrap_or_default()
    }),
    _ => clip.radius,
  };
  Some(RecordingSceneClip {
    boxes: Some(SceneBoxes {
      screen: share(screen),
      camera: camera_box.map(share),
      camera_behind: false,
      screen_hidden: camera_only,
      camera_hidden: clip.preset == RecordingScenePreset::ScreenOnly,
    }),
    camera: framing,
    radius,
    // A pane moved by hand keeps its place, so a pane turned back on stays
    // where this scene puts it rather than going back to a layout.
    layout_before: None,
    shapes: None,
    ..clip.clone()
  })
}
