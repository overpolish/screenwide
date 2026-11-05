// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An annotation's own place in a scene: moved while the playhead is in that
//! scene, it is drawn there for the scene alone and glides there and back as
//! the scene arrives and leaves. Only the shape's geometry is the scene's -
//! where it sits, how big it is, which way it turns and how far a lens zooms.
//! What it says and is (a text's words and look, a counter's number, an
//! image's picture, a stroke's seed) stays the annotation's own, so editing
//! it anywhere edits it in every scene. `recording-annotation-scenes.ts` is
//! the twin that splits island edits the same way.

use screenwide_scenes::RecordingSceneClip;
use screenwide_timeline::TimelineRange;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::timing::{video_ms, AnnotationTrack, FrameTime, RecordingAnnotationClip};
use super::{Annotation, AnnotationShape};

mod geometry;
mod glide;
pub use glide::SceneMoment;

/// The shape an annotation takes in the scene `scene_id`, in place of its
/// own geometry there.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SceneShape {
  pub scene_id: String,
  pub shape: AnnotationShape,
}

/// Where an edit on the picture is made: the scene clips, where the frame
/// stands among them, and the scene the playhead is in, where an edit to an
/// annotation's geometry is that scene's.
pub struct SceneEdit {
  scenes: Vec<RecordingSceneClip>,
  timeline_ms: u64,
  video_ms: Option<u64>,
  /// The screen's picture and the camera's, in source pixels.
  pictures: [(u32, u32); 2],
  scene_id: Option<String>,
}

impl SceneEdit {
  /// An edit made on the frame `source_ms` into the recording, among
  /// `scenes` timed on the timeline `ranges` lay out, over the screen's and
  /// the camera's `pictures`.
  pub fn at(
    scenes: Vec<RecordingSceneClip>,
    ranges: &[TimelineRange],
    source_ms: u64,
    pictures: [(u32, u32); 2],
  ) -> Self {
    let timeline_ms = FrameTime::on(ranges, source_ms).timeline_ms;
    let scene_id = scenes
      .iter()
      .find(|scene| scene.start_ms <= timeline_ms && timeline_ms < scene.end_ms)
      .map(|scene| scene.id.clone());
    Self {
      scenes,
      timeline_ms,
      video_ms: video_ms(ranges),
      pictures,
      scene_id,
    }
  }

  /// Whether the playhead is in a scene, where an edit's geometry is that
  /// scene's.
  pub fn is_in_scene(&self) -> bool {
    self.scene_id.is_some()
  }

  /// Writes `shown` - the annotation as an edit left it at `ms` into the
  /// recording - back into `clip`, as `fold_in_scene` says.
  pub fn fold(&self, clip: &mut RecordingAnnotationClip, shown: &Annotation, ms: u64) {
    let picture = match clip.track_id {
      AnnotationTrack::Primary => self.pictures[0],
      AnnotationTrack::Camera => self.pictures[1],
    };
    let moment = SceneMoment {
      timeline_ms: self.timeline_ms,
      video_ms: self.video_ms,
      picture,
    };
    let drawn = clip
      .shape_in_scenes(&self.scenes, moment)
      .unwrap_or_else(|| clip.annotation.shape.clone());
    fold_in_scene(
      clip,
      shown,
      ms,
      self.scene_id.as_deref().map(|id| (id, &drawn)),
    );
  }
}

impl RecordingAnnotationClip {
  /// The shape the annotation takes in the scene `scene_id`: its own, with
  /// the geometry it was given there, if any.
  pub fn shape_in_scene(&self, scene_id: &str) -> Option<AnnotationShape> {
    self
      .scene_shapes
      .iter()
      .find(|scene| scene.scene_id == scene_id)
      .map(|scene| self.annotation.shape.with_geometry_of(&scene.shape))
  }
}

/// Writes `shown` - the annotation as an edit left it at `ms` into the
/// recording - back into `clip`, the playhead being in the scene `scene_id`
/// if any, where the frame drew the annotation as `drawn`. Inside a scene an
/// edit that moves the annotation is that scene's: its geometry becomes the
/// scene's shape, and the rest of the edit, its words or its look, the
/// annotation's own. An edit that leaves the geometry as drawn gives the
/// scene nothing, so choosing an annotation, or retyping its words, does not
/// pin it there. Outside every scene, or for a pinned annotation, which
/// follows its content, it is written as before (`pin::fold`).
fn fold_in_scene(
  clip: &mut RecordingAnnotationClip,
  shown: &Annotation,
  ms: u64,
  scene: Option<(&str, &AnnotationShape)>,
) {
  let Some((scene_id, drawn)) = scene.filter(|_| clip.pin.is_none()) else {
    super::pin::fold(clip, shown, ms);
    return;
  };
  let moved = drawn.with_geometry_of(&shown.shape) != *drawn;
  let own = Annotation {
    shape: shown.shape.with_geometry_of(&clip.annotation.shape),
    ..shown.clone()
  };
  clip.annotation = own;
  if !moved {
    return;
  }
  let shape = shown.shape.clone();
  match clip
    .scene_shapes
    .iter_mut()
    .find(|scene| scene.scene_id == scene_id)
  {
    Some(scene) => scene.shape = shape,
    None => clip.scene_shapes.push(SceneShape {
      scene_id: scene_id.to_owned(),
      shape,
    }),
  }
}

#[cfg(test)]
mod tests;
