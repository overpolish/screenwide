// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_background::Background;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use super::boxes::{SceneBox, SceneBoxes};
use super::framing::SceneFraming;
use super::measure::Measure;
use super::rotation::SceneRotation;
use super::variant::SceneVariant;

/// The twin of `RecordingScenePreset` in
/// `src/features/editor/recording/scenes/recording-scenes.ts`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
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
  /// Neither the screen nor the camera: the background alone, with what is
  /// drawn over it, while the recording plays on underneath.
  Blank,
}

impl RecordingScenePreset {
  /// Whether the arrangement places a camera, which a scene can only do
  /// where the camera is drawn into the picture.
  fn needs_camera(self) -> bool {
    !matches!(self, Self::Full | Self::ScreenOnly | Self::Blank)
  }
}

/// The corner radius a scene gives each pane, like the recording's own: the
/// screen's in points of its capture (a percent of the pane in clips saved
/// before, and for a recording of the camera alone, whose screen pane is the
/// camera), the camera's a percent of its pane's shorter side. A pane
/// without one keeps the recording's radius.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SceneRadius {
  #[serde(
    default,
    deserialize_with = "crate::measure::optional_measure_or_bare_percent",
    skip_serializing_if = "Option::is_none"
  )]
  #[ts(optional)]
  pub screen: Option<Measure>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub camera: Option<f64>,
}

impl SceneRadius {
  pub(crate) fn is_valid(self) -> bool {
    self.screen.is_none_or(|screen| screen.is_valid(50.0))
      && self
        .camera
        .is_none_or(|camera| (0.0..=50.0).contains(&camera))
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingSceneClip {
  pub id: String,
  pub start_ms: u64,
  pub end_ms: u64,
  pub preset: RecordingScenePreset,
  /// What part of the screen fills its box; the whole of it where unset.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub screen: Option<SceneFraming>,
  /// What part of the camera fills its box. Unset, a full scene keeps the
  /// recording's own camera crop and an arrangement shows the whole camera.
  /// While the scene follows the face, its focus is where the face sits in
  /// the box instead: the middle of the picture centres the face.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub camera: Option<SceneFraming>,
  /// Whether the camera's framing follows the face found in it, so the face
  /// holds its place in the box however the person moves. Unset, the scene
  /// does as the camera does outside every scene.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub follow_face: Option<bool>,
  /// Whether the cursor shows in this scene, over the recording's own choice
  /// (`CursorEffectSettings::bake`), fading in or out as the scene arrives
  /// and leaves. Unset, the scene does as the recording does.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub cursor: Option<bool>,
  /// What the canvas sits on in this scene, over the recording's own,
  /// blooming in as the scene arrives and out as it leaves. Unset, the
  /// scene shows the recording's.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub background: Option<Background>,
  /// Set, a custom scene: the panes sit in these boxes instead of the
  /// preset's, which a reset returns them to.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub boxes: Option<SceneBoxes>,
  /// The panes' corner radii in this scene, which a reset takes away.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub radius: Option<SceneRadius>,
  /// How far the scene pads its screen's box: the box grows outwards by it
  /// on every side, filled with the colour found behind the recording's
  /// content, as the recording's own inset pads its crop. Points of the
  /// recording's capture as the scene draws it; a percent of the box's
  /// shorter side in clips saved before. None where unset.
  #[serde(
    default,
    deserialize_with = "crate::measure::optional_measure_or_bare_share",
    skip_serializing_if = "Option::is_none"
  )]
  #[ts(optional)]
  pub inset: Option<Measure>,
  /// How the scene turns the screen in depth, with everything drawn on it;
  /// upright where unset.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub rotation: Option<SceneRotation>,
  /// How the preset is laid out, each option its default where unset.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub variant: Option<SceneVariant>,
  /// The layout a preset scene had before one of its panes was turned off,
  /// which turning the pane back on returns to. Unset where no pane was
  /// turned off, and once a layout is chosen or a pane moved by hand.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub layout_before: Option<RecordingScenePreset>,
  /// The shape a preset scene crops each pane to, which its layout fits the
  /// pane at, so a crop keeps the preset and the layout makes room for it.
  /// Unset, each pane keeps the shape of its picture as the recording crops
  /// it. A custom scene's boxes carry their own shapes instead.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub shapes: Option<SceneShapes>,
  /// Set on a zoom the auto zoom made that nobody has edited since. Making
  /// the auto zooms again replaces these and leaves every other scene alone.
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  #[ts(type = "true")]
  pub auto: bool,
}

/// A preset scene's own crop shapes, each width over height; a pane without
/// one keeps its picture's shape.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct SceneShapes {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub screen: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub camera: Option<f64>,
}

impl SceneShapes {
  /// Whether every shape is one a box can take: neither a sliver nor a line.
  fn is_valid(self) -> bool {
    [self.screen, self.camera]
      .into_iter()
      .flatten()
      .all(|shape| shape.is_finite() && (0.02..=50.0).contains(&shape))
  }
}

impl RecordingSceneClip {
  /// Whether the scene places a camera, which it can only do where the
  /// camera is drawn into the picture. One it hides it does not place.
  pub fn needs_camera(&self) -> bool {
    self.boxes.map_or(self.preset.needs_camera(), |boxes| {
      boxes.camera.is_some() && !boxes.camera_hidden
    })
  }

  /// One where the camera is drawn in front of the screen, zero where a
  /// custom scene puts it behind.
  pub fn camera_front(&self) -> f64 {
    if self.boxes.is_some_and(|boxes| boxes.camera_behind) {
      0.0
    } else {
      1.0
    }
  }

  /// Whether the scene shows the cursor over the recording's own choice,
  /// `None` where it makes none. A blank scene never does: the cursor moves
  /// over a screen it hides.
  pub fn cursor_choice(&self) -> Option<bool> {
    if self.preset == RecordingScenePreset::Blank {
      Some(false)
    } else {
      self.cursor
    }
  }
}

/// The furthest a scene saved before paddings had a unit pads its screen,
/// in percent of the box's shorter side.
const MAX_SCENE_INSET_PERCENT: f64 = 50.0;

/// Whether a recording's scene clips can be played: each spans time, has an
/// id of its own and framings a scene can hold, and they run in order without
/// overlapping, because only one scene can be on screen at a time. Bounds are
/// half open, so one clip may end exactly where the next begins. Every box's
/// middle stays on the canvas, so it can always be found and dragged back.
pub fn validate_clips(clips: &[RecordingSceneClip]) -> Result<(), String> {
  validated(clips, SceneBoxes::is_valid)
}

/// Whether the scene the crop tool holds can be drawn: a clip as
/// [`validate_clips`] asks, except that its boxes may lie past the canvas's
/// edge, where a preset held while cropped can leave a pane.
pub fn validate_held_clip(clip: &RecordingSceneClip) -> Result<(), String> {
  validated(std::slice::from_ref(clip), SceneBoxes::is_drawable)
}

fn validated(
  clips: &[RecordingSceneClip],
  boxes_hold: fn(SceneBoxes) -> bool,
) -> Result<(), String> {
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
      || clip.boxes.is_some_and(|boxes| !boxes_hold(boxes))
      || clip.radius.is_some_and(|radius| !radius.is_valid())
      || clip
        .inset
        .is_some_and(|inset| !inset.is_valid(MAX_SCENE_INSET_PERCENT))
      || clip.rotation.is_some_and(|rotation| !rotation.is_valid())
      || clip.shapes.is_some_and(|shapes| !shapes.is_valid())
    {
      return Err("The scene clips are invalid".to_owned());
    }
    previous_end = clip.end_ms;
  }
  Ok(())
}
