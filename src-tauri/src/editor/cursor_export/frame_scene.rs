// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The arrangement an exported frame is drawn in, resolved the way the
//! preview resolves it.

use super::CursorExportRequest;
use crate::editor::annotations::timing::covers;
use crate::editor::media_preview::{BakedVideoExportOptions, VideoExportOptions};
use crate::editor::scenes::{self, RecordingSceneClip};
use crate::editor::timeline_edit::TimelineRange;
use crate::screenshots::ScreenshotOutputSettings;

/// The export's scene clips, and the baked camera they move where there is
/// one. A scene that places a camera stands idle without one, while a full
/// scene still frames the screen.
pub(super) struct ExportScenes {
  camera: Option<BakedVideoExportOptions>,
  clips: Vec<RecordingSceneClip>,
  ranges: Vec<TimelineRange>,
}

impl ExportScenes {
  pub(super) fn for_request(request: &CursorExportRequest<'_>) -> Self {
    Self {
      camera: request.camera.map(|(_, options)| options),
      clips: request
        .timeline
        .map_or_else(Vec::new, |timeline| timeline.scene_clips.clone()),
      ranges: request
        .timeline
        .map_or(&[][..], |timeline| timeline.ranges())
        .to_vec(),
    }
  }

  /// The frame `source_ms` into the recording as the scenes arrange it: the
  /// screen's settings on the canvas `output` describes, carrying how far a
  /// scene moved it over the frame's `frame_ms` of source time, and the baked
  /// camera's options measured on that same canvas where there is one. `None`
  /// outside every scene, where the request's own composition stands.
  pub(super) fn at(
    &self,
    output: &ScreenshotOutputSettings,
    source_ms: u64,
    frame_ms: f32,
  ) -> Option<(ScreenshotOutputSettings, Option<BakedVideoExportOptions>)> {
    if !self
      .clips
      .iter()
      .any(|clip| covers(&self.ranges, [clip.start_ms, clip.end_ms], source_ms))
    {
      return None;
    }
    // The camera is placed on the canvas it was measured against, which a
    // scaled export draws at another size; it is carried onto this canvas
    // first, so both panes are arranged in the same pixels.
    let mut camera = self.camera.map(|options| {
      let sx = f64::from(output.width) / f64::from(options.screen_width.max(1));
      let sy = f64::from(output.height) / f64::from(options.screen_height.max(1));
      let mut overlay = options.overlay;
      overlay.camera_x *= sx;
      overlay.camera_y *= sy;
      overlay.camera_width *= sx;
      overlay.frame_x *= sx;
      overlay.frame_y *= sy;
      overlay.frame_width *= sx;
      overlay.frame_height *= sy;
      BakedVideoExportOptions {
        overlay,
        screen_height: output.height,
        screen_width: output.width,
        video: VideoExportOptions {
          resolution_scale_percent: 100,
          source_scale_percent: 100,
          ..options.video
        },
        ..options
      }
    });
    let mut arranged = output.clone();
    let placed = scenes::arrange(
      &self.clips,
      &self.ranges,
      source_ms,
      frame_ms,
      &mut arranged,
      camera.as_mut().map(|options| {
        (
          &mut options.overlay,
          (options.camera_width, options.camera_height),
        )
      }),
    );
    placed.then_some((arranged, camera))
  }
}
