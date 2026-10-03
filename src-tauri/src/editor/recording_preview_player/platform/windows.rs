// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview backend: Media Foundation hardware decode into D3D11
//! textures presented by native flip-model swap chains. Live frames never enter
//! system memory or cross the Tauri IPC boundary.

mod composed_frame;
mod decoder;
mod gpu_decoder;
mod still;
mod thumbnails;

pub(crate) use composed_frame::composed_frame_image;
pub(crate) use decoder::LumaReader;
pub(crate) use thumbnails::{each_source_frame, source_frame_image};

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
        let ranges = sources
          .animation_ranges
          .read()
          .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::editor::recording_preview_player::annotation_preview::apply_clips(
          &mut settings,
          &clips,
          &ranges,
          frame.timestamp_ms,
          frame_ms,
          sources.annotation_pictures(),
          &sources.held_fills,
        );
      }
      sources.arrange_scene(&mut settings, frame.timestamp_ms, frame_ms);
      crate::editor::recording_preview_player::annotation_preview::carry_camera_annotations(
        &mut settings,
        sources.annotation_pictures(),
      );
      // Annotations are authored against the full-resolution source; this frame
      // was decoded on its own grid, so the points move with it. A baked
      // camera's are placed on its frame when they are composed into it.
      let baked_frame = settings.bake_camera && index == 1;
      let track = if index == 0 {
        &mut settings.recording_output.primary
      } else {
        &mut settings.recording_output.camera
      };
      let output_width = track.width;
      if let Some(pane) = sources
        .layout
        .panes
        .get(index as usize)
        .filter(|_| !baked_frame)
      {
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
            (
              &settings.recording_output.camera,
              sources.annotation_pictures()[1],
            ),
            settings.camera_overlay,
            settings.recording_output.camera.drop_shadow,
            crate::editor::recording_model::CAMERA_IN_FRONT,
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
