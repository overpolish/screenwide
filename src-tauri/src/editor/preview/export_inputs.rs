// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What an export of the open recording reads, gathered from the export
//! window's options once, for the size estimate and the preview frame to
//! describe the same pictures the export draws.

use std::path::PathBuf;
use std::sync::Arc;

use super::*;
use crate::editor::export_picture::{ExportPicture, Screen};
use screenwide_captions::CaptionTrack;
use screenwide_cursor_effects::CursorEffectSettings;
use screenwide_export::camera_effects::live::LiveCameraEffects;
use screenwide_export::media_preview::VideoExportOptions;
use screenwide_export::recording_model::{
  RecordingAudioTrack, RecordingCamera, RecordingOutputSettings,
};
use screenwide_export::track_selection::{AudioLayout, TrackSelection};
use screenwide_keyboard_effects::KeyboardEffectSettings;
use screenwide_scenes::CameraOverlaySettings;
use screenwide_timeline_edit::TimelinePlan;

pub(super) struct ExportInputs {
  pub bake_camera: bool,
  pub camera: Option<RecordingCamera>,
  pub camera_compression: u8,
  pub camera_effects: Option<Arc<LiveCameraEffects>>,
  camera_overlay: CameraOverlaySettings,
  pub camera_resolution_scale_percent: u16,
  captions: Option<CaptionTrack>,
  pub compression: u8,
  cursor: Option<(PathBuf, CursorEffectSettings)>,
  pub duration_ms: u64,
  pub include_camera: bool,
  include_primary_video: bool,
  keyboard: Option<(PathBuf, KeyboardEffectSettings)>,
  pub layout: AudioLayout,
  pub path: PathBuf,
  primary_kind: PrimaryRecordingKind,
  pub recording_output: RecordingOutputSettings,
  pub resolution_scale_percent: u16,
  pub selection: TrackSelection,
  size: (u32, u32),
  source_scale_percent: u16,
  pub timeline: Option<TimelinePlan>,
  pub tracks: Vec<RecordingAudioTrack>,
}

impl ExportInputs {
  /// The inputs of an export of the recording `artifact_id` with `options`,
  /// checked as the export checks them.
  pub(super) fn read(
    state: &EditorState,
    artifact_id: u64,
    options: RecordingExportOptions,
  ) -> Result<Self, String> {
    let RecordingExportOptions {
      audio_track_volumes,
      bake_camera,
      camera_compression,
      camera_overlay,
      camera_resolution_scale_percent,
      collapse_audio,
      compression,
      delete_project_after_export: _,
      cursor_effects,
      keyboard_effects,
      enabled_stream_indices,
      include_camera,
      include_primary_video,
      resolution_scale_percent,
      mut recording_output,
      screenshot_output: _,
      subtitles: _,
      timeline_edit: timeline_model,
      captions: _,
    } = options;
    if compression > 4 || camera_compression > 4 {
      return Err("Compression must be between 0 and 4".to_owned());
    }
    validate_camera_overlay(
      camera_overlay,
      (
        recording_output.primary.width,
        recording_output.primary.height,
      ),
    )?;
    let (camera_recording, captures, recording) = {
      let artifact = state
        .recording
        .artifact
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
      let Some(
        artifact @ EditorArtifact::Recording {
          audio_tracks,
          camera,
          cursor,
          keyboard,
          duration_ms,
          height,
          id,
          path,
          primary_kind,
          project,
          source_scale_percent,
          width,
          ..
        },
      ) = artifact.as_ref()
      else {
        return Err("There is no recording to export".to_owned());
      };
      if *id != artifact_id {
        return Err("That recording is no longer available in the editor".to_owned());
      }
      (
        crate::editor::camera_effects::CameraRecording::of(artifact),
        artifact.captures(),
        (
          audio_tracks.clone(),
          camera.clone(),
          cursor.as_ref().map(|cursor| cursor.path.clone()),
          keyboard.as_ref().map(|keyboard| keyboard.path.clone()),
          *duration_ms,
          (*width, *height),
          path.clone(),
          *primary_kind,
          project.clone(),
          *source_scale_percent,
        ),
      )
    };
    let (
      tracks,
      camera,
      cursor,
      keyboard,
      duration_ms,
      size,
      path,
      primary_kind,
      project,
      source_scale_percent,
    ) = recording;
    if bake_camera && (!include_primary_video || !include_camera || camera.is_none()) {
      return Err("There is no camera recording to bake in".to_owned());
    }
    if !include_primary_video && !include_camera && enabled_stream_indices.is_empty() {
      return Err("Select at least one track to export".to_owned());
    }
    validate_primary_resolution_scale(
      resolution_scale_percent,
      source_scale_percent,
      primary_kind,
    )?;
    validate_camera_resolution_scale(camera_resolution_scale_percent)?;
    recording_output.stamp_captures(captures);

    let project_folder = project.parent().unwrap_or(std::path::Path::new("."));
    let selection = crate::editor::speech::as_heard(
      TrackSelection::with_volumes(&tracks, &enabled_stream_indices, &audio_track_volumes)?,
      project_folder,
      &enabled_stream_indices,
    );
    let layout = if collapse_audio {
      AudioLayout::Mixdown
    } else {
      AudioLayout::SeparateTracks
    };
    let timeline =
      timeline_edit::export_plan(timeline_model.as_ref(), &project, artifact_id, duration_ms);
    let edit =
      timeline_edit::export_edit(timeline_model.as_ref(), &project, artifact_id, duration_ms);
    // The camera's effects over the edit's background ranges, and the
    // captions drawn into the picture.
    let camera_effects = camera_recording
      .and_then(|camera| crate::editor::camera_effects::live::load(&camera, edit.as_ref()));
    let primary_output = &recording_output.primary;
    let captions = edit.as_ref().and_then(|edit| {
      crate::editor::captions::track_for(
        (&project, &tracks, duration_ms),
        edit,
        crate::editor::captions::frame_for(primary_output.width, primary_output.height),
      )
    });
    Ok(Self {
      bake_camera,
      camera,
      camera_compression,
      camera_effects,
      camera_overlay,
      camera_resolution_scale_percent,
      captions,
      compression,
      cursor: cursor
        .filter(|_| cursor_effects.bake)
        .map(|cursor| (cursor, cursor_effects)),
      duration_ms,
      include_camera,
      include_primary_video,
      keyboard: keyboard
        .filter(|_| keyboard_effects.bake)
        .map(|keyboard| (keyboard, keyboard_effects)),
      layout,
      path,
      primary_kind,
      recording_output,
      resolution_scale_percent,
      selection,
      size,
      source_scale_percent,
      timeline,
      tracks,
    })
  }

  /// The scale and compression the screen's video is exported at.
  pub(super) fn video(&self) -> VideoExportOptions {
    VideoExportOptions {
      compression: self.compression,
      resolution_scale_percent: self.resolution_scale_percent,
      source_scale_percent: self.source_scale_percent,
    }
  }

  /// The scale "Original" stands for: the capture's own, or the camera's
  /// where the recording is of the camera alone.
  pub(super) fn original_scale_percent(&self) -> u16 {
    match self.primary_kind {
      PrimaryRecordingKind::Camera => 100,
      _ => self.source_scale_percent.clamp(100, 400),
    }
  }

  /// The screen's video file's picture: the camera baked in where it is,
  /// the screen alone otherwise. `None` where the export writes no screen
  /// video.
  pub(super) fn primary_picture(&self) -> Option<ExportPicture<'_>> {
    if !self.include_primary_video || self.size.0 == 0 || self.size.1 == 0 {
      return None;
    }
    let camera_is_primary = self.primary_kind == PrimaryRecordingKind::Camera;
    let screen = |camera_effects| Screen {
      camera_effects,
      captions: self.captions.as_ref(),
      cursor: self
        .cursor
        .as_ref()
        .map(|(path, settings)| (path.as_path(), *settings)),
      keyboard: self
        .keyboard
        .as_ref()
        .map(|(path, settings)| (path.as_path(), *settings)),
      duration_ms: self.duration_ms,
      layout: self.layout,
      output: &self.recording_output.primary,
      screen: &self.path,
      selection: &self.selection,
      size: self.size,
      timeline: self.timeline.as_ref(),
      video: self.video(),
    };
    Some(match self.camera.as_ref() {
      Some(camera) if self.bake_camera => ExportPicture::baked(
        screen(self.camera_effects.clone()),
        camera,
        self.camera_overlay,
        self.recording_output.camera.drop_shadow,
      ),
      _ => ExportPicture::screen(screen(
        self.camera_effects.clone().filter(|_| camera_is_primary),
      )),
    })
  }
}
