// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A scene's layout in the formats it is laid out differently in: where its
//! panes go and how they are framed. When and how long it plays, its
//! background, cursor and following the face are the scene's in every
//! format. Kept with the recording's formats, by the scene's id.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::boxes::SceneBoxes;
use super::framing::SceneFraming;
use super::measure::Measure;
use super::model::{RecordingScenePreset, SceneRadius, SceneShapes};
use super::rotation::SceneRotation;
use super::variant::SceneVariant;

/// One scene's layout in some formats, each field as `RecordingSceneClip`
/// has it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SceneFormatLayout {
  pub preset: RecordingScenePreset,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub layout_before: Option<RecordingScenePreset>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub variant: Option<SceneVariant>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub boxes: Option<SceneBoxes>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub shapes: Option<SceneShapes>,
  #[serde(
    default,
    deserialize_with = "crate::measure::optional_measure_or_bare_share",
    skip_serializing_if = "Option::is_none"
  )]
  #[ts(optional)]
  pub inset: Option<Measure>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub radius: Option<SceneRadius>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub rotation: Option<SceneRotation>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub screen: Option<SceneFraming>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub camera: Option<SceneFraming>,
}
