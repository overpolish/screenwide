// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotations an exported frame draws, resolved on the CPU for the
//! shared compositor exactly as the preview resolves them.

use super::CursorExportRequest;
use crate::editor::annotations::camera_baked::baked_camera_annotations;
use crate::editor::annotations::spotlight::handoff::{drawn_until, spotlight_links};
use crate::editor::annotations::timing::{
  revealed_annotations, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationKind};
use crate::editor::media_preview::BakedVideoExportOptions;
use crate::editor::preview_platform::compositor::KindMask;
use crate::editor::recording_preview_player::{held_surfaces, pin_paths};
use crate::editor::timeline_edit::TimelineRange;
use crate::screenshots::ScreenshotOutputSettings;

/// The export's annotation clips, ready to be revealed frame by frame, and
/// what their reveal is measured against.
pub(super) struct ExportAnnotations {
  clips: Vec<RecordingAnnotationClip>,
  track: AnnotationTrack,
  ranges: Vec<TimelineRange>,
  picture: (u32, u32),
  /// Where the camera is drawn into the screen's video, in One video: its
  /// annotations are drawn over it there.
  baked_camera: Option<BakedVideoExportOptions>,
}

impl ExportAnnotations {
  pub(super) fn for_request(request: &CursorExportRequest<'_>) -> Self {
    let mut clips = request
      .timeline
      .map_or(&[][..], |timeline| timeline.annotation_clips())
      .to_vec();
    // Pinned annotations take their paths, which the preview has normally
    // worked out already. The redactions then take the fills read from their
    // clips' frames: a secure pixelation's zones from its first, and the
    // surface across it, both where the pin carries the box, each read from
    // its own picture's recording. Both come before the stroke is scaled
    // below: the preview placed counters' and text boxes' tips at the edited
    // scale, so the export asks for the same paths.
    if request.annotation_track == AnnotationTrack::Primary {
      pin_paths::attach_for_export(
        request.screen,
        request.duration_ms,
        &mut clips,
        request.output.size_image_width(),
        request.cancelled,
      );
    }
    held_surfaces::attach_for_export(
      (request.screen, request.annotation_track),
      request.duration_ms,
      &mut clips,
      request.output.capture_width_points,
    );
    // A baked camera is captured one pixel to the point.
    if let Some((path, options)) = request
      .camera
      .filter(|_| request.annotation_track == AnnotationTrack::Primary)
    {
      held_surfaces::attach_for_export(
        (path, AnnotationTrack::Camera),
        request.duration_ms,
        &mut clips,
        f64::from(options.camera_width),
      );
    }
    // Annotations are authored against the source at its own scale while the
    // stroke follows the output. Scaled once rather than per frame. A
    // redaction's width is its block, which covers the source rather than
    // drawing on the output, so it keeps its size in source pixels. A baked
    // camera's annotations are drawn into its own picture, or carried over it
    // at the size the camera is drawn, so they take no output scale here.
    for clip in clips
      .iter_mut()
      .filter(|clip| clip.track_id == request.annotation_track)
    {
      if clip.annotation.shape.kind() != AnnotationKind::Redact {
        clip.annotation.style.width *= f64::from(request.video.resolution_scale_percent)
          / f64::from(request.video.source_scale_percent.max(1));
      }
    }
    Self {
      clips,
      track: request.annotation_track,
      ranges: request
        .timeline
        .map_or(&[][..], |timeline| timeline.ranges())
        .to_vec(),
      picture: (request.width, request.height),
      baked_camera: request
        .camera
        .filter(|_| request.annotation_track == AnnotationTrack::Primary)
        .map(|(_, options)| options),
    }
  }

  /// What the frame `position_ms` into the source draws, revealed as far as
  /// it has arrived on the edited timeline. `window_ms` is how much source
  /// time the frame covers, which a moving annotation smears over.
  pub(super) fn at(&self, position_ms: u64, window_ms: f32) -> Vec<Annotation> {
    let mut annotations = revealed_annotations(
      &self.clips,
      self.track,
      &self.ranges,
      position_ms,
      window_ms,
      self.picture,
    );
    held_surfaces::resolve_surfaces(&mut annotations, &self.clips, position_ms);
    annotations
  }

  /// [`Self::at`], with the camera's annotations where the camera is drawn
  /// into this video: those over it carried into the screen's list, and
  /// those composed into its picture beside it. `screen` is the output the
  /// frame is drawn with, and `scene_camera` where a scene puts the camera
  /// this frame, if one does.
  pub(super) fn with_camera_at(
    &self,
    position_ms: u64,
    window_ms: f32,
    screen: &ScreenshotOutputSettings,
    scene_camera: Option<BakedVideoExportOptions>,
  ) -> FrameAnnotations {
    let mut annotations = self.at(position_ms, window_ms);
    let Some(camera) = scene_camera.or(self.baked_camera) else {
      return FrameAnnotations {
        screen: annotations,
        camera: None,
      };
    };
    let picture = (camera.camera_width, camera.camera_height);
    let mut on_camera = revealed_annotations(
      &self.clips,
      AnnotationTrack::Camera,
      &self.ranges,
      position_ms,
      window_ms,
      picture,
    );
    held_surfaces::resolve_surfaces(&mut on_camera, &self.clips, position_ms);
    let split = baked_camera_annotations(
      &on_camera,
      picture,
      camera.overlay,
      (
        f64::from(camera.screen_width),
        f64::from(camera.screen_height),
      ),
      screen,
      self.picture,
    );
    annotations.extend(split.over);
    FrameAnnotations {
      screen: annotations,
      // A camera is captured one point to its pixel, as its own file has it.
      camera: (!split.within.is_empty()).then(|| {
        (
          ScreenshotOutputSettings {
            annotations: split.within,
            capture_scale: 1.0,
            capture_width_points: f64::from(camera.camera_width),
            ..screen.clone()
          },
          picture,
        )
      }),
    }
  }

  /// Every set of annotation kinds whose clips are drawn together somewhere,
  /// on either track. A frame draws at most the clips running at its moment,
  /// so one of these holds every kind it shows, and the canvas pipelines
  /// compiled for them leave no frame waiting.
  pub(super) fn kind_sets(&self) -> Vec<KindMask> {
    let links = spotlight_links(&self.clips);
    let mut edges: Vec<(u64, bool, AnnotationKind)> = self
      .clips
      .iter()
      .enumerate()
      .flat_map(|(index, clip)| {
        // A spotlight handing to the next is drawn on until it takes over,
        // as the reveal draws it.
        let end = drawn_until(&self.clips, &links, index);
        let kind = clip.annotation.shape.kind();
        (end > clip.start_ms)
          .then_some([(clip.start_ms, true, kind), (end, false, kind)])
          .into_iter()
          .flatten()
      })
      .collect();
    // At one moment, clips that end there go before those that start: the
    // two are never drawn in the same frame.
    edges.sort_by_key(|&(at, starts, _)| (at, starts));
    let mut running = [0_u32; KindMask::BITS as usize];
    let mut sets = Vec::new();
    for (index, &(at, starts, kind)) in edges.iter().enumerate() {
      let count = &mut running[kind.raw() as usize];
      *count = if starts {
        *count + 1
      } else {
        count.saturating_sub(1)
      };
      if edges.get(index + 1).is_some_and(|next| next.0 == at) {
        continue;
      }
      let kinds = running
        .iter()
        .zip(0..)
        .filter(|(count, _)| **count > 0)
        .fold(0, |kinds, (_, bit)| kinds | 1 << bit);
      if kinds != 0 && !sets.contains(&kinds) {
        sets.push(kinds);
      }
    }
    sets
  }
}

/// What an exported frame draws: the screen's annotations, and with a baked
/// camera that has any drawn into its picture, the camera's own settings
/// carrying them and the camera's source size, which they are placed in.
pub(super) struct FrameAnnotations {
  pub(super) screen: Vec<Annotation>,
  pub(super) camera: Option<(ScreenshotOutputSettings, (u32, u32))>,
}
