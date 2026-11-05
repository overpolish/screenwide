// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The arrangement an exported frame is drawn in, resolved the way the
//! preview resolves it.

use super::CursorExportRequest;
use crate::media_preview::BakedVideoExportOptions;
use crate::scenes;
use screenwide_annotations::timing::{covers, cursor_in_scenes, video_ms, FrameTime};
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::{FacePath, RecordingSceneClip};
use screenwide_timeline::TimelineRange;
use std::sync::Arc;

/// The export's scene clips, the baked camera they move where there is one,
/// and where the face is over that camera where the camera or a scene
/// follows it. A scene that places a camera stands idle without one, while a
/// full scene still frames the screen.
pub(super) struct ExportScenes {
  camera: Option<BakedVideoExportOptions>,
  clips: Vec<RecordingSceneClip>,
  ranges: Vec<TimelineRange>,
  face_path: Option<Arc<FacePath>>,
}

impl ExportScenes {
  pub(super) fn for_request(request: &CursorExportRequest<'_>) -> Self {
    let clips = request
      .timeline
      .map_or_else(Vec::new, |timeline| timeline.scene_clips.clone());
    let follows = request
      .camera
      .is_some_and(|(_, options)| options.overlay.follow_face)
      || clips.iter().any(|clip| clip.follow_face == Some(true));
    Self {
      // The camera is placed on the canvas it was measured against, which a
      // scaled export draws at another size; it is carried onto the drawn
      // canvas first, so both panes are arranged in the same pixels.
      camera: super::export_output::camera_on_canvas(request),
      ranges: request
        .timeline
        .map_or(&[][..], |timeline| timeline.ranges())
        .to_vec(),
      face_path: request
        .camera
        .filter(|_| follows)
        .and_then(|(path, _)| scenes::face_track::load(path)),
      clips,
    }
  }

  /// Whether the recording's own camera follows the face, which moves it on
  /// every frame, outside the scenes too.
  fn camera_follows(&self) -> bool {
    self.face_path.is_some()
      && self
        .camera
        .is_some_and(|options| options.overlay.follow_face)
  }

  /// How much of the cursor shows on the frame `source_ms` into the
  /// recording, from nothing to whole, `shown` being the recording's own
  /// choice: as the scene there says, faded as it arrives and leaves.
  pub(super) fn cursor_shown(&self, source_ms: u64, shown: bool) -> f32 {
    let timeline_ms = FrameTime::on(&self.ranges, source_ms).timeline_ms;
    cursor_in_scenes(&self.clips, shown, (timeline_ms, video_ms(&self.ranges)))
  }

  /// The frame `source_ms` into the recording as the scenes arrange it: the
  /// screen's settings on the canvas `output` describes, carrying how far a
  /// scene moved it over the frame's `frame_ms` of source time, and the baked
  /// camera's options measured on that same canvas where there is one. `None`
  /// outside every scene, where the request's own composition stands, unless
  /// the camera follows the face there.
  pub(super) fn at(
    &self,
    output: &ScreenshotOutputSettings,
    source_ms: u64,
    frame_ms: f32,
  ) -> Option<(ScreenshotOutputSettings, Option<BakedVideoExportOptions>)> {
    let timeline_ms = FrameTime::on(&self.ranges, source_ms).timeline_ms;
    let video_ms = video_ms(&self.ranges);
    if !self.camera_follows()
      && !self
        .clips
        .iter()
        .any(|clip| covers(video_ms, [clip.start_ms, clip.end_ms], timeline_ms))
    {
      return None;
    }
    let mut camera = self.camera;
    let mut arranged = output.clone();
    let placed = scenes::arrange_following(
      &self.clips,
      &self.ranges,
      (source_ms, frame_ms),
      &mut arranged,
      camera.as_mut().map(|options| {
        (
          &mut options.overlay,
          (options.camera_width, options.camera_height),
        )
      }),
      self
        .face_path
        .as_ref()
        .map(|path| path.at(source_ms as f64)),
    );
    (placed || self.camera_follows()).then_some((arranged, camera))
  }
}
