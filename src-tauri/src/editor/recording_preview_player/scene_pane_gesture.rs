// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The select gestures that edit the scene under the playhead pane by pane:
//! a scene's boxes moved and resized, a preset made custom by the first
//! move, and the screen turned with the rotation gizmo.

use super::{PreviewCompositionSettings, PreviewPlayerManager};
use screenwide_export::scenes::made_custom;
use screenwide_scenes::{RecordingSceneClip, RecordingScenePreset, SceneRotation};
use screenwide_workspace::editing::SelectionGestureOperation;

/// The index of the clip `timeline_ms` falls in.
fn clip_at(clips: &[RecordingSceneClip], timeline_ms: u64) -> Option<usize> {
  clips
    .iter()
    .position(|clip| clip.start_ms <= timeline_ms && timeline_ms < clip.end_ms)
}

/// A select gesture on a pane of a custom scene `timeline_ms` into the
/// edited video: the clips with that pane's box moved or resized. A preset
/// scene's panes move too, the camera's wherever the preset shows it: the
/// gesture first makes the scene custom where it stands (`custom`, as the
/// panel's Custom does), so nothing jumps as it starts. What a pane shows
/// inside its box is the crop tool's. `None` where the clip there is
/// neither; `Some(None)` for a pane its boxes do not place, whose gestures
/// stay the recording's own. `delta` is in shares of the `canvas`, as the
/// boxes are; the screen's is measured on its box padded by the scene's
/// inset, `inset` canvas pixels on every side, where its handles are drawn.
pub(super) fn custom_box_gesture(
  clips: &[RecordingSceneClip],
  timeline_ms: u64,
  (bake_camera, custom): (
    bool,
    impl FnOnce(&RecordingSceneClip) -> Option<RecordingSceneClip>,
  ),
  layer_id: u32,
  (delta, scale, canvas, inset): ((f64, f64), f64, (f64, f64), f64),
) -> Option<Option<Vec<RecordingSceneClip>>> {
  let index = clip_at(clips, timeline_ms)?;
  let mut next = clips.to_vec();
  if next[index].boxes.is_none() {
    let preset = clips[index].preset;
    let frees = match layer_id {
      0 => true,
      1 => bake_camera && (clips[index].needs_camera() || preset == RecordingScenePreset::Full),
      _ => false,
    };
    if !frees {
      return None;
    }
    next[index] = custom(&clips[index])?;
  }
  let target = next[index].boxes.as_mut()?;
  match (layer_id, target.camera) {
    (0, _) => {
      let screen = target.screen;
      target.screen = screen.moved_padded(delta, scale, (inset / canvas.0, inset / canvas.1));
    }
    (1, Some(camera)) if bake_camera => target.camera = Some(camera.moved(delta, scale)),
    _ => return Some(None),
  }
  Some(Some(next))
}

/// The screen turned to `rotation` with the rotation gizmo `timeline_ms` into
/// the edited video: the clips with the scene there turning it so. `None`
/// outside every scene and for any pane but the screen, which is the only one
/// a scene turns.
pub(super) fn rotation_gesture(
  clips: &[RecordingSceneClip],
  timeline_ms: u64,
  layer_id: u32,
  rotation: SceneRotation,
) -> Option<Vec<RecordingSceneClip>> {
  if layer_id != 0 {
    return None;
  }
  let index = clip_at(clips, timeline_ms)?;
  let mut next = clips.to_vec();
  next[index].rotation = Some(rotation).filter(|rotation| !rotation.is_upright());
  Some(next)
}

impl PreviewPlayerManager {
  /// What a gesture does while the paused frame is inside a scene. A custom
  /// scene's boxes move and resize with the panes in them, and a preset
  /// scene's screen moves and resizes as a custom scene's would, making the
  /// scene custom. A preset's camera stays where the preset puts it: dragged,
  /// its picture pans inside its box, and resized, it zooms, which reframes
  /// the camera of the clip under the playhead. A turn of the rotation gizmo
  /// turns the screen in any scene. `clips` are the clips as the gesture
  /// found them, which its deltas are measured from.
  /// `None` outside every scene and for the canvas's frame, which a scene
  /// leaves as it is; `Some(None)` for a gesture a scene leaves with nothing
  /// to do; else the clips the gesture leaves.
  pub(super) fn scene_gesture(
    &self,
    snapshot: &PreviewCompositionSettings,
    clips: &[RecordingSceneClip],
    (layer_id, operation): (u32, SelectionGestureOperation),
    scale: f64,
    delta: (f64, f64),
  ) -> Option<Option<Vec<RecordingSceneClip>>> {
    let scale = match operation {
      SelectionGestureOperation::Move => 1.0,
      SelectionGestureOperation::Resize | SelectionGestureOperation::Rotate => scale,
      _ => return None,
    };
    let sources = self.sources.as_ref()?;
    let mut arranged = snapshot.clone();
    if !sources.arrange_scene(&mut arranged, self.position_ms, 0.0) {
      return None;
    }
    let timeline_ms = sources.scenes.timeline_ms(self.position_ms);
    // A turn reports the turn it reaches: about X and Y as its travel, about
    // Z as its scale.
    if operation == SelectionGestureOperation::Rotate {
      let rotation = SceneRotation {
        x: delta.0,
        y: delta.1,
        z: scale,
      };
      return Some(rotation_gesture(clips, timeline_ms, layer_id, rotation));
    }
    let output = &snapshot.recording_output.primary;
    // The gesture's deltas are shares of the canvas it was made on.
    let canvas = crate::editor::preview_workspace_model::output_canvas(output);
    // The camera drawn into the picture, where it is, with its source size.
    let camera = sources
      .camera_source_size()
      .filter(|_| snapshot.bake_camera);
    let custom = |clip: &RecordingSceneClip| {
      made_custom(
        clip,
        output,
        camera.map(|size| (&snapshot.camera_overlay, size)),
      )
    };
    // How far the scene pads the screen's box, in canvas pixels, as the
    // arranged frame draws it: the padded crop past the box it shows.
    let arranged_screen = &arranged.recording_output.primary;
    let inset = arranged_screen
      .scene_window
      .map_or(0.0, |[_, _, width, _]| {
        ((arranged_screen.crop_width - width) / 2.0).max(0.0)
      });
    if let Some(next) = custom_box_gesture(
      clips,
      timeline_ms,
      (snapshot.bake_camera, custom),
      layer_id,
      (delta, scale, canvas, inset),
    ) {
      return next.map(Some);
    }
    // A pane no scene places keeps its own gestures: the camera left out of
    // the video. Anything else the scene hides is swallowed.
    if camera.is_none() && layer_id == 1 {
      return None;
    }
    Some(None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_scenes::model::{SceneBox, SceneBoxes};

  const CAMERA: SceneBox = SceneBox {
    x: 0.7,
    y: 0.2,
    width: 0.25,
    height: 0.3,
  };

  fn scene(preset: &str) -> Vec<RecordingSceneClip> {
    vec![serde_json::from_value(serde_json::json!({
      "id": "s", "startMs": 0, "endMs": 10_000, "preset": preset
    }))
    .unwrap()]
  }

  /// The scene made custom where it stands, its camera at `CAMERA`.
  fn custom(clip: &RecordingSceneClip) -> Option<RecordingSceneClip> {
    Some(RecordingSceneClip {
      boxes: Some(SceneBoxes {
        screen: SceneBox {
          x: 0.05,
          y: 0.2,
          width: 0.6,
          height: 0.6,
        },
        camera: Some(CAMERA),
        camera_behind: false,
        screen_hidden: false,
        camera_hidden: false,
      }),
      ..clip.clone()
    })
  }

  fn drag_camera(preset: &str, bake_camera: bool) -> Option<Option<Vec<RecordingSceneClip>>> {
    custom_box_gesture(
      &scene(preset),
      5_000,
      (bake_camera, custom),
      1,
      ((0.1, 0.0), 1.0, (1_600.0, 900.0), 0.0),
    )
  }

  #[test]
  fn a_presets_camera_moves_its_box_as_the_screen_does() {
    for preset in ["split-two-thirds", "picture-in-picture", "full"] {
      let moved = drag_camera(preset, true).flatten().expect(preset);
      let camera = moved[0].boxes.and_then(|boxes| boxes.camera).expect(preset);
      assert!((camera.x - (CAMERA.x + 0.1)).abs() < 1e-9, "{preset}");
    }
  }

  #[test]
  fn a_camera_out_of_the_video_or_hidden_is_not_taken() {
    assert_eq!(drag_camera("split-two-thirds", false), None);
    assert_eq!(drag_camera("screen-only", true), None);
  }
}
