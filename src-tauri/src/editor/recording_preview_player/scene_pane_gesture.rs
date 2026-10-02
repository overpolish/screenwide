// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The select gestures that edit the scene under the playhead pane by pane:
//! a custom scene's boxes moved and resized, and any scene's corner radii.

use crate::editor::scenes::{RecordingSceneClip, SceneRadius};

/// The index of the clip `source_ms` falls in.
fn clip_at(clips: &[RecordingSceneClip], source_ms: u64) -> Option<usize> {
  clips
    .iter()
    .position(|clip| clip.start_ms <= source_ms && source_ms < clip.end_ms)
}

/// A select gesture on a pane of a custom scene at `source_ms`: the clips with
/// that pane's box moved or resized. `None` where the clip there is no custom
/// scene; `Some(None)` for a pane its boxes do not place, whose gestures stay
/// the recording's own. `delta` is in shares of the canvas, as the boxes are.
pub(super) fn custom_box_gesture(
  clips: &[RecordingSceneClip],
  source_ms: u64,
  bake_camera: bool,
  layer_id: u32,
  (delta, scale): ((f64, f64), f64),
) -> Option<Option<Vec<RecordingSceneClip>>> {
  let index = clip_at(clips, source_ms)?;
  let boxes = clips[index].boxes?;
  let mut next = clips.to_vec();
  let target = next[index].boxes.as_mut()?;
  match (layer_id, boxes.camera) {
    (0, _) => target.screen = boxes.screen.moved(delta, scale),
    (1, Some(camera)) if bake_camera => target.camera = Some(camera.moved(delta, scale)),
    _ => return Some(None),
  }
  Some(Some(next))
}

/// A corner radius dragged on a pane of the scene at `source_ms`: the clips
/// with that scene's radius for the pane set to `radius` percent, held to
/// what a radius can be. `None` for a pane no scene places, the camera as a
/// pane of its own, whose radius stays the recording's.
pub(super) fn radius_gesture(
  clips: &[RecordingSceneClip],
  source_ms: u64,
  bake_camera: bool,
  layer_id: u32,
  radius: f64,
) -> Option<Vec<RecordingSceneClip>> {
  let index = clip_at(clips, source_ms)?;
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
