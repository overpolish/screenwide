// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as they are drawn: the effects pass (`gpu`, and
//! `cpu` for thumbnails and stills) draws the person over what the
//! background is, from the camera and the matte read beside it
//! (`matte_frames`), with Enhance's grade and sharpening. The app makes the
//! matte and the grade and loads them into a [`live::LiveCameraEffects`].

mod balance;
pub mod cpu;
pub mod gpu;
pub mod grade;
pub mod live;
mod look;
pub mod matte_frames;
pub mod preview;
mod sharpen;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What is behind the person on camera.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CameraBackground {
  /// The room, as recorded.
  #[default]
  Kept,
  /// Nothing: whatever the camera is drawn over shows through.
  Transparent,
  /// The room, blurred.
  Blurred,
  /// The picture chosen for it.
  Image,
}
