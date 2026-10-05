// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One moment's decoded frames, composed and presented together. A baked
//! camera is drawn into the screen's canvas, so presenting each frame on its
//! own would compose that canvas twice and throw the first drawing away.

use super::super::super::{annotation_preview, PlayerSources};
use super::gpu_decoder::GpuFrame;
use crate::editor::preview_platform::{ComposedFrame, DecodedTexture};
use crate::screenshots::ScreenshotOutputSettings;

/// Presents `screen` and `camera`, the frames one moment decoded to; a pane
/// without a frame keeps the picture it holds. `frame_ms` is how much source
/// time the drawn frame covers, which is what the reveal's motion blur is
/// measured over. A paused still passes zero: nothing is moving, so nothing is
/// blurred.
pub(in crate::editor::recording_preview_player::platform) fn present_native_frames(
  sources: &PlayerSources,
  screen: Option<&GpuFrame>,
  camera: Option<&GpuFrame>,
  frame_ms: f32,
) -> bool {
  let Some(timestamp_ms) = screen.or(camera).map(|frame| frame.timestamp_ms) else {
    return false;
  };
  let Some(surface) = sources.preview_surface.as_ref() else {
    return false;
  };
  let Some(mut settings) = sources
    .composition_settings
    .as_ref()
    .and_then(|settings| settings.read().ok().map(|settings| settings.clone()))
  else {
    return false;
  };
  if let Ok(clips) = sources.annotation_clips.read() {
    let ranges = sources
      .animation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    annotation_preview::apply_clips(
      &mut settings,
      &clips,
      &ranges,
      timestamp_ms,
      frame_ms,
      sources.annotation_pictures(),
      &sources.held_fills,
    );
  }
  sources.arrange_scene(&mut settings, timestamp_ms, frame_ms);
  annotation_preview::carry_camera_annotations(&mut settings, sources.annotation_pictures());
  // Annotations are authored against the full-resolution source; each frame
  // was decoded on its own grid, so the points move with it. A baked
  // camera's are placed on its frame when they are composed into it.
  if let Some(frame) = screen {
    remap(sources, 0, &mut settings.recording_output.primary, frame);
  }
  if let Some(frame) = camera.filter(|_| !settings.bake_camera) {
    remap(sources, 1, &mut settings.recording_output.camera, frame);
  }
  let composed = screen_composition(sources, &settings.recording_output.primary, screen);
  if settings.bake_camera && sources.camera_path.is_some() {
    surface
      .present_baked_camera_textures(
        screen.map(decoded),
        camera.map(decoded),
        &settings.recording_output.primary,
        (
          &settings.recording_output.camera,
          sources.annotation_pictures()[1],
        ),
        settings.camera_overlay,
        settings.recording_output.camera.drop_shadow,
        crate::editor::recording_model::CAMERA_IN_FRONT,
        composed.unwrap_or(ComposedFrame {
          cursor: None,
          keyboard: None,
          foreground_only: false,
          seconds: timestamp_ms as f64 / 1_000.0,
        }),
      )
      .unwrap_or(false)
  } else {
    // Two panes of their own still reach the compositor in one pass.
    let _batch = (screen.is_some() && camera.is_some()).then(|| surface.present_batch());
    let present = |index: u32, frame: &GpuFrame, output, composition| {
      let (texture, subresource, size) = decoded(frame);
      surface
        .present_composed_texture(index, texture, subresource, size, output, composition)
        .unwrap_or(false)
    };
    let screen_drawn = screen.zip(composed).is_none_or(|(frame, composition)| {
      present(0, frame, &settings.recording_output.primary, composition)
    });
    let camera_drawn = camera.is_none_or(|frame| {
      present(
        1,
        frame,
        &settings.recording_output.camera,
        ComposedFrame {
          cursor: None,
          keyboard: None,
          foreground_only: false,
          seconds: frame.timestamp_ms as f64 / 1_000.0,
        },
      )
    });
    screen_drawn && camera_drawn
  }
}

fn decoded(frame: &GpuFrame) -> DecodedTexture<'_> {
  (
    &frame.texture,
    frame.subresource,
    (frame.width, frame.height),
  )
}

fn remap(
  sources: &PlayerSources,
  index: usize,
  track: &mut ScreenshotOutputSettings,
  frame: &GpuFrame,
) {
  let Some(pane) = sources.layout.panes.get(index) else {
    return;
  };
  let output_width = track.width;
  annotation_preview::remap_source(
    track,
    (pane.source_width, pane.source_height),
    (frame.width, frame.height),
    output_width,
  );
}

/// What the screen's frame carries over its picture: the baked cursor and
/// the shortcut strip, which is fitted against the recording canvas.
fn screen_composition(
  sources: &PlayerSources,
  primary: &ScreenshotOutputSettings,
  screen: Option<&GpuFrame>,
) -> Option<ComposedFrame> {
  let frame = screen?;
  let cursor_settings = sources
    .cursor_settings
    .read()
    .map(|settings| *settings)
    .unwrap_or_default();
  let cursor = sources
    .cursor
    .as_deref()
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
  let keyboard = sources.keyboard_overlay(
    frame.timestamp_ms,
    keyboard_settings,
    (primary.width, primary.height),
  );
  Some(ComposedFrame {
    cursor,
    keyboard,
    foreground_only: false,
    seconds: frame.timestamp_ms as f64 / 1_000.0,
  })
}
