// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{path::Path, sync::Arc, thread};

use screenwide_export::camera_effects::live::LiveCameraEffects;
use tauri::{image::Image, ipc::Channel, AppHandle};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::{
  composed_frames::{compose_preview_frame, PreviewFrameRequest},
  platform,
  sources::headless_sources,
  PlayerSources,
};
pub(super) const HEADER_MARKER: u32 = u32::from_le_bytes(*b"OCTH");
const HEADER_VERSION: u32 = 1;
const MAX_THUMBNAILS: u32 = 32;
const MIN_THUMBNAILS: u32 = 4;
pub(super) const THUMBNAIL_HEIGHT: u32 = 64;

pub(super) fn payload(track: u32, index: u32, count: u32, jpeg: &[u8]) -> Vec<u8> {
  let mut payload = Vec::with_capacity(20 + jpeg.len());
  payload.extend_from_slice(&HEADER_MARKER.to_le_bytes());
  payload.extend_from_slice(&HEADER_VERSION.to_le_bytes());
  payload.extend_from_slice(&track.to_le_bytes());
  payload.extend_from_slice(&index.to_le_bytes());
  payload.extend_from_slice(&count.to_le_bytes());
  payload.extend_from_slice(jpeg);
  payload
}

pub(super) fn target_width(source_width: u32, source_height: u32) -> u32 {
  if source_width == 0 || source_height == 0 {
    return THUMBNAIL_HEIGHT;
  }
  ((f64::from(source_width) * f64::from(THUMBNAIL_HEIGHT) / f64::from(source_height))
    .round()
    .max(1.0)) as u32
}

/// Where the `index`th of `count` thumbnails is taken in a recording
/// `duration_ms` long: the middle of the stretch of strip it is drawn across,
/// so the picture matches the time under it.
pub(super) fn cell_centre_ms(duration_ms: u64, index: u32, count: u32) -> u64 {
  duration_ms.saturating_mul(2 * u64::from(index) + 1) / (2 * u64::from(count.max(1)))
}

/// How far from its cell's centre a thumbnail may be taken: half a cell, so
/// it stays inside its own cell. The slack lets the decoder return a nearby
/// keyframe instead of decoding forward to an exact frame, several times
/// less work; on macOS it also avoids frames AVFoundation cannot decode
/// exactly, which an exact request leaves as gaps in the strip.
pub(super) fn cell_tolerance_ms(duration_ms: u64, count: u32) -> u64 {
  duration_ms / u64::from(count.max(1)) / 2
}

/// One video's thumbnails: the strip they fill, and the camera's effects
/// where that video is the camera.
pub(super) struct ThumbnailTrack<'a> {
  pub(super) id: u32,
  pub(super) path: &'a Path,
  pub(super) duration_ms: u64,
  pub(super) width: u32,
  pub(super) effects: Option<Arc<LiveCameraEffects>>,
}

/// The screen's thumbnails and, where there is one, the camera's. The
/// camera's show its effects, see-through where it has no background, which
/// only a PNG carries.
pub(super) fn thumbnail_tracks(sources: &PlayerSources) -> Vec<ThumbnailTrack<'_>> {
  let effects = sources.camera_effects_now();
  let camera_only = sources.primary_kind == screenwide_project::PrimaryRecordingKind::Camera;
  let screen = (
    0,
    sources.screen_path.as_path(),
    sources.duration_ms,
    camera_only,
  );
  let camera = sources.camera_path.as_deref().map(|path| {
    let duration_ms = sources.camera_duration_ms.unwrap_or(sources.duration_ms);
    (1, path, duration_ms, !camera_only)
  });
  std::iter::once(screen)
    .chain(camera)
    .filter_map(|(id, path, duration_ms, shows_camera)| {
      let pane = sources.playback_layout.panes.get(id as usize)?;
      Some(ThumbnailTrack {
        id,
        path,
        duration_ms,
        width: target_width(pane.source_width, pane.source_height),
        effects: shows_camera.then(|| effects.clone()).flatten(),
      })
    })
    .collect()
}

/// A see-through thumbnail, as a PNG: a JPEG has no transparency.
pub(super) fn encoded_png(image: &screenwide_imaging::CapturedImage) -> Result<Vec<u8>, String> {
  let rgba = ::image::RgbaImage::from_raw(image.width, image.height, image.rgba.clone())
    .ok_or_else(|| "The thumbnail is not the size it says".to_owned())?;
  let mut bytes = std::io::Cursor::new(Vec::new());
  rgba
    .write_to(&mut bytes, ::image::ImageFormat::Png)
    .map_err(|error| error.to_string())?;
  Ok(bytes.into_inner())
}

/// Copies the composed primary frame at the playhead. Decoding and bitmap
/// composition happen only for this explicit action; live preview stays on
/// the native GPU surface and does not cross IPC.
#[tauri::command]
pub async fn copy_recording_preview_frame_to_clipboard(
  app: AppHandle,
  frame: PreviewFrameRequest,
) -> Result<(), String> {
  let composed = compose_preview_frame(&app, frame).await?;
  app
    .clipboard()
    .write_image(&Image::new(&composed.rgba, composed.width, composed.height))
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn stream_recording_timeline_thumbnails(
  app: AppHandle,
  artifact_id: u64,
  count: u32,
  channel: Channel,
) -> Result<(), String> {
  let sources = headless_sources(&app, artifact_id)?;
  let count = count.clamp(MIN_THUMBNAILS, MAX_THUMBNAILS);
  thread::Builder::new()
    .name("recording-timeline-thumbnails".to_owned())
    .spawn(move || platform::generate_thumbnails(sources, count, channel))
    .map_err(|error| error.to_string())?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn keeps_thumbnail_width_in_source_aspect_ratio() {
    assert_eq!(target_width(1_920, 1_080), 114);
    assert_eq!(target_width(1_080, 1_920), 36);
  }

  #[test]
  fn thumbnails_sit_at_the_centre_of_their_cells() {
    assert_eq!(cell_centre_ms(9_000, 0, 9), 500);
    assert_eq!(cell_centre_ms(9_000, 8, 9), 8_500);
    assert_eq!(cell_tolerance_ms(9_000, 9), 500);
  }

  #[test]
  fn payload_has_a_stable_header() {
    let bytes = payload(1, 2, 20, &[3, 4]);
    assert_eq!(&bytes[..4], b"OCTH");
    assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 1);
    assert_eq!(&bytes[20..], &[3, 4]);
  }
}
