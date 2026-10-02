// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scene clips arrange and frame the screen and the camera for a stretch of a
//! recording: a side by side, a picture in picture, or a zoom into the screen
//! where the recording already puts it. Like annotation clips they follow
//! source time, so a cut or a speed change never moves where a scene sits in
//! the recording. Outside every clip the recording keeps the composition it
//! already has, and the preview and the export arrange each frame through the
//! one function here, so the two always agree.

mod arrange;
mod framing;
mod geometry;
mod model;
mod motion;
mod placement;
mod template;
mod variant;

pub(crate) use arrange::{arrange, camera_target};
pub use model::RecordingSceneClip;
pub(crate) use model::{validate_clips, SceneRadius};
pub use motion::SceneMotion;
pub use template::SceneTemplate;

#[cfg(test)]
mod custom_tests;
#[cfg(test)]
mod preset_tests;
#[cfg(test)]
mod radius_tests;
#[cfg(test)]
mod tests;
