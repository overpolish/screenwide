// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One moment's decoded frames, staged on the workspace scene together. A
//! baked camera is drawn into the screen's canvas, so it rides on the
//! screen's layer rather than a pane of its own.

use super::super::super::{annotation_preview, PlayerSources};
use crate::editor::preview_platform::{
  baked_camera_overlay, LayerPicture, NativeWorkspacePlacement, RecordingWorkspaceLayer,
};
use screenwide_compositor::compositor::ComposedFrame;
use screenwide_export::decode::windows::gpu_decoder::GpuFrame;
use screenwide_output::ScreenshotOutputSettings;

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
      .annotation_ranges
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
  let seconds = timestamp_ms as f64 / 1_000.0;
  let captions = sources.captions_at(timestamp_ms, frame_ms);
  let screen_is_camera = sources.primary_kind == screenwide_project::PrimaryRecordingKind::Camera;
  let baked = settings.bake_camera && sources.camera_path.is_some();
  // The baked camera's place on the canvas follows its frame's size, which
  // only a new frame tells; a held camera stays where it was.
  let overlay = match camera.filter(|_| baked) {
    Some(frame) => {
      match baked_camera_overlay(
        (frame.width, frame.height),
        &settings.recording_output.primary,
        settings.camera_overlay,
        settings.recording_output.camera.drop_shadow,
        screenwide_export::recording_model::CAMERA_IN_FRONT,
      ) {
        Ok(overlay) => Some(overlay),
        Err(_) => return false,
      }
    }
    None => None,
  };
  let mut layers = vec![RecordingWorkspaceLayer {
    pane_index: 0,
    source_token: 0,
    source: picture(screen),
    settings: settings.recording_output.primary.clone(),
    placement: NativeWorkspacePlacement::default(),
    seconds,
    cursor: composed.and_then(|composed| composed.cursor),
    keyboard: composed.and_then(|composed| composed.keyboard),
    captions,
    camera: baked.then(|| picture(camera)),
    overlay: overlay.as_ref(),
    camera_settings: baked.then(|| {
      (
        &settings.recording_output.camera,
        sources.annotation_pictures()[1],
      )
    }),
    clip_cursor_at_video_edge: composed
      .and_then(|composed| composed.cursor)
      .is_some_and(|cursor| cursor.clip_at_video_edge),
    foreground_only: false,
    source_is_camera: screen_is_camera,
  }];
  if !baked && sources.camera_path.is_some() {
    layers.push(RecordingWorkspaceLayer {
      pane_index: 1,
      source_token: 1,
      source: picture(camera),
      settings: settings.recording_output.camera.clone(),
      placement: NativeWorkspacePlacement::default(),
      seconds,
      cursor: None,
      keyboard: None,
      captions: None,
      camera: None,
      overlay: None,
      camera_settings: None,
      clip_cursor_at_video_edge: false,
      foreground_only: false,
      source_is_camera: true,
    });
  }
  surface
    .present_recording_workspace(&layers)
    .unwrap_or(false)
}

/// The frame a layer is drawn from, or, where the decoder had no new frame
/// for it, the picture it holds.
fn picture(frame: Option<&GpuFrame>) -> LayerPicture<'_> {
  frame.map_or(LayerPicture::Held, decoded)
}

fn decoded(frame: &GpuFrame) -> LayerPicture<'_> {
  LayerPicture::Decoded((
    &frame.texture,
    frame.subresource,
    (frame.width, frame.height),
  ))
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
