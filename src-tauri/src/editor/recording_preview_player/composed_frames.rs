// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Frames of the open recording composed as the live preview draws them and
//! read back into memory: the clipboard's frame, the project's preview, and
//! its scrub strip.

use screenwide_annotations::timing::RecordingAnnotationClip;
use tauri::AppHandle;

use super::{composed_frame::composed_frame_image, sources::sources_with_surface};
use screenwide_cursor_effects::CursorEffectSettings;
use screenwide_export::recording_model::RecordingOutputSettings;
use screenwide_imaging::CapturedImage;
use screenwide_keyboard_effects::KeyboardEffectSettings;
use screenwide_scenes::CameraOverlaySettings;

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
  tauri::async_runtime::spawn_blocking(move || {
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
          &scenes,
          AnnotationTrack::Primary,
          &ranges,
          position_ms,
          0.0,
          primary,
        );
        output.camera.annotations = revealed_annotations(
          clips,
          &scenes,
          AnnotationTrack::Camera,
          &ranges,
          position_ms,
          0.0,
          camera,
        );
      }
      composed_frame_image(
        &sources,
        position_ms,
        bake_camera,
        camera_overlay,
        cursor_effects,
        keyboard_effects,
        &output,
      )
    };
    // One frame at a time: each opens its own screen and camera decoders,
    // and four at once held about 2 GB of decoder surfaces for a 3.5K
    // recording. The still compositor draws one frame at a time anyway.
    positions_ms
      .iter()
      .map(|&position| compose_at(position).and_then(|frame| shrunk(frame, max_width)))
      .collect::<Result<Vec<_>, String>>()
  })
  .await
  .map_err(|error| error.to_string())?
}

/// `frame` at most `max_width` wide, its proportions kept.
fn shrunk(frame: CapturedImage, max_width: Option<u32>) -> Result<CapturedImage, String> {
  let Some(width) = max_width.filter(|width| *width < frame.width) else {
    return Ok(frame);
  };
  let height =
    (u64::from(frame.height) * u64::from(width) / u64::from(frame.width.max(1))).max(1) as u32;
  let picture = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
    .ok_or_else(|| "A composed frame is not the size it says".to_owned())?;
  let smaller = image::imageops::resize(
    &picture,
    width,
    height,
    image::imageops::FilterType::Triangle,
  );
  Ok(CapturedImage {
    rgba: smaller.into_raw(),
    width,
    height,
  })
}
