// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a preset is laid out: which side the camera takes and how much of the
//! pair it is, and for a picture in picture which corner it sits over and how
//! big it is. The twin of `SceneVariant` in
//! `src/features/editor/recording/scenes/recording-scenes.ts`.

use serde::{Deserialize, Serialize};

/// How much of a side by side or stacked pair the camera takes.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SceneCameraSize {
  #[default]
  Third,
  TwoThirds,
}

/// The corner of the screen a picture in picture's camera sits over.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SceneCorner {
  TopLeft,
  TopRight,
  BottomLeft,
  #[default]
  BottomRight,
}

/// How big a picture in picture's camera is beside the screen.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SceneBubbleSize {
  Small,
  #[default]
  Large,
}

/// A preset's options, each its default where unset.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneVariant {
  /// The camera before the screen: on the left beside it, or above it.
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub swap: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub camera_size: Option<SceneCameraSize>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub corner: Option<SceneCorner>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub size: Option<SceneBubbleSize>,
}
