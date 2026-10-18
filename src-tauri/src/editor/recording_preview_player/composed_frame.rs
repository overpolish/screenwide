// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One frame of the open recording composed as the live preview draws it,
//! read back into memory: the platform decodes the source frames, and the
//! shared still compositor draws them.

use super::PlayerSources;
use screenwide_compositor::compositor::cursor_placement::placed_cursor;
use screenwide_compositor::compositor::still_overlay::compose_output_layers;
use screenwide_cursor_effects::CursorEffectSettings;
use screenwide_export::camera_effects::cpu::StillEffects;
use screenwide_export::decode::source_frame_image;
use screenwide_export::recording_model::{RecordingOutputSettings, CAMERA_IN_FRONT};
use screenwide_imaging::CapturedImage;
use screenwide_keyboard_effects::KeyboardEffectSettings;
use screenwide_scenes::CameraOverlaySettings;
use screenwide_workspace::scene::baked_camera_overlay;

/// The longest side a composed still reads the camera's matte at: stills
/// are copied or shown small, and the edge it draws is soft.
const STILL_MATTE_SIDE: u32 = 1_920;

/// The frame at `position_ms` composed with `recording_output`, the camera
/// drawn into it where `bake_camera`, with the cursor and the shortcut strip
/// `cursor_effects` and `keyboard_effects` describe.
pub(super) fn composed_frame_image(
  sources: &PlayerSources,
  position_ms: u64,
  bake_camera: bool,
  camera_overlay: CameraOverlaySettings,
  cursor_effects: CursorEffectSettings,
  keyboard_effects: KeyboardEffectSettings,
  recording_output: &RecordingOutputSettings,
) -> Result<CapturedImage, String> {
  let position_ms = position_ms.min(sources.duration_ms.saturating_sub(1));
  let mut screen = source_frame_image(&sources.screen_path, position_ms, sources.duration_ms)?;
  let camera_duration_ms = sources.camera_duration_ms.unwrap_or(sources.duration_ms);
  let mut camera = sources
    .camera_path
    .as_ref()
    .filter(|_| bake_camera)
    .map(|path| source_frame_image(path, position_ms, camera_duration_ms))
    .transpose()?;
  if let Some(mut effects) = StillEffects::new(sources.camera_effects_now(), STILL_MATTE_SIDE) {
    if sources.primary_kind == screenwide_project::PrimaryRecordingKind::Camera {
      effects.apply(&mut screen, position_ms);
    } else if let Some(camera) = camera.as_mut() {
      effects.apply(camera, position_ms.min(camera_duration_ms));
    }
  }
  let primary = &recording_output.primary;
  let cursor = match sources.cursor.as_deref().filter(|_| cursor_effects.bake) {
    Some(cursor) => placed_cursor(
      cursor,
      position_ms,
      cursor_effects,
      (screen.width, screen.height),
      primary,
    )?,
    None => None,
  };
  let keyboard = sources.keyboard_overlay(
    position_ms,
    keyboard_effects,
    (primary.width, primary.height),
  );
  let overlay = camera
    .as_ref()
    .map(|camera| {
      baked_camera_overlay(
        (camera.width, camera.height),
        primary,
        camera_overlay,
        recording_output.camera.drop_shadow,
        CAMERA_IN_FRONT,
      )
    })
    .transpose()?;
  compose_output_layers(
    &screen,
    primary,
    position_ms as f64 / 1_000.0,
    true,
    cursor.as_ref(),
    camera.as_ref(),
    overlay.as_ref(),
    keyboard.as_ref(),
    cursor_effects.clip_at_video_edge,
    false,
  )
}
