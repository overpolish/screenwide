// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What one exported video file shows, described once for the export that
//! writes it and the estimate that sizes it, so the two always agree on
//! whether the picture is drawn by the compositor and on what it draws.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use screenwide_annotations::timing::AnnotationTrack;
use screenwide_captions::CaptionTrack;
use screenwide_cursor_effects::CursorEffectSettings;
use screenwide_export::camera_effects::live::LiveCameraEffects;
use screenwide_export::cursor_export::{self, CursorExportRequest, ExportCameraEffects};
use screenwide_export::media_preview::{BakedVideoExportOptions, VideoExportOptions};
use screenwide_export::recording_model::RecordingCamera;
use screenwide_export::track_selection::{AudioLayout, TrackSelection};
use screenwide_keyboard_effects::KeyboardEffectSettings;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_timeline_edit::TimelinePlan;

/// One video file's picture and the audio muxed beside it.
pub(in crate::editor) struct ExportPicture<'a> {
  annotation_track: AnnotationTrack,
  audio_layout: AudioLayout,
  audio_source: Option<&'a Path>,
  camera: Option<(&'a Path, BakedVideoExportOptions)>,
  camera_effects: Option<ExportCameraEffects>,
  captions: Option<&'a CaptionTrack>,
  cursor: Option<(&'a Path, CursorEffectSettings)>,
  keyboard: Option<(&'a Path, KeyboardEffectSettings)>,
  duration_ms: u64,
  output: &'a ScreenshotOutputSettings,
  screen: &'a Path,
  selection: &'a TrackSelection,
  size: (u32, u32),
  timeline: Option<&'a TimelinePlan>,
  video: VideoExportOptions,
}

/// The screen recording's own picture and what is baked into it.
pub(in crate::editor) struct Screen<'a> {
  pub camera_effects: Option<Arc<LiveCameraEffects>>,
  pub captions: Option<&'a CaptionTrack>,
  pub cursor: Option<(&'a Path, CursorEffectSettings)>,
  pub keyboard: Option<(&'a Path, KeyboardEffectSettings)>,
  pub duration_ms: u64,
  pub layout: AudioLayout,
  pub output: &'a ScreenshotOutputSettings,
  pub screen: &'a Path,
  pub selection: &'a TrackSelection,
  pub size: (u32, u32),
  pub timeline: Option<&'a TimelinePlan>,
  pub video: VideoExportOptions,
}

/// Where an export is written and who hears how far it has got.
pub(in crate::editor) struct Run<'a> {
  pub cancelled: &'a AtomicBool,
  pub destination: &'a Path,
  pub on_progress: &'a mut dyn FnMut(u64),
}

impl<'a> ExportPicture<'a> {
  /// The screen, with the camera's effects on it where the recording is of
  /// the camera alone.
  pub(in crate::editor) fn screen(screen: Screen<'a>) -> Self {
    Self {
      annotation_track: AnnotationTrack::Primary,
      audio_layout: screen.layout,
      audio_source: None,
      camera: None,
      camera_effects: screen.camera_effects.map(|live| ExportCameraEffects {
        live,
        on_screen: true,
      }),
      captions: screen.captions,
      cursor: screen.cursor,
      keyboard: screen.keyboard,
      duration_ms: screen.duration_ms,
      output: screen.output,
      screen: screen.screen,
      selection: screen.selection,
      size: screen.size,
      timeline: screen.timeline,
      video: screen.video,
    }
  }

  /// The screen with the camera baked into it, its effects on the camera.
  /// The camera's geometry is placed at the output's reference size; the
  /// export's scale reaches the compositor through the screen's video.
  pub(in crate::editor) fn baked(
    screen: Screen<'a>,
    camera: &'a RecordingCamera,
    overlay: screenwide_scenes::CameraOverlaySettings,
    camera_drop_shadow: bool,
  ) -> Self {
    let effects = screen.camera_effects.clone();
    let baked = BakedVideoExportOptions {
      camera_drop_shadow,
      camera_height: camera.height,
      camera_width: camera.width,
      overlay,
      screen_height: screen.output.height,
      screen_width: screen.output.width,
      video: VideoExportOptions {
        compression: screen.video.compression,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
    };
    Self {
      camera: Some((&camera.path, baked)),
      camera_effects: effects.map(|live| ExportCameraEffects {
        live,
        on_screen: false,
      }),
      ..Self::screen(screen)
    }
  }

  /// The camera recording as a file of its own, with `audio` beside it
  /// where it stands in for the screen: the file its audio is read from,
  /// which of its tracks, and how they are laid out.
  pub(in crate::editor) fn camera(
    camera: &'a RecordingCamera,
    output: &'a ScreenshotOutputSettings,
    timeline: Option<&'a TimelinePlan>,
    camera_effects: Option<Arc<LiveCameraEffects>>,
    compression: u8,
    audio: Option<(&'a Path, &'a TrackSelection, AudioLayout)>,
  ) -> Self {
    static SILENT: std::sync::LazyLock<TrackSelection> =
      std::sync::LazyLock::new(TrackSelection::default);
    let (audio_source, selection, audio_layout) = audio.map_or(
      (None, &*SILENT, AudioLayout::SeparateTracks),
      |(source, selection, layout)| (Some(source), selection, layout),
    );
    Self {
      annotation_track: AnnotationTrack::Camera,
      audio_layout,
      audio_source,
      camera: None,
      camera_effects: camera_effects.map(|live| ExportCameraEffects {
        live,
        on_screen: true,
      }),
      captions: None,
      cursor: None,
      keyboard: None,
      duration_ms: camera.duration_ms,
      output,
      screen: &camera.path,
      selection,
      size: (camera.width, camera.height),
      timeline,
      video: VideoExportOptions {
        compression,
        resolution_scale_percent: 100,
        source_scale_percent: 100,
      },
    }
  }

  /// Whether the compositor has to draw this picture rather than FFmpeg
  /// copying or re-encoding the recording: something is baked or drawn
  /// over it, or it is framed differently from how it was recorded.
  pub(in crate::editor) fn is_composed(&self) -> bool {
    self.camera.is_some()
      || self.cursor.is_some()
      || self.keyboard.is_some()
      || self.camera_effects.is_some()
      || self.captions.is_some()
      || self
        .timeline
        .is_some_and(|plan| !plan.annotation_clips().is_empty())
      || cursor_export::needs_composition(self.output, self.size.0, self.size.1)
  }

  /// The compositor's export of this picture, run as `run` says.
  pub(in crate::editor) fn request(self, run: Run<'a>) -> CursorExportRequest<'a> {
    CursorExportRequest {
      annotation_track: self.annotation_track,
      audio_layout: self.audio_layout,
      audio_source: self.audio_source,
      camera: self.camera,
      camera_effects: self.camera_effects,
      camera_on_top: screenwide_export::recording_model::CAMERA_IN_FRONT,
      cancelled: run.cancelled,
      captions: self.captions,
      cursor: self.cursor.map(|(path, _)| path),
      cursor_effects: self
        .cursor
        .map_or_else(CursorEffectSettings::default, |(_, settings)| settings),
      keyboard: self.keyboard.map(|(path, _)| path),
      keyboard_effects: self
        .keyboard
        .map_or_else(KeyboardEffectSettings::default, |(_, settings)| settings),
      destination: run.destination,
      duration_ms: self.duration_ms,
      height: self.size.1,
      on_progress: run.on_progress,
      output: self.output,
      screen: self.screen,
      selection: self.selection,
      #[cfg(target_os = "windows")]
      surface: crate::editor::preview_platform::recording_export_surface,
      timeline: self.timeline,
      video: self.video,
      width: self.size.0,
    }
  }
}
