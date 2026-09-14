// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::editor::recording_preview_player::platform::macos) fn spawn(
  sources: &PlayerSources,
  playback_factors: &[f64],
  start_ms: u64,
  playback_rate: f64,
  cancelled: Arc<AtomicBool>,
  sender: SyncSender<VideoFrame>,
) -> Result<std::thread::JoinHandle<()>, String> {
  // Decoding and composing at the on-screen pane size keeps playback frames
  // pixel-identical to paused frames and avoids Core Animation minifying a
  // full-resolution drawable, which visibly brightens thin glyphs.
  let screen_factor = playback_factors.first().copied().unwrap_or(1.0);
  let camera_factor = playback_factors.get(1).copied().unwrap_or(1.0);
  let screen_pane = &sources.playback_layout.panes[0];
  let mut screen = NativeVideoReader::open(
    &*open_asset(&sources.screen_path)?,
    scaled_dimension(screen_pane.source_width, screen_factor),
    scaled_dimension(screen_pane.source_height, screen_factor),
    start_ms,
    sources.duration_ms,
  )?;
  let mut camera = match (
    sources.camera_path.as_deref(),
    sources.camera_duration_ms,
    sources.playback_layout.panes.get(1),
  ) {
    (Some(path), Some(duration_ms), Some(pane)) => Some(NativeVideoReader::open(
      &*open_asset(path)?,
      scaled_dimension(pane.source_width, camera_factor),
      scaled_dimension(pane.source_height, camera_factor),
      start_ms.min(duration_ms.saturating_sub(1)),
      duration_ms,
    )?),
    _ => None,
  };
  let cursor = sources.cursor.clone();
  let cursor_settings = Arc::clone(&sources.cursor_settings);
  let keyboard = sources.keyboard.clone();
  let sources_keyboard_animation_ranges = Arc::clone(&sources.keyboard_animation_ranges);
  let keyboard_settings = Arc::clone(&sources.keyboard_settings);
  let composition_settings = sources.composition_settings.clone();
  let duration_ms = sources.duration_ms;
  let cursor_output = (
    sources.playback_layout.panes[0].source_width,
    sources.playback_layout.panes[0].source_height,
  );

  std::thread::Builder::new()
    .name("recording-preview-video-native".to_owned())
    .spawn(move || {
      let mut index = 0;
      while !cancelled.load(Ordering::Acquire) {
        let target_ms = source_position_ms(start_ms, index, playback_rate);
        if target_ms >= duration_ms {
          break;
        }
        let cursor_settings = cursor_settings
          .read()
          .map(|settings| *settings)
          .unwrap_or_default();
        let composition = composition_settings
          .as_ref()
          .expect("the recording player always has composition settings")
          .read()
          .map(|settings| settings.clone())
          .unwrap_or_else(|poisoned| poisoned.into_inner().clone());
        let raw_screen = match screen.pixel_frame_at(target_ms) {
          Ok(Some(frame)) => frame,
          Ok(None) | Err(_) => break,
        };
        let raw_camera = match camera.as_mut() {
          Some(reader) => match reader.pixel_frame_at(target_ms) {
            Ok(Some(frame)) => Some(frame),
            Ok(None) => None,
            Err(_) => break,
          },
          None => None,
        };
        let cursor_frame: Option<GpuCursorPreview> =
          gpu_cursor_preview(cursor.as_deref(), target_ms, cursor_settings, cursor_output);
        let keyboard_settings = keyboard_settings
          .read()
          .map(|settings| *settings)
          .unwrap_or_default();
        let keyboard_overlay = keyboard.as_deref().and_then(|_| {
          let ranges = sources_keyboard_animation_ranges
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
          keyboard.as_deref()?.evaluate_fitted_with_ranges(
            target_ms,
            keyboard_settings,
            (
              composition.recording_output.primary.width,
              composition.recording_output.primary.height,
            ),
            (!ranges.is_empty()).then_some(ranges.as_slice()),
          )
        });
        let screen_output = super::super::still_decode::scaled_output(
          &composition.recording_output.primary,
          screen_factor,
        );
        let screen_metadata = raw_screen.metadata();
        let camera_metadata = raw_camera.as_ref().map(|frame| frame.metadata());
        let (cursor, overlay) = match gpu_still_overlay(
          &screen_metadata,
          &screen_output,
          cursor_frame.as_ref(),
          composition
            .bake_camera
            .then_some(camera_metadata.as_ref())
            .flatten(),
          composition
            .bake_camera
            .then_some(composition.camera_overlay),
          composition.recording_output.camera.drop_shadow,
          composition.recording_output.camera_on_top,
        ) {
          Ok(value) => value,
          Err(_) => break,
        };
        let camera_output = super::super::still_decode::scaled_output(
          &composition.recording_output.camera,
          camera_factor,
        );
        let mut frame = VideoFrame {
          presentation_elapsed_ms: presentation_elapsed_ms(index),
          payload: VideoFramePayload::Native {
            screen: raw_screen,
            camera: raw_camera,
            screen_output,
            camera_output,
            cursor,
            keyboard: keyboard_overlay,
            overlay,
            bake_camera: composition.bake_camera,
            seconds: target_ms as f64 / 1_000.0,
            clip_cursor_at_video_edge: cursor_settings.clip_at_video_edge,
          },
        };
        loop {
          match sender.try_send(frame) {
            Ok(()) => break,
            Err(TrySendError::Full(returned)) => {
              if cancelled.load(Ordering::Acquire) {
                return;
              }
              frame = returned;
              std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Err(TrySendError::Disconnected(_)) => return,
          }
        }
        index += 1;
      }
    })
    .map_err(|error| error.to_string())
}

pub(super) fn scaled_dimension(source: u32, factor: f64) -> u32 {
  ((f64::from(source) * factor.clamp(0.0, 1.0))
    .round()
    .max(2.0) as u32)
    .min(source)
}
