// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The select gestures that edit the scene under the playhead pane by pane:
//! a custom scene's boxes moved and resized, any scene's corner radii, and
//! the screen turned with the rotation gizmo.

use super::{PreviewCompositionSettings, PreviewPlayerManager};
use screenwide_export::scenes::camera_target;
use screenwide_scenes::model::{SceneBox, SceneBoxes};
use screenwide_scenes::{RecordingSceneClip, RecordingScenePreset, SceneRadius, SceneRotation};
use screenwide_workspace::editing::SelectionGestureOperation;

/// The index of the clip `timeline_ms` falls in.
fn clip_at(clips: &[RecordingSceneClip], timeline_ms: u64) -> Option<usize> {
  clips
    .iter()
    .position(|clip| clip.start_ms <= timeline_ms && timeline_ms < clip.end_ms)
}

/// A select gesture on a pane of a custom scene `timeline_ms` into the
/// edited video: the clips with that pane's box moved or resized. A full
/// scene stands for the recording's own composition and is offered as
/// Custom, so its screen moves too, from `composition`, the box the
/// composition puts it in, which makes the scene custom as the panel's Custom
/// does. `None` where the clip there is neither; `Some(None)` for a pane its
/// boxes do not place, whose gestures stay the recording's own. `delta` is in
/// shares of the canvas, as the boxes are.
pub(super) fn custom_box_gesture(
  clips: &[RecordingSceneClip],
  timeline_ms: u64,
  (bake_camera, composition): (bool, Option<SceneBox>),
  layer_id: u32,
  (delta, scale): ((f64, f64), f64),
) -> Option<Option<Vec<RecordingSceneClip>>> {
  let index = clip_at(clips, timeline_ms)?;
  let clip = &clips[index];
  let boxes = match (clip.boxes, composition) {
    (Some(boxes), _) => boxes,
    (None, Some(screen)) if clip.preset == RecordingScenePreset::Full && layer_id == 0 => {
      SceneBoxes {
        screen,
        camera: None,
        camera_behind: false,
      }
    }
    _ => return None,
  };
  let mut next = clips.to_vec();
  let target = next[index].boxes.insert(boxes);
  match (layer_id, boxes.camera) {
    (0, _) => target.screen = boxes.screen.moved(delta, scale),
    (1, Some(camera)) if bake_camera => target.camera = Some(camera.moved(delta, scale)),
    _ => return Some(None),
  }
  Some(Some(next))
}

/// A corner radius dragged on a pane of the scene `timeline_ms` into the
/// edited video: the clips with that scene's radius for the pane set to
/// `radius` percent, held to what a radius can be. `None` for a pane no scene
/// places, the camera as a pane of its own, whose radius stays the
/// recording's.
pub(super) fn radius_gesture(
  clips: &[RecordingSceneClip],
  timeline_ms: u64,
  bake_camera: bool,
  layer_id: u32,
  radius: f64,
) -> Option<Vec<RecordingSceneClip>> {
  let index = clip_at(clips, timeline_ms)?;
  let mut next = clips.to_vec();
  let held = Some(radius.clamp(0.0, 50.0));
  let scene = next[index].radius.get_or_insert_with(SceneRadius::default);
  match layer_id {
    0 => scene.screen = held,
    1 if bake_camera => scene.camera = held,
    _ => return None,
  }
  Some(next)
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
  /// scene's boxes move and resize with the panes in them, and a full
  /// scene's screen moves and resizes as a custom scene's would. Any other
  /// scene owns where the panes sit, so a move or a resize only reaches the
  /// camera's picture: dragged, it pans inside its box, and resized, it
  /// zooms, which reframes the camera of the clip under the playhead. A turn
  /// of the rotation gizmo turns the screen in any scene. `clips` are the
  /// clips as the gesture found them, which its deltas are measured from.
  /// `None` outside every scene and for the gestures a scene leaves as they
  /// are, a corner radius or the canvas's frame; `Some(None)` for a gesture a
  /// scene leaves with nothing to do; else the clips the gesture leaves.
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
      SelectionGestureOperation::Resize
      | SelectionGestureOperation::Radius
      | SelectionGestureOperation::Rotate => scale,
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
    // A radius gesture reports the radius it reaches as its scale.
    if operation == SelectionGestureOperation::Radius {
      return radius_gesture(clips, timeline_ms, snapshot.bake_camera, layer_id, scale).map(Some);
    }
    let output = &snapshot.recording_output.primary;
    // The gesture's deltas are shares of the canvas it was made on.
    let canvas = crate::editor::preview_workspace_model::output_canvas(output);
    let composition = (output.crop_width > 0.0 && output.crop_height > 0.0).then(|| SceneBox {
      x: output.crop_x / canvas.0,
      y: output.crop_y / canvas.1,
      width: output.crop_width / canvas.0,
      height: output.crop_height / canvas.1,
    });
    if let Some(next) = custom_box_gesture(
      clips,
      timeline_ms,
      (snapshot.bake_camera, composition),
      layer_id,
      (delta, scale),
    ) {
      return next.map(Some);
    }
    // With the camera a pane of its own, no scene places it, so its gestures
    // stay its own; the screen's are swallowed.
    let camera = sources
      .camera_source_size()
      .filter(|_| snapshot.bake_camera);
    let camera = match (camera, layer_id) {
      (Some(camera), 1) => camera,
      (None, 1) => return None,
      _ => return Some(None),
    };
    let Some((index, frame, framing)) =
      camera_target(clips, timeline_ms, output, &snapshot.camera_overlay, camera)
    else {
      return Some(None);
    };
    let mut next = clips.to_vec();
    next[index].camera = Some(framing.reframed(
      frame,
      f64::from(camera.0) / f64::from(camera.1.max(1)),
      (delta.0 * canvas.0, delta.1 * canvas.1),
      scale,
    ));
    Some(Some(next))
  }
}
