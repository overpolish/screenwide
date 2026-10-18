// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One moment's decoded frames, staged on the workspace scene together, on
//! every platform. The composition the moment shows is worked out here: its
//! annotations and scene, the cursor and the shortcut strip baked over the
//! screen, the camera baked into the screen's canvas or drawn as a pane of
//! its own. A platform only says what a decoded frame is.

use super::super::{annotation_preview, PlayerSources};
use screenwide_compositor::compositor::cursor_placement::placed_cursor;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_workspace::scene::{
  baked_camera_overlay, LayerPicture, NativeWorkspacePlacement, RecordingWorkspaceLayer,
};

/// A frame a platform's decoder produced for one pane.
pub(in crate::editor::recording_preview_player) trait PaneFrame {
  fn picture(&self) -> LayerPicture<'_>;
  /// The size it was decoded at, which may be smaller than the source.
  fn size(&self) -> (u32, u32);
}

/// The moment a pair of frames shows.
#[derive(Clone, Copy)]
pub(in crate::editor::recording_preview_player) struct Moment {
  pub timestamp_ms: u64,
  /// How much source time the frame covers, which is what an annotation's
  /// reveal blurs and captions are exposed over. A paused still covers none.
  pub frame_ms: f32,
  /// How far each pane's composition shrinks toward its on-screen size, the
  /// screen's first, matching how far its decode shrank.
  pub factors: (f64, f64),
}

/// Presents `screen` and `camera`, the frames one moment decoded to; a pane
/// without a frame keeps the picture it holds.
pub(in crate::editor::recording_preview_player) fn present<F: PaneFrame>(
  sources: &PlayerSources,
  screen: Option<&F>,
  camera: Option<&F>,
  moment: Moment,
) -> bool {
  let Moment {
    timestamp_ms,
    frame_ms,
    factors,
  } = moment;
  if screen.is_none() && camera.is_none() {
    return false;
  }
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
  // The shortcut strip is fitted against the full recording canvas, before
  // the composition shrinks toward the pane.
  let canvas = (
    settings.recording_output.primary.width,
    settings.recording_output.primary.height,
  );
  // Annotations are authored against the full-resolution source; each frame
  // was decoded on its own grid, so the points move with it. A baked
  // camera's are placed on its frame when they are composed into it.
  if let Some(frame) = screen {
    fit(
      sources,
      0,
      &mut settings.recording_output.primary,
      frame,
      factors.0,
    );
  }
  if let Some(frame) = camera.filter(|_| !settings.bake_camera) {
    fit(
      sources,
      1,
      &mut settings.recording_output.camera,
      frame,
      factors.1,
    );
  }
  let seconds = timestamp_ms as f64 / 1_000.0;
  let cursor_settings = sources
    .cursor_settings
    .read()
    .map(|settings| *settings)
    .unwrap_or_default();
  let cursor = screen.and_then(|frame| {
    let cursor = sources.cursor.as_deref().filter(|_| cursor_settings.bake)?;
    placed_cursor(
      cursor,
      timestamp_ms,
      cursor_settings,
      frame.size(),
      &settings.recording_output.primary,
    )
    .ok()
    .flatten()
  });
  let keyboard_settings = sources
    .keyboard_settings
    .read()
    .map(|settings| *settings)
    .unwrap_or_default();
  let keyboard =
    screen.and_then(|_| sources.keyboard_overlay(timestamp_ms, keyboard_settings, canvas));
  let baked = settings.bake_camera && sources.camera_path.is_some();
  // The baked camera's place on the canvas follows its frame's size, which
  // only a new frame tells; a held camera stays where it was.
  let overlay = match camera.filter(|_| baked) {
    Some(frame) => match baked_camera_overlay(
      frame.size(),
      &settings.recording_output.primary,
      settings.camera_overlay,
      settings.recording_output.camera.drop_shadow,
      screenwide_export::recording_model::CAMERA_IN_FRONT,
    ) {
      Ok(overlay) => Some(overlay),
      Err(_) => return false,
    },
    None => None,
  };
  let token = timestamp_ms << 2;
  let mut layers = vec![RecordingWorkspaceLayer {
    pane_index: 0,
    source_token: token,
    source: picture(screen),
    settings: settings.recording_output.primary.clone(),
    placement: NativeWorkspacePlacement::default(),
    seconds,
    cursor,
    keyboard,
    captions: sources.captions_at(timestamp_ms, frame_ms),
    camera: baked.then(|| picture(camera)),
    overlay: overlay.as_ref(),
    camera_settings: baked.then(|| {
      (
        &settings.recording_output.camera,
        sources.annotation_pictures()[1],
      )
    }),
    clip_cursor_at_video_edge: cursor.is_some_and(|cursor| cursor.clip_at_video_edge),
    foreground_only: false,
    source_is_camera: sources.primary_kind == screenwide_project::PrimaryRecordingKind::Camera,
  }];
  if !baked && sources.camera_path.is_some() {
    layers.push(RecordingWorkspaceLayer {
      pane_index: 1,
      source_token: token | 1,
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
fn picture<F: PaneFrame>(frame: Option<&F>) -> LayerPicture<'_> {
  frame.map_or(LayerPicture::Held, PaneFrame::picture)
}

/// Shrinks `track` by `factor` toward its pane and moves what is authored on
/// the source grid onto the grid `frame` was decoded on.
fn fit<F: PaneFrame>(
  sources: &PlayerSources,
  index: usize,
  track: &mut ScreenshotOutputSettings,
  frame: &F,
  factor: f64,
) {
  let Some(pane) = sources.playback_layout.panes.get(index) else {
    return;
  };
  let output_width = track.width;
  *track = scaled_output(track, factor);
  screenwide_workspace::remap_source(
    track,
    (pane.source_width, pane.source_height),
    frame.size(),
    output_width,
  );
}

/// `settings` shrunk by `factor`, never below the compositor's 64-pixel
/// minimum.
pub(in crate::editor::recording_preview_player) fn scaled_output(
  settings: &ScreenshotOutputSettings,
  factor: f64,
) -> ScreenshotOutputSettings {
  if factor >= 1.0 {
    return settings.clone();
  }
  let minimum = (64.0 / f64::from(settings.width.max(1)))
    .max(64.0 / f64::from(settings.height.max(1)))
    .min(1.0);
  let factor = factor.max(minimum);
  let mut scaled = settings.clone();
  scaled.width = ((f64::from(settings.width) * factor).round().max(64.0)) as u32;
  scaled.height = ((f64::from(settings.height) * factor).round().max(64.0)) as u32;
  scaled
}

/// Decode width used before the webview has reported how large the preview
/// panes actually are on screen.
const FALLBACK_TARGET_WIDTH: u32 = 1_600;

/// How much a pane's decode and composition shrink so the presented picture
/// matches the on-screen pane instead of the full output resolution. Zooming
/// grows the reported target, so this reaches 1.0 exactly when the view needs
/// native pixels.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub(in crate::editor::recording_preview_player) fn pane_factor(
  target_sizes: &[(u32, u32)],
  index: usize,
  output_width: u32,
) -> f64 {
  if output_width == 0 {
    return 1.0;
  }
  // A pane that has not reported a real on-screen size yet (or was laid out
  // collapsed) must not shrink the composition to nothing.
  let target_width = target_sizes
    .get(index)
    .map(|size| size.0)
    .filter(|width| *width >= 16)
    .unwrap_or(FALLBACK_TARGET_WIDTH);
  (f64::from(target_width) / f64::from(output_width)).min(1.0)
}
