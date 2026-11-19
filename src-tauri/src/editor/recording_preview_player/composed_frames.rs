// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Frames of the open recording composed as the live preview draws them and
//! read back into memory: the clipboard's frame, the project's preview, and
//! its scrub strip.

use screenwide_annotations::timing::RecordingAnnotationClip;
use tauri::AppHandle;

use super::{
  composed_frame::composed_frame_image,
  sources::{sources_with_surface, PlayerSources},
};
use screenwide_cursor_effects::CursorEffectSettings;
use screenwide_export::recording_model::RecordingOutputSettings;
use screenwide_imaging::CapturedImage;
use screenwide_keyboard_effects::KeyboardEffectSettings;
use screenwide_scenes::CameraOverlaySettings;
use screenwide_timeline::TimelineRange;

/// One composed frame of the open recording: the preview's settings at
/// `position_ms` in the source.
#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewFrameRequest {
  pub(crate) artifact_id: u64,
  position_ms: u64,
  bake_camera: bool,
  camera_overlay: CameraOverlaySettings,
  cursor_effects: CursorEffectSettings,
  keyboard_effects: KeyboardEffectSettings,
  recording_output: RecordingOutputSettings,
  annotation_clips: Option<Vec<RecordingAnnotationClip>>,
  /// The timeline `annotation_clips` are timed on; the saved edit's when
  /// absent.
  playback_ranges: Option<Vec<super::RecordingPreviewPlaybackRange>>,
}

/// Composes `frame` at its own position.
pub(crate) async fn compose_preview_frame(
  app: &AppHandle,
  frame: PreviewFrameRequest,
) -> Result<CapturedImage, String> {
  let position_ms = frame.position_ms;
  compose_preview_frames(app, frame, vec![position_ms], None)
    .await?
    .pop()
    .ok_or_else(|| "No frame was composed".to_owned())
}

/// Composes `frame` at each of `positions_ms`, in order, from one read of
/// the recording's sources: its cursor, keyboard, pins and annotation
/// pictures are worked out once for the whole set rather than per frame.
/// With `max_width`, each frame is shrunk to that width as soon as it is
/// composed, so only one full-size frame is ever held.
pub(crate) async fn compose_preview_frames(
  app: &AppHandle,
  frame: PreviewFrameRequest,
  positions_ms: Vec<u64>,
  max_width: Option<u32>,
) -> Result<Vec<CapturedImage>, String> {
  let PreviewFrameRequest {
    artifact_id,
    position_ms: _,
    bake_camera,
    camera_overlay,
    cursor_effects,
    keyboard_effects,
    mut recording_output,
    annotation_clips,
    playback_ranges,
  } = frame;
  // Windows composes the frame through the editor window's compositor, which
  // every session shares; macOS composes it without one.
  let sources = sources_with_surface(app, artifact_id, None, cfg!(target_os = "windows"))?;
  recording_output.stamp_captures(sources.captures);
  if let Some(clips) = &annotation_clips {
    screenwide_annotations::timing::validate_clips(clips)?;
  }
  let ranges = playback_ranges.map_or_else(
    || {
      sources
        .annotation_ranges
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
    },
    |ranges| super::animation_timeline_ranges(&ranges),
  );
  let look = FrameLook {
    bake_camera,
    camera_overlay,
    cursor_effects,
    keyboard_effects,
    recording_output,
  };
  compose_from(
    sources,
    look,
    annotation_clips,
    ranges,
    positions_ms,
    max_width,
  )
  .await
}

/// How a frame is drawn: the editor's settings, sent by the window or saved
/// in the project.
pub(super) struct FrameLook {
  pub(super) bake_camera: bool,
  pub(super) camera_overlay: CameraOverlaySettings,
  pub(super) cursor_effects: CursorEffectSettings,
  pub(super) keyboard_effects: KeyboardEffectSettings,
  pub(super) recording_output: RecordingOutputSettings,
}

/// Composes `sources` drawn with `look` at each of `positions_ms`, in the
/// source, with `annotation_clips` timed on `ranges`.
pub(super) async fn compose_from(
  sources: PlayerSources,
  look: FrameLook,
  annotation_clips: Option<Vec<RecordingAnnotationClip>>,
  ranges: Vec<TimelineRange>,
  positions_ms: Vec<u64>,
  max_width: Option<u32>,
) -> Result<Vec<CapturedImage>, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let recording_output = &look.recording_output;
    let clips = annotation_clips.map(|mut clips| {
      // What follows the picture under a clip is read across the stretch of
      // the recording it plays.
      screenwide_annotations::timing::lay_on_recording(&mut clips, &ranges);
      // The preview has normally tracked every pin already; one it has not
      // is tracked here, off the async runtime, once for every frame.
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      screenwide_export::pin_paths::attach_for_export(
        &sources.screen_path,
        sources.duration_ms,
        &mut clips,
        recording_output.primary.size_image_width(),
        &std::sync::atomic::AtomicBool::new(false),
      );
      clips
    });
    let compose_at = |position_ms: u64| {
      let position_ms = position_ms.min(sources.duration_ms.saturating_sub(1));
      let mut output = recording_output.clone();
      if let Some(clips) = &clips {
        use screenwide_annotations::timing::{revealed_annotations, AnnotationTrack};
        // A frame standing still shows the reveal this instant holds, with
        // nothing moving for the blur to fade.
        let [primary, camera] = sources.annotation_pictures();
        let scenes = sources
          .scenes
          .clips
          .read()
          .unwrap_or_else(|poisoned| poisoned.into_inner());
        output.primary.annotations = revealed_annotations(
          clips,
          scenes.own(),
          AnnotationTrack::Primary,
          &ranges,
          position_ms,
          0.0,
          primary,
        );
        output.camera.annotations = revealed_annotations(
          clips,
          scenes.own(),
          AnnotationTrack::Camera,
          &ranges,
          position_ms,
          0.0,
          camera,
        );
      }
      composed_frame_image(&sources, position_ms, &look, &output)
    };
    // One frame at a time: each opens its own screen and camera decoders,
    // and four at once held about 2 GB of decoder surfaces for a 3.5K
    // recording. The still compositor draws one frame at a time anyway.
    positions_ms
      .iter()
      .map(|&position| {
        compose_at(position).and_then(|frame| match max_width {
          Some(width) => screenwide_export::still_size::shrunk(frame, width),
          None => Ok(frame),
        })
      })
      .collect::<Result<Vec<_>, String>>()
  })
  .await
  .map_err(|error| error.to_string())?
}
