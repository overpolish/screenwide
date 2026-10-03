// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

use super::framing::SceneFraming;
use super::geometry::Rect;
use super::variant::SceneVariant;

/// The twin of `RecordingScenePreset` in
/// `src/features/editor/recording/scenes/recording-scenes.ts`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecordingScenePreset {
  /// The screen and the camera where the recording's own composition puts
  /// them: a scene that only reframes, such as a zoom into the screen.
  Full,
  /// The screen and the camera side by side, the camera a third of the pair
  /// unless its variant says otherwise.
  SplitTwoThirds,
  /// The screen with the camera over one of its corners, overlapping the
  /// screen and the background around it.
  PictureInPicture,
  /// The screen above the camera, the camera a third of the pair unless its
  /// variant says otherwise.
  Stacked,
  /// The camera alone, filling the canvas.
  CameraOnly,
  /// The screen alone, where the recording's own composition puts it.
  ScreenOnly,
}

impl RecordingScenePreset {
  /// Whether the arrangement places a camera, which a scene can only do
  /// where the camera is drawn into the picture.
  fn needs_camera(self) -> bool {
    !matches!(self, Self::Full | Self::ScreenOnly)
  }
}

/// One pane's box in a custom scene, as shares of the canvas's width and
/// height.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct SceneBox {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

/// The furthest a box may grow and shrink, as shares of the canvas. The twins
/// of the limits in `recording-scene-custom.ts`.
const MAX_BOX_SHARE: f64 = 4.0;
const MIN_BOX_SHARE: f64 = 0.02;

impl SceneBox {
  /// A box with some size whose middle stays on the canvas, so it can always
  /// be found and dragged back.
  pub(crate) fn is_valid(self) -> bool {
    let (x, y) = (self.x + self.width / 2.0, self.y + self.height / 2.0);
    (0.0..=1.0).contains(&x)
      && (0.0..=1.0).contains(&y)
      && self.width > 0.0
      && self.height > 0.0
      && self.width <= MAX_BOX_SHARE
      && self.height <= MAX_BOX_SHARE
  }

  /// The box on a `canvas` of output pixels.
  pub(crate) fn on(self, (width, height): (f64, f64)) -> Rect {
    Rect {
      x: self.x * width,
      y: self.y * height,
      width: self.width * width,
      height: self.height * height,
    }
  }

  /// The box a select gesture leaves: its corner moved `delta` shares of the
  /// canvas and its size scaled by `scale`, its shape kept. It stays between
  /// the smallest and largest a box may be, its middle on the canvas.
  pub(crate) fn moved(self, (dx, dy): (f64, f64), scale: f64) -> Self {
    let low = MIN_BOX_SHARE / self.width.min(self.height).max(f64::EPSILON);
    let high = MAX_BOX_SHARE / self.width.max(self.height).max(f64::EPSILON);
    let scale = scale.max(low).min(high);
    let width = self.width * scale;
    let height = self.height * scale;
    Self {
      x: (self.x + dx).clamp(-width / 2.0, 1.0 - width / 2.0),
      y: (self.y + dy).clamp(-height / 2.0, 1.0 - height / 2.0),
      width,
      height,
    }
  }
}

/// Where a custom scene puts its panes in place of its preset's boxes, and
/// which of them is drawn in front. The camera keeps the recording's own box
/// where it has none, and is drawn in front unless `camera_behind` says
/// otherwise.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneBoxes {
  pub screen: SceneBox,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub camera: Option<SceneBox>,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub camera_behind: bool,
}

impl SceneBoxes {
  /// Whether every box has some size and its middle on the canvas.
  pub(crate) fn is_valid(self) -> bool {
    [Some(self.screen), self.camera]
      .into_iter()
      .flatten()
      .all(SceneBox::is_valid)
  }
}

/// The corner radius a scene gives each pane, as a share of the pane's
/// shorter side in percent, like the recording's own. A pane without one
/// keeps the recording's radius.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SceneRadius {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub screen: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub camera: Option<f64>,
}

impl SceneRadius {
  pub(crate) fn is_valid(self) -> bool {
    [self.screen, self.camera]
      .into_iter()
      .flatten()
      .all(|radius| (0.0..=50.0).contains(&radius))
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSceneClip {
  pub id: String,
  pub start_ms: u64,
  pub end_ms: u64,
  pub preset: RecordingScenePreset,
  /// What part of the screen fills its box; the whole of it where unset.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub screen: Option<SceneFraming>,
  /// What part of the camera fills its box. Unset, a full scene keeps the
  /// recording's own camera crop and an arrangement shows the whole camera.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub camera: Option<SceneFraming>,
  /// Set, a custom scene: the panes sit in these boxes instead of the
  /// preset's, which a reset returns them to.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub boxes: Option<SceneBoxes>,
  /// The panes' corner radii in this scene, which a reset takes away.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub radius: Option<SceneRadius>,
  /// How the preset is laid out, each option its default where unset.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub variant: Option<SceneVariant>,
  /// Set on a zoom the auto zoom made that nobody has edited since. Making
  /// the auto zooms again replaces these and leaves every other scene alone.
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub auto: bool,
}

impl RecordingSceneClip {
  /// Whether the scene places a camera, which it can only do where the
  /// camera is drawn into the picture.
  pub(crate) fn needs_camera(&self) -> bool {
    self
      .boxes
      .map_or(self.preset.needs_camera(), |boxes| boxes.camera.is_some())
  }

  /// One where the camera is drawn in front of the screen, zero where a
  /// custom scene puts it behind.
  pub(crate) fn camera_front(&self) -> f64 {
    if self.boxes.is_some_and(|boxes| boxes.camera_behind) {
      0.0
    } else {
      1.0
    }
  }
}

/// Whether a recording's scene clips can be played: each spans time, has an
/// id of its own and framings a scene can hold, and they run in order without
/// overlapping, because only one scene can be on screen at a time. Bounds are
/// half open, so one clip may end exactly where the next begins.
pub(crate) fn validate_clips(clips: &[RecordingSceneClip]) -> Result<(), String> {
  let mut ids = std::collections::HashSet::with_capacity(clips.len());
  let mut previous_end = 0;
  for clip in clips {
    if clip.id.is_empty()
      || !ids.insert(clip.id.as_str())
      || clip.end_ms <= clip.start_ms
      || clip.start_ms < previous_end
      || [clip.screen, clip.camera]
        .into_iter()
        .flatten()
        .any(|framing| !framing.is_valid())
      || clip.boxes.is_some_and(|boxes| !boxes.is_valid())
      || clip.radius.is_some_and(|radius| !radius.is_valid())
    {
      return Err("The scene clips are invalid".to_owned());
    }
    previous_end = clip.end_ms;
  }
  Ok(())
}
