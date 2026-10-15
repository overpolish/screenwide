// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One frame composed as the live preview draws it, read back into memory:
//! what copying the preview's frame puts on the clipboard.

use super::super::super::PlayerSources;
use super::GpuVideoReader;
use screenwide_compositor::compositor::ComposedFrame;
use screenwide_export::camera_effects::matte_frames::MatteFrames;
use screenwide_export::camera_effects::preview::StillCameraEffects;
use screenwide_export::media_preview;

/// The longest side a composed still reads the camera's matte at: stills
/// are copied or shown small, and the edge it draws is soft.
const STILL_MATTE_SIDE: u32 = 1_920;

pub(crate) fn composed_frame_image(
  sources: &PlayerSources,
  position_ms: u64,
  bake_camera: bool,
  camera_overlay: screenwide_scenes::CameraOverlaySettings,
  cursor_effects: screenwide_cursor_effects::CursorEffectSettings,
  keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings,
  recording_output: &screenwide_export::recording_model::RecordingOutputSettings,
) -> Result<screenwide_imaging::CapturedImage, String> {
  let surface = sources
    .preview_surface
    .clone()
    .ok_or_else(|| "Windows GPU preview has no native presentation surface".to_owned())?;
  let position_ms = position_ms.min(sources.duration_ms.saturating_sub(1));
  let mut reader = GpuVideoReader::open(&sources.screen_path, position_ms, surface.device())?;
  let frame = reader
    .frame_at(position_ms)?
    .ok_or_else(|| "Media Foundation returned no source frame".to_owned())?;
  let mut camera_reader = if bake_camera {
    sources
      .camera_path
      .as_ref()
      .map(|path| GpuVideoReader::open(path, position_ms, surface.device()))
      .transpose()?
  } else {
    None
  };
  let camera_frame = camera_reader
    .as_mut()
    .map(|reader| reader.frame_at(position_ms))
    .transpose()?
    .flatten()
    .filter(|frame| frame.timestamp_ms <= position_ms.saturating_add(50));
  let camera_geometry = camera_frame
    .as_ref()
    .map(|camera| {
      media_preview::bake_geometry(media_preview::BakedVideoExportOptions {
        camera_drop_shadow: recording_output.camera.drop_shadow,
        camera_height: camera.height,
        camera_width: camera.width,
        overlay: camera_overlay,
        screen_height: recording_output.primary.height,
        screen_width: recording_output.primary.width,
        video: media_preview::VideoExportOptions {
          compression: 0,
          resolution_scale_percent: 100,
          source_scale_percent: 100,
        },
      })
    })
    .transpose()?;
  let cursor = sources
    .cursor
    .as_deref()
    .filter(|_| cursor_effects.bake)
    .and_then(|cursor| {
      cursor.gpu_cursor(
        frame.timestamp_ms,
        (frame.width, frame.height),
        cursor_effects,
      )
    });
  let keyboard = sources.keyboard_overlay(
    position_ms,
    keyboard_effects,
    (
      recording_output.primary.width,
      recording_output.primary.height,
    ),
  );
  // The camera is shown with its effects, the matte read for this moment.
  let live = sources.camera_effects_now();
  let on_screen = sources.primary_kind == screenwide_project::PrimaryRecordingKind::Camera;
  let matte = live
    .as_deref()
    .filter(|_| on_screen || camera_frame.is_some())
    .filter(|live| !live.matte.is_empty())
    .and_then(|live| {
      MatteFrames::open(&live.matte, STILL_MATTE_SIDE)
        .at(position_ms)
        .ok()
        .flatten()
    });
  surface.compose_texture_to_image(
    (
      &frame.texture,
      frame.subresource,
      (frame.width, frame.height),
    ),
    &recording_output.primary,
    ComposedFrame {
      cursor,
      keyboard,
      foreground_only: false,
      seconds: frame.timestamp_ms as f64 / 1_000.0,
    },
    camera_frame
      .as_ref()
      .zip(camera_geometry)
      .map(|(camera, geometry)| {
        (
          &camera.texture,
          camera.subresource,
          (camera.width, camera.height),
          geometry,
          recording_output.camera.drop_shadow,
          screenwide_export::recording_model::CAMERA_IN_FRONT,
        )
      }),
    live.as_deref().map(|live| StillCameraEffects {
      live,
      matte: matte.as_deref(),
      source_ms: position_ms,
      on_screen,
    }),
  )
}
