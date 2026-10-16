// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The paths of the canvas shader a frame takes besides its annotations, a
//! bit each as `canvas_features` in `preview.wgsl` reads them.
//!
//! The canvas shader holds every path a frame might take, and the GPU
//! compiler sizes each pixel's registers for the deepest of them whether a
//! frame takes it or not. A 3600 x 2338 export frame with only a cursor drew
//! in 24 ms with every path kept and in under 1 ms with only the paths it took,
//! so a frame is drawn by the variant that keeps only its own paths
//! ([`super::canvas_pipelines`]). A bit is set wherever its path might do
//! anything for the frame: each one cleared stands for a path the shader would
//! have left at its first early return, so the slim variant draws the same
//! picture.

use super::Constants;

/// A set of the canvas shader's optional paths.
pub(super) type FeatureMask = u32;

/// Every path: the shader as written, which the build precompiles.
pub(super) const ALL_FEATURES: FeatureMask = u32::MAX;

const CURSOR: FeatureMask = 1;
const CAMERA: FeatureMask = 2;
const KEYBOARD: FeatureMask = 4;
const MAGNIFIER: FeatureMask = 8;
const SCENE_MOTION: FeatureMask = 16;
const TURN: FeatureMask = 32;
const FOCUS: FeatureMask = 64;
const CROP_PREVIEW: FeatureMask = 128;
const BACKGROUND_PICTURE: FeatureMask = 256;
const BACKGROUND_MESH: FeatureMask = 512;
const BACKGROUND_GENERATOR: FeatureMask = 1024;

/// The line every slim variant replaces.
const ALL_LINE: &str = "const canvas_features: u32 = 0xffffffffu;";

#[cfg(test)]
thread_local! {
  /// For tests on this thread: draw with every path kept (`Some(false)`), or
  /// with the slim variant, compiled where it is drawn (`Some(true)`), rather
  /// than as the app draws.
  pub(super) static TEST_SLIM: std::cell::Cell<Option<bool>> =
    const { std::cell::Cell::new(None) };
}

/// The paths the frame `values` describes takes, with or without a focus and
/// the shortcut strip. Each test is the negation of the early return the
/// shader takes when its path does nothing; a NaN keeps a path, as the shader
/// would not have returned early on it either.
pub(super) fn of(values: &Constants, focus: bool, keyboard: bool) -> FeatureMask {
  if !cfg!(target_os = "macos") {
    // Windows compiles a variant from WGSL with DXC, which takes seconds a
    // variant, so it keeps to the variants the build compiled.
    return ALL_FEATURES;
  }
  #[cfg(test)]
  if TEST_SLIM.get() == Some(false) {
    return ALL_FEATURES;
  }
  let mesh = values.options[1] != 0;
  let generator = values.background_options[1] != 0;
  [
    (values.cursor_options[1] != 0, CURSOR),
    (values.camera_effects[1] != 0.0, CAMERA),
    (keyboard, KEYBOARD),
    (
      values.magnifier[2] > 0.0 || values.magnifier[2].is_nan(),
      MAGNIFIER,
    ),
    // The shader averages `max(u32(w), 1)` steps; under two is one.
    (
      values.scene_motion[3] >= 2.0 || values.scene_motion[3].is_nan(),
      SCENE_MOTION,
    ),
    (
      values.scene_turn[0][3] != 0.0 || values.scene_turn[6][3] != 0.0,
      TURN,
    ),
    (focus, FOCUS),
    (values.crop_preview_effects[1] != 0.0, CROP_PREVIEW),
    (values.background_options[0] != 0, BACKGROUND_PICTURE),
    (mesh && !generator, BACKGROUND_MESH),
    (mesh && generator, BACKGROUND_GENERATOR),
  ]
  .into_iter()
  .filter(|(kept, _)| *kept)
  .fold(0, |features, (_, feature)| features | feature)
}

/// `shader` keeping only `features`.
pub(super) fn keeping(shader: String, features: FeatureMask) -> String {
  debug_assert!(
    shader.contains(ALL_LINE),
    "the canvas shader declares the paths it keeps"
  );
  if features == ALL_FEATURES {
    return shader;
  }
  shader.replacen(
    ALL_LINE,
    &format!("const canvas_features: u32 = {features}u;"),
    1,
  )
}
