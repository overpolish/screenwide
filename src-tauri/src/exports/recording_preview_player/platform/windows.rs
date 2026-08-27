// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview backend: Media Foundation hardware decode into D3D11 textures
//! presented by native flip-model swap chains. Live frames never enter system
//! memory or cross the Tauri IPC boundary.

mod decoder;
mod gpu_decoder;
mod still;
mod thumbnails;

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
use crate::exports::preview_platform::ComposedFrame;

pub(crate) const NATIVE_STILLS: bool = true;
pub(crate) type StillDecoder = still::NativeStillDecoder;

pub(crate) enum VideoFramePayload {
  Native {
    frame: GpuFrame,
    index: u32,
    presented: Option<SyncSender<()>>,
  },
}

pub(super) fn present_native_frame(sources: &PlayerSources, index: u32, frame: &GpuFrame) -> bool {
  sources.preview_surface.as_ref().is_some_and(|surface| {
    let settings = sources
      .composition_settings
      .as_ref()
      .and_then(|settings| settings.read().ok().map(|settings| settings.clone()));
    settings.is_some_and(|settings| {
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
  let result = present_native_frame(sources, index, &frame);
  if let Some(presented) = presented {
    let _ = presented.send(());
  }
  result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_video(
  sources: &PlayerSources,
  _playback_factors: &[f64],
  start_ms: u64,
  playback_rate: f64,
  _still: bool,
  cancelled: Arc<AtomicBool>,
  _child: Arc<Mutex<Option<Child>>>,
  sender: SyncSender<VideoFrame>,
) -> Result<std::thread::JoinHandle<()>, String> {
  if sources.playback_layout.panes.is_empty() {
    return Err("The recording has no video pane".to_owned());
  }
  let mut paths = vec![(0_u32, sources.screen_path.clone(), sources.duration_ms)];
  if let Some(path) = &sources.camera_path {
    paths.push((
      1,
      path.clone(),
      sources.camera_duration_ms.unwrap_or(sources.duration_ms),
    ));
  }
  let surface = sources
    .preview_surface
    .clone()
    .ok_or_else(|| "Windows GPU preview has no native presentation surface".to_owned())?;

  let (startup_tx, startup_rx) = mpsc::sync_channel(1);
  let thread = std::thread::Builder::new()
    .name("recording-preview-video-windows".to_owned())
    .spawn(move || {
      let mut streams = Vec::with_capacity(paths.len());
      for (index, path, duration_ms) in paths {
        let mut reader = match GpuVideoReader::open(&path, start_ms, surface.clone()) {
          Ok(reader) => reader,
          Err(error) => {
            let _ = startup_tx.send(Err(error));
            return;
          }
        };
        match reader.frame_at(start_ms) {
          Ok(_) => {}
          Err(error) => {
            let _ = startup_tx.send(Err(error));
            return;
          }
        }
        streams.push((index, duration_ms, reader));
      }
      let _ = startup_tx.send(Ok(()));
      let mut output_frame = 0_u64;
      while !cancelled.load(Ordering::Acquire) {
        let target_ms = source_position_ms(start_ms, output_frame, playback_rate);
        let mut sent = false;
        for (index, duration_ms, reader) in &mut streams {
          if target_ms >= *duration_ms {
            continue;
          }
          let mut frame = match reader.frame_at(target_ms) {
            Ok(Some(frame)) => frame,
            Ok(None) => continue,
            Err(_) => return,
          };
          // Presentation metadata follows the output tick even when the
          // retained source texture repeats, so overlays still animate at
          // the full output cadence.
          frame.timestamp_ms = target_ms;
          frame.timestamp_100ns = i64::try_from(target_ms)
            .unwrap_or(i64::MAX / 10_000)
            .saturating_mul(10_000);
          // The decoder's sample owns a pooled DXGI surface. Keep it retained
          // until this output tick has submitted the texture; repeated slow-
          // motion ticks safely reuse the retained sample before decoding on.
          let (presented_tx, presented_rx) = mpsc::sync_channel(0);
          let mut frame = VideoFrame {
            presentation_elapsed_ms: presentation_elapsed_ms(output_frame),
            payload: VideoFramePayload::Native {
              frame,
              index: *index,
              presented: Some(presented_tx),
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
                std::thread::yield_now();
              }
              Err(TrySendError::Disconnected(_)) => return,
            }
          }
          while !cancelled.load(Ordering::Acquire) {
            match presented_rx.recv_timeout(Duration::from_millis(50)) {
              Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
              Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
          }
          sent = true;
        }
        if !sent {
          break;
        }
        output_frame = output_frame.saturating_add(1);
      }
    })
    .map_err(|error| error.to_string())?;
  match startup_rx.recv_timeout(Duration::from_secs(5)) {
    Ok(Ok(())) => Ok(thread),
    Ok(Err(error)) => {
      let _ = thread.join();
      Err(error)
    }
    Err(_) => {
      let _ = thread.join();
      Err("Media Foundation did not open the preview in time".to_owned())
    }
  }
}

pub(crate) fn playback_factors(
  pane_target_sizes: &[(u32, u32)],
  sources: &PlayerSources,
) -> Vec<f64> {
  sources
    .playback_layout
    .panes
    .iter()
    .enumerate()
    .map(|(index, pane)| {
      pane_target_sizes
        .get(index)
        .map(|size| f64::from(size.0.max(16)) / f64::from(pane.source_width.max(1)))
        .unwrap_or(0.5)
        .clamp(0.1, 1.0)
    })
    .collect()
}

pub(crate) fn generate_thumbnails(sources: PlayerSources, count: u32, channel: Channel) {
  thumbnails::generate(sources, count, channel);
}

// Backend-parity shim; the Windows player does not call it yet.
#[allow(dead_code)]
pub(crate) fn source_frame_jpeg(
  path: &std::path::Path,
  position_ms: u64,
  duration_ms: u64,
) -> Result<Vec<u8>, String> {
  thumbnails::source_frame_jpeg(path, position_ms, duration_ms)
}

pub(crate) fn composed_frame_image(
  sources: &PlayerSources,
  position_ms: u64,
  bake_camera: bool,
  camera_overlay: crate::exports::CameraOverlaySettings,
  cursor_effects: crate::exports::cursor_effects::CursorEffectSettings,
  keyboard_effects: crate::exports::keyboard_effects::KeyboardEffectSettings,
  recording_output: &crate::exports::RecordingOutputSettings,
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
      crate::exports::media_preview::bake_geometry(
        crate::exports::media_preview::BakedVideoExportOptions {
          camera_drop_shadow: recording_output.camera.drop_shadow,
          camera_height: camera.height,
          camera_width: camera.width,
          overlay: camera_overlay,
          screen_height: recording_output.primary.height,
          screen_width: recording_output.primary.width,
          video: crate::exports::media_preview::VideoExportOptions {
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
