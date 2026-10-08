// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Frames of the open recording composed as the live preview draws them and
//! read back into memory: the clipboard's frame, the project's preview, and
//! its scrub strip.

use tauri::AppHandle;

use super::{platform, sources::sources_with_surface};
use crate::editor::{
  annotations::timing::RecordingAnnotationClip, cursor_effects::CursorEffectSettings,
  keyboard_effects::KeyboardEffectSettings, CameraOverlaySettings, RecordingOutputSettings,
};
use crate::screenshots::CapturedImage;

/// macOS composes this many frames at once: each decodes on its own and the
/// GPU device takes work from any thread.
#[cfg(target_os = "macos")]
const WORKERS: usize = 4;

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
}

/// Composes `frame` at its own position.
pub(crate) async fn compose_preview_frame(
  app: &AppHandle,
  frame: PreviewFrameRequest,
) -> Result<CapturedImage, String> {
  let position_ms = frame.position_ms;
  compose_preview_frames(app, frame, vec![position_ms])
    .await?
    .pop()
    .ok_or_else(|| "No frame was composed".to_owned())
}

/// Composes `frame` at each of `positions_ms`, in order, from one read of
/// the recording's sources: its cursor, keyboard, pins and annotation
/// pictures are worked out once for the whole set rather than per frame.
pub(crate) async fn compose_preview_frames(
  app: &AppHandle,
  frame: PreviewFrameRequest,
  positions_ms: Vec<u64>,
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
  } = frame;
  // Windows composes the frame through the editor window's compositor, which
  // every session shares; macOS composes it without one.
  let sources = sources_with_surface(app, artifact_id, None, cfg!(target_os = "windows"))?;
  recording_output.stamp_captures(sources.captures);
  if let Some(clips) = &annotation_clips {
    crate::editor::annotations::timing::validate_clips(clips)?;
  }
  tauri::async_runtime::spawn_blocking(move || {
    let clips = annotation_clips.map(|mut clips| {
      // The preview has normally tracked every pin already; one it has not
      // is tracked here, off the async runtime, once for every frame.
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      super::pin_paths::attach_for_export(
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
        use crate::editor::annotations::timing::{revealed_annotations, AnnotationTrack};
        let ranges = sources
          .animation_ranges
          .read()
          .unwrap_or_else(|poisoned| poisoned.into_inner());
        // A frame standing still shows the reveal this instant holds, with
        // nothing moving for the blur to fade.
        let [primary, camera] = sources.annotation_pictures();
        output.primary.annotations = revealed_annotations(
          clips,
          AnnotationTrack::Primary,
          &ranges,
          position_ms,
          0.0,
          primary,
        );
        output.camera.annotations = revealed_annotations(
          clips,
          AnnotationTrack::Camera,
          &ranges,
          position_ms,
          0.0,
          camera,
        );
      }
      platform::composed_frame_image(
        &sources,
        position_ms,
        bake_camera,
        camera_overlay,
        cursor_effects,
        keyboard_effects,
        &output,
      )
    };
    compose_spread(&positions_ms, &compose_at)
  })
  .await
  .map_err(|error| error.to_string())?
}

/// Runs `compose_at` over `positions_ms` on up to [`WORKERS`] threads, each
/// taking every [`WORKERS`]th position, and returns the frames in order.
#[cfg(target_os = "macos")]
fn compose_spread(
  positions_ms: &[u64],
  compose_at: &(dyn Fn(u64) -> Result<CapturedImage, String> + Sync),
) -> Result<Vec<CapturedImage>, String> {
  if positions_ms.len() <= 1 {
    return positions_ms
      .iter()
      .map(|&position| compose_at(position))
      .collect();
  }
  let mut slots: Vec<Option<CapturedImage>> = positions_ms.iter().map(|_| None).collect();
  std::thread::scope(|scope| {
    let workers: Vec<_> = (0..WORKERS)
      .map(|worker| {
        scope.spawn(move || {
          positions_ms
            .iter()
            .enumerate()
            .skip(worker)
            .step_by(WORKERS)
            .map(|(index, &position)| compose_at(position).map(|frame| (index, frame)))
            .collect::<Result<Vec<_>, String>>()
        })
      })
      .collect();
    for worker in workers {
      let composed = worker
        .join()
        .map_err(|_| "A frame could not be composed".to_owned())??;
      for (index, frame) in composed {
        slots[index] = Some(frame);
      }
    }
    Ok::<(), String>(())
  })?;
  slots
    .into_iter()
    .map(|slot| slot.ok_or_else(|| "A frame was not composed".to_owned()))
    .collect()
}

/// Runs `compose_at` over `positions_ms` one at a time, in order. Windows
/// composes through the editor window's one shared surface, so frames cannot
/// be composed in parallel.
#[cfg(not(target_os = "macos"))]
fn compose_spread(
  positions_ms: &[u64],
  compose_at: &dyn Fn(u64) -> Result<CapturedImage, String>,
) -> Result<Vec<CapturedImage>, String> {
  positions_ms
    .iter()
    .map(|&position| compose_at(position))
    .collect()
}
