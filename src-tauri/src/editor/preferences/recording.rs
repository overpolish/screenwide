// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use ts_rs::TS;

/// A baked camera's placement with the screen canvas and camera size it was
/// measured against. Placement is in screen pixels, so another recording can
/// only reuse it once it is scaled onto that recording's own geometry, which
/// the editor does when it seeds the next capture.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RememberedCameraOverlay {
  pub camera_height: u32,
  pub camera_width: u32,
  pub overlay: CameraOverlaySettings,
  pub screen_height: u32,
  pub screen_width: u32,
}

/// Recording export choices carried to the next recording.
///
/// `None` means the choice has never been offered by an exported recording,
/// so the editor keeps its own default. An export only overwrites the choices
/// it offered: exporting a recording without a camera must not forget whether
/// the camera was baked last time there was one.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingExportChoices {
  pub bake_camera: Option<bool>,
  pub camera_compression: Option<u8>,
  pub camera_overlay: Option<RememberedCameraOverlay>,
  pub camera_resolution_scale_percent: Option<u16>,
  pub collapse_audio: Option<bool>,
  pub compression: Option<u8>,
  pub delete_project_after_export: Option<bool>,
  /// The output scale as a share of the "Original" scale, which is how the
  /// choice is labelled. A raw percentage would pick a different size on a
  /// display with another scale factor.
  pub resolution_scale_ratio: Option<f64>,
}

impl RecordingExportChoices {
  /// The choices a finished recording export offered, and nothing else.
  pub(in crate::editor) fn offered_by(
    artifact: &EditorArtifact,
    options: &RecordingExportOptions,
  ) -> Self {
    let EditorArtifact::Recording {
      camera,
      primary_kind,
      source_scale_percent,
      ..
    } = artifact
    else {
      return Self::default();
    };
    let has_picture = *primary_kind != PrimaryRecordingKind::Audio;
    let can_compress = has_picture && media_preview::supports_compression();
    let camera = camera.as_ref();
    // Matches the first entry of the editor's resolution choices.
    let original_scale = match primary_kind {
      PrimaryRecordingKind::Camera => 100,
      _ => (*source_scale_percent).clamp(100, 400),
    };
    Self {
      bake_camera: (camera.is_some() && options.include_primary_video && options.include_camera)
        .then_some(options.bake_camera),
      camera_compression: (camera.is_some() && can_compress).then_some(options.camera_compression),
      camera_overlay: camera
        .filter(|_| has_picture)
        .map(|camera| RememberedCameraOverlay {
          camera_height: camera.height,
          camera_width: camera.width,
          overlay: options.camera_overlay,
          screen_height: options.recording_output.primary.height,
          screen_width: options.recording_output.primary.width,
        }),
      camera_resolution_scale_percent: camera
        .is_some()
        .then_some(options.camera_resolution_scale_percent),
      collapse_audio: (options.enabled_stream_indices.len() > 1).then_some(options.collapse_audio),
      compression: can_compress.then_some(options.compression),
      delete_project_after_export: Some(options.delete_project_after_export),
      resolution_scale_ratio: has_picture
        .then(|| f64::from(options.resolution_scale_percent) / f64::from(original_scale)),
    }
  }

  pub(super) fn merged(self, newer: Self) -> Self {
    Self {
      bake_camera: newer.bake_camera.or(self.bake_camera),
      camera_compression: newer.camera_compression.or(self.camera_compression),
      camera_overlay: newer.camera_overlay.or(self.camera_overlay),
      camera_resolution_scale_percent: newer
        .camera_resolution_scale_percent
        .or(self.camera_resolution_scale_percent),
      collapse_audio: newer.collapse_audio.or(self.collapse_audio),
      compression: newer.compression.or(self.compression),
      delete_project_after_export: newer
        .delete_project_after_export
        .or(self.delete_project_after_export),
      resolution_scale_ratio: newer.resolution_scale_ratio.or(self.resolution_scale_ratio),
    }
  }

  /// Drops any choice the export would refuse, so a damaged preferences file
  /// cannot seed an editor that then fails to save.
  pub(super) fn sanitized(self) -> Self {
    let compression = |value: &u8| *value <= 4;
    Self {
      camera_compression: self.camera_compression.filter(compression),
      camera_overlay: self.camera_overlay.filter(|remembered| {
        remembered.camera_width > 0
          && remembered.camera_height > 0
          && validate_camera_overlay(
            remembered.overlay,
            (remembered.screen_width, remembered.screen_height),
          )
          .is_ok()
      }),
      camera_resolution_scale_percent: self
        .camera_resolution_scale_percent
        .filter(|scale| validate_camera_resolution_scale(*scale).is_ok()),
      compression: self.compression.filter(compression),
      resolution_scale_ratio: self
        .resolution_scale_ratio
        .filter(|ratio| ratio.is_finite() && *ratio > 0.0 && *ratio <= 1.0),
      ..self
    }
  }
}

/// What a finished recording export leaves behind for the next recording.
pub(in crate::editor) struct CompletedRecordingExport {
  pub choices: RecordingExportChoices,
  pub keyboard: keyboard_effects::KeyboardEffectSettings,
  pub output: RecordingOutputSettings,
}

pub(in crate::editor) fn load_keyboard_effects(
  app: &AppHandle,
) -> keyboard_effects::KeyboardEffectSettings {
  load_preferences(app)
    .map(|preferences| preferences.keyboard_effects.normalized())
    .unwrap_or_default()
}

pub(in crate::editor) fn load_recording_choices(app: &AppHandle) -> RecordingExportChoices {
  load_preferences(app)
    .map(|preferences| preferences.recording_choices.sanitized())
    .unwrap_or_default()
}

pub(super) fn remember_keyboard_effects(
  app: &AppHandle,
  effects: keyboard_effects::KeyboardEffectSettings,
) -> Result<(), String> {
  let effects = effects.normalized();
  *app
    .state::<EditorState>()
    .keyboard_effects
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = effects;
  let mut preferences = load_preferences(app).unwrap_or_default();
  preferences.keyboard_effects = effects;
  store_preferences(app, &preferences)
}

pub(super) fn remember_recording_choices(
  app: &AppHandle,
  offered: RecordingExportChoices,
) -> Result<(), String> {
  let mut preferences = load_preferences(app).unwrap_or_default();
  let choices = preferences
    .recording_choices
    .sanitized()
    .merged(offered.sanitized());
  *app
    .state::<EditorState>()
    .recording_choices
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = choices;
  preferences.recording_choices = choices;
  store_preferences(app, &preferences)
}
