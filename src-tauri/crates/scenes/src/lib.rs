// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scene clips arrange and frame the screen and the camera for a stretch of a
//! recording: a side by side, a picture in picture, or a zoom into the screen
//! where the recording already puts it. Like annotation clips they follow
//! source time, so a cut or a speed change never moves where a scene sits in
//! the recording. Outside every clip the recording keeps the composition it
//! already has. Applying a clip to a frame's output settings is the app's.

mod camera_overlay;
pub mod framing;
pub mod geometry;
pub mod model;
pub mod motion;
pub mod placement;
pub mod template;
pub mod variant;

pub use camera_overlay::CameraOverlaySettings;
pub use framing::SceneFraming;
pub use model::{validate_clips, RecordingSceneClip, RecordingScenePreset, SceneRadius};
pub use motion::SceneMotion;
pub use template::SceneTemplate;
