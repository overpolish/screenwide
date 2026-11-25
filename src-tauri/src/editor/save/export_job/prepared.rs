// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What one file of a recording's export is made from, worked out before
//! anything is written: the edit and plan it follows, the audio it carries,
//! and what kind of picture it is. A format exported on its own and the
//! formats exported in one pass are prepared alike.

use std::sync::Arc;

use super::*;
use crate::editor::export_picture::{ExportPicture, Screen};
use screenwide_export::camera_effects::live::LiveCameraEffects;
use screenwide_export::track_selection;

/// What kind of file the export writes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::editor::save) enum Route {
  /// The sound alone: asked for as M4A or WAV, or all that is left with no
  /// picture in it.
  Sound,
  /// The camera standing in for the screen.
  CameraAlone,
  /// The screen with the camera drawn into it.
  Baked,
  /// The screen, with the camera, where it is kept, as a file of its own.
  Screen,
}

pub(in crate::editor::save) struct Prepared {
  pub(in crate::editor::save) options: RecordingExportOptions,
  pub(in crate::editor::save) timeline: Option<screenwide_timeline_edit::TimelinePlan>,
  pub(in crate::editor::save) selection: track_selection::TrackSelection,
  pub(in crate::editor::save) layout: track_selection::AudioLayout,
  pub(in crate::editor::save) camera_effects: Option<Arc<LiveCameraEffects>>,
  pub(in crate::editor::save) route: Route,
}

/// Checks `options` against `artifact` and works out what its export is
/// made from. The microphone cleaned up and the songs' singing separated are
/// made here, before anything is drawn.
pub(in crate::editor::save) fn prepare(
  artifact: &EditorArtifact,
  progress_app: &AppHandle,
  mut options: RecordingExportOptions,
) -> Result<Prepared, String> {
  let EditorArtifact::Recording {
    audio_tracks,
    camera,
    duration_ms,
    id,
    path: working,
    primary_kind,
    project,
    source_scale_percent,
    ..
  } = artifact
  else {
    return Err("Only a recording is exported as a video".to_owned());
  };
  options.recording_output.stamp_captures(artifact.captures());
  validate_primary_resolution_scale(
    options.resolution_scale_percent,
    *source_scale_percent,
    *primary_kind,
  )?;
  validate_camera_resolution_scale(options.camera_resolution_scale_percent)?;
  // The overlay places a baked-in camera on the screen canvas, which an audio
  // recording has only as a 1 by 1 stand-in, so it is checked only when used.
  if options.bake_camera {
    let canvas = &options.recording_output.primary;
    validate_camera_overlay(options.camera_overlay, (canvas.width, canvas.height))?;
  }
  // A microphone with its noise taken out or its voice cleaned up is
  // exported from the file made of it, leveled by Auto volume, and the
  // system audio making way for it, as the preview plays them.
  let project_folder = project.parent().unwrap_or(Path::new("."));
  crate::editor::speech::ready_for_export(
    progress_app,
    project_folder,
    audio_tracks,
    &options.enabled_stream_indices,
    (working, *duration_ms),
  )?;
  // The edit and plan the export follows, its songs' singing separated first.
  let (edit, timeline) = timeline_edit::for_export(
    progress_app,
    options.timeline_edit.as_ref(),
    project,
    (*id, *duration_ms),
  )?;
  let selection = crate::editor::speech::as_heard(
    track_selection::TrackSelection::with_volumes(
      audio_tracks,
      &options.enabled_stream_indices,
      &options.audio_track_volumes,
    )?,
    project_folder,
    &options.enabled_stream_indices,
  );
  let layout = if options.collapse_audio {
    track_selection::AudioLayout::Mixdown
  } else {
    track_selection::AudioLayout::SeparateTracks
  };
  let camera_effects = crate::editor::camera_effects::CameraRecording::of(artifact)
    .and_then(|camera| crate::editor::camera_effects::live::load(&camera, edit.as_ref()));
  if !options.include_primary_video
    && !options.include_camera
    && options.enabled_stream_indices.is_empty()
  {
    return Err("Select at least one track to export".to_owned());
  }
  if *primary_kind == PrimaryRecordingKind::Audio
    && (options.include_primary_video || options.include_camera || options.bake_camera)
  {
    return Err("This audio recording has no video track to export".to_owned());
  }
  let route = if options.recording_format.is_sound_only()
    || *primary_kind == PrimaryRecordingKind::Audio
    || (!options.include_primary_video && !options.include_camera)
  {
    Route::Sound
  } else if !options.include_primary_video {
    if camera.is_none() {
      return Err("There is no camera track to export".to_owned());
    }
    Route::CameraAlone
  } else if options.bake_camera {
    if !options.include_camera {
      return Err("Select the camera track before baking it in".to_owned());
    }
    if camera.is_none() {
      return Err("There is no camera recording to bake in".to_owned());
    }
    Route::Baked
  } else {
    Route::Screen
  };
  Ok(Prepared {
    options,
    timeline,
    selection,
    layout,
    camera_effects,
    route,
  })
}

impl Prepared {
  /// How long the exported video runs.
  pub(in crate::editor::save) fn duration_ms(&self, artifact: &EditorArtifact) -> u64 {
    let EditorArtifact::Recording { duration_ms, .. } = artifact else {
      return 0;
    };
    self.timeline.as_ref().map_or(
      *duration_ms,
      screenwide_timeline_edit::TimelinePlan::duration_ms,
    )
  }

  /// The screen's own picture and what is drawn into it, with the camera's
  /// effects where `camera_effects` says.
  pub(in crate::editor::save) fn screen<'a>(
    &'a self,
    artifact: &'a EditorArtifact,
    camera_effects: Option<Arc<LiveCameraEffects>>,
  ) -> Screen<'a> {
    let EditorArtifact::Recording {
      cursor,
      keyboard,
      duration_ms,
      height,
      path,
      source_scale_percent,
      width,
      ..
    } = artifact
    else {
      unreachable!("only a recording is prepared");
    };
    let options = &self.options;
    // A scene can show the cursor where the recording hides it, so it is
    // drawn wherever any part of the video shows it.
    let cursor = cursor
      .as_ref()
      .filter(|_| {
        screenwide_annotations::timing::cursor_shown_anywhere(
          self
            .timeline
            .as_ref()
            .map_or(&[][..], |plan| &plan.scene_clips),
          options.cursor_effects.bake,
        )
      })
      .map(|cursor| (cursor.path.as_path(), options.cursor_effects));
    let keyboard = keyboard
      .as_ref()
      .filter(|_| options.keyboard_effects.bake)
      .map(|keyboard| (keyboard.path.as_path(), options.keyboard_effects));
    Screen {
      camera_effects,
      captions: options.captions.as_ref(),
      cursor,
      keyboard,
      duration_ms: *duration_ms,
      layout: self.layout,
      output: &options.recording_output.primary,
      screen: path,
      selection: &self.selection,
      size: (*width, *height),
      timeline: self.timeline.as_ref(),
      video: screenwide_export::media_preview::VideoExportOptions {
        compression: options.compression,
        resolution_scale_percent: options.resolution_scale_percent,
        source_scale_percent: *source_scale_percent,
      },
    }
  }

  /// The screen's picture on its own, with the camera's effects only where
  /// the recording is of the camera alone.
  pub(in crate::editor::save) fn screen_alone<'a>(
    &'a self,
    artifact: &'a EditorArtifact,
  ) -> Screen<'a> {
    let camera_is_primary = matches!(
      artifact,
      EditorArtifact::Recording {
        primary_kind: PrimaryRecordingKind::Camera,
        ..
      }
    );
    self.screen(
      artifact,
      self.camera_effects.clone().filter(|_| camera_is_primary),
    )
  }

  /// The picture the compositor draws for this file, where it draws one:
  /// the screen with the camera baked in, or the screen framed, drawn over
  /// or cut differently from how it was recorded.
  pub(in crate::editor::save) fn drawn<'a>(
    &'a self,
    artifact: &'a EditorArtifact,
  ) -> Option<ExportPicture<'a>> {
    match self.route {
      Route::Baked => {
        let EditorArtifact::Recording {
          camera: Some(camera),
          ..
        } = artifact
        else {
          return None;
        };
        Some(ExportPicture::baked(
          self.screen(artifact, self.camera_effects.clone()),
          camera,
          self.options.camera_overlay,
          self.options.recording_output.camera.drop_shadow,
        ))
      }
      Route::Screen => {
        Some(ExportPicture::screen(self.screen_alone(artifact))).filter(ExportPicture::is_composed)
      }
      Route::Sound | Route::CameraAlone => None,
    }
  }
}
