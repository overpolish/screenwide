// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The select gestures that edit the scene under the playhead pane by pane:
//! a scene's boxes moved and resized, a preset made custom by the first
//! move, and the screen turned with the rotation gizmo.

use super::{PreviewCompositionSettings, PreviewPlayerManager};
use screenwide_export::scenes::{camera_target, made_custom};
use screenwide_scenes::{RecordingSceneClip, SceneRotation};
use screenwide_workspace::editing::SelectionGestureOperation;

/// The index of the clip `timeline_ms` falls in.
fn clip_at(clips: &[RecordingSceneClip], timeline_ms: u64) -> Option<usize> {
  clips
    .iter()
    .position(|clip| clip.start_ms <= timeline_ms && timeline_ms < clip.end_ms)
}

/// A select gesture on a pane of a custom scene `timeline_ms` into the
/// edited video: the clips with that pane's box moved or resized. A preset
/// scene's screen moves too: the gesture first makes the scene custom where
/// it stands (`custom`, as the panel's Custom does), so nothing jumps as it
/// starts. A preset's camera is left to pan and zoom inside its box. `None`
/// where the clip there is neither; `Some(None)` for a pane its boxes do not
/// place, whose gestures stay the recording's own. `delta` is in shares of
/// the `canvas`, as the boxes are; the screen's is measured on its box padded
/// by the scene's inset, where its handles are drawn.
pub(super) fn custom_box_gesture(
  clips: &[RecordingSceneClip],
  timeline_ms: u64,
  (bake_camera, custom): (
    bool,
    impl FnOnce(&RecordingSceneClip) -> Option<RecordingSceneClip>,
  ),
  layer_id: u32,
  (delta, scale, canvas): ((f64, f64), f64, (f64, f64)),
) -> Option<Option<Vec<RecordingSceneClip>>> {
  let index = clip_at(clips, timeline_ms)?;
  let mut next = clips.to_vec();
  if next[index].boxes.is_none() {
    if layer_id != 0 {
      return None;
    }
    next[index] = custom(&clips[index])?;
  }
  let clip = &next[index];
  let inset = clip.inset.unwrap_or(0.0);
  let target = next[index].boxes.as_mut()?;
  match (layer_id, target.camera) {
    (0, _) => {
      let screen = target.screen;
      let inset = inset * (screen.width * canvas.0).min(screen.height * canvas.1);
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
    if let Some(next) = custom_box_gesture(
      clips,
      timeline_ms,
      (snapshot.bake_camera, custom),
      layer_id,
      (delta, scale, canvas),
    ) {
      return next.map(Some);
    }
    // With the camera a pane of its own, no scene places it, so its gestures
    // stay its own; the screen's are swallowed.
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
