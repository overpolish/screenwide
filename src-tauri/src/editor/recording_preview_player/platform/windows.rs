// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview backend: Media Foundation hardware decode into D3D11
//! textures presented by native flip-model swap chains. Live frames never enter
//! system memory or cross the Tauri IPC boundary.

mod decoder;
mod gpu_decoder;
mod still;
mod thumbnails;

pub(crate) use decoder::LumaReader;
pub(crate) use thumbnails::{each_source_frame, source_frame_image};

#[path = "windows/video_playback.rs"]
mod video_playback;
pub(crate) use video_playback::playback_factors;
pub(crate) use video_playback::spawn_video;

use std::{
  process::Child,
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, SyncSender, TrySendError},
    Arc, Mutex,
  },
  time::Duration,
};

use tauri::ipc::Channel;

use self::gpu_decoder::GpuFrame;
pub(crate) use self::gpu_decoder::GpuVideoReader;
use super::super::{
  video::{presentation_elapsed_ms, source_position_ms, VideoFrame},
  PlayerSources,
};
use crate::editor::preview_platform::ComposedFrame;

pub(crate) const NATIVE_STILLS: bool = true;
pub(crate) type StillDecoder = still::NativeStillDecoder;

pub(crate) enum VideoFramePayload {
  Native {
    frame: GpuFrame,
    index: u32,
    presented: Option<SyncSender<()>>,
  },
}

/// `frame_ms` is how much source time this drawn frame covers, which is what
/// the reveal's motion blur is measured over. A paused still passes zero:
/// nothing is moving, so nothing is blurred.
pub(super) fn present_native_frame(
  sources: &PlayerSources,
  index: u32,
  frame: &GpuFrame,
  frame_ms: f32,
) -> bool {
  sources.preview_surface.as_ref().is_some_and(|surface| {
    let settings = sources
      .composition_settings
      .as_ref()
      .and_then(|settings| settings.read().ok().map(|settings| settings.clone()));
    settings.is_some_and(|mut settings| {
      if let Ok(clips) = sources.annotation_clips.read() {
        crate::editor::recording_preview_player::annotation_preview::apply_clips(
          &mut settings,
          &clips,
          frame.timestamp_ms,
          frame_ms,
          sources.held_fills.as_ref(),
        );
      }
      // Annotations are authored against the full-resolution source; this frame
      // was decoded on its own grid, so the points move with it.
      let track = if index == 0 {
        &mut settings.recording_output.primary
      } else {
        &mut settings.recording_output.camera
      };
      let output_width = track.width;
      if let Some(pane) = sources.layout.panes.get(index as usize) {
        crate::editor::recording_preview_player::annotation_preview::remap_source(
          track,
          (pane.source_width, pane.source_height),
          (frame.width, frame.height),
          output_width,
        );
      }
      let cursor_settings = sources
        .cursor_settings
        .read()
        .map(|settings| *settings)
        .unwrap_or_default();
      let cursor = (index == 0)
        .then_some(())
        .and(sources.cursor.as_deref())
        .filter(|_| cursor_settings.bake)
        .and_then(|cursor| {
          cursor.gpu_cursor(
            frame.timestamp_ms,
            (frame.width, frame.height),
            cursor_settings,
          )
        });
      let keyboard_settings = sources
        .keyboard_settings
        .read()
        .map(|settings| *settings)
        .unwrap_or_default();
      // The shortcut strip belongs to the screen pane and is fitted against the
      // recording canvas, never against the camera pane's own output.
      let keyboard = (index == 0)
        .then(|| {
          sources.keyboard_overlay(
            frame.timestamp_ms,
            keyboard_settings,
            (
              settings.recording_output.primary.width,
              settings.recording_output.primary.height,
            ),
          )
        })
        .flatten();
      let composed = ComposedFrame {
        cursor,
        keyboard,
        foreground_only: false,
        seconds: frame.timestamp_ms as f64 / 1_000.0,
      };
      if settings.bake_camera && sources.camera_path.is_some() {
        surface
          .present_baked_camera_texture(
            index,
            &frame.texture,
            frame.subresource,
            (frame.width, frame.height),
            &settings.recording_output.primary,
            settings.camera_overlay,
            settings.recording_output.camera.drop_shadow,
            settings.recording_output.camera_on_top,
            composed,
          )
          .unwrap_or(false)
      } else {
        let output = if index == 0 {
          &settings.recording_output.primary
        } else {
          &settings.recording_output.camera
        };
        surface
          .present_composed_texture(
            index,
            &frame.texture,
            frame.subresource,
            (frame.width, frame.height),
            output,
            composed,
          )
          .unwrap_or(false)
      }
    })
  })
}

pub(crate) fn send_frame(sources: &PlayerSources, payload: VideoFramePayload) -> bool {
  let VideoFramePayload::Native {
    frame,
    index,
    presented,
  } = payload;
  // Playback exposes each frame for one frame interval, which is the window
  // a moving annotation smears over; a paused still exposes nothing.
  let frame_ms = sources
    .frames_per_second
    .filter(|rate| *rate > 0.0)
    .map_or(0.0, |rate| (1_000.0 / rate) as f32);
  let result = present_native_frame(sources, index, &frame, frame_ms);
  if let Some(presented) = presented {
    let _ = presented.send(());
  }
  result
}

pub(crate) fn generate_thumbnails(sources: PlayerSources, count: u32, channel: Channel) {
  thumbnails::generate(sources, count, channel);
}

pub(crate) fn composed_frame_image(
  sources: &PlayerSources,
  position_ms: u64,
  bake_camera: bool,
  camera_overlay: crate::editor::CameraOverlaySettings,
  cursor_effects: crate::editor::cursor_effects::CursorEffectSettings,
  keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings,
  recording_output: &crate::editor::RecordingOutputSettings,
) -> Result<crate::screenshots::CapturedImage, String> {
  let surface = sources
    .preview_surface
    .clone()
    .ok_or_else(|| "Windows GPU preview has no native presentation surface".to_owned())?;
  let position_ms = position_ms.min(sources.duration_ms.saturating_sub(1));
  let mut reader = GpuVideoReader::open(&sources.screen_path, position_ms, surface.clone())?;
  let frame = reader
    .frame_at(position_ms)?
    .ok_or_else(|| "Media Foundation returned no source frame".to_owned())?;
  let mut camera_reader = if bake_camera {
    sources
      .camera_path
      .as_ref()
      .map(|path| GpuVideoReader::open(path, position_ms, surface.clone()))
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
      crate::editor::media_preview::bake_geometry(
        crate::editor::media_preview::BakedVideoExportOptions {
          camera_drop_shadow: recording_output.camera.drop_shadow,
          camera_height: camera.height,
          camera_width: camera.width,
          overlay: camera_overlay,
          screen_height: recording_output.primary.height,
          screen_width: recording_output.primary.width,
          video: crate::editor::media_preview::VideoExportOptions {
            compression: 0,
            resolution_scale_percent: 100,
            source_scale_percent: 100,
          },
        },
      )
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
  surface.compose_texture_to_image(
    &frame.texture,
    frame.subresource,
    (frame.width, frame.height),
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
          recording_output.camera_on_top,
        )
      }),
  )
}
