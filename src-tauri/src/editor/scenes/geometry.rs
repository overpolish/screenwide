// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where each preset puts the screen and the camera on the canvas. The twin of
//! `recordingScenePanes` in `src/features/editor/recording/scenes/recording-scene-geometry.ts`,
//! which draws the Scene panel's buttons and places the paused preview's
//! selection by the same rules.

use super::model::RecordingScenePreset;
use super::variant::{SceneBubbleSize, SceneCameraSize, SceneCorner, SceneVariant};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Rect {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

impl Rect {
  pub fn centre(self) -> (f64, f64) {
    (self.x + self.width / 2.0, self.y + self.height / 2.0)
  }

  /// This rect scaled by `factor` about its middle.
  pub fn scaled(self, factor: f64) -> Self {
    let (x, y) = self.centre();
    Self {
      x: x - self.width * factor / 2.0,
      y: y - self.height * factor / 2.0,
      width: self.width * factor,
      height: self.height * factor,
    }
  }
}

/// The boxes a preset gives the panes, `None` for a pane it hides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PresetPanes {
  pub screen: Option<Rect>,
  pub camera: Option<Rect>,
}

/// How far a picture in picture's camera reaches past the screen's corner
/// each way, as a share of its side.
const BUBBLE_OVERHANG: f64 = 0.4;

/// The camera's side in a picture in picture, as a share of the screen's
/// height.
fn bubble_share(size: SceneBubbleSize) -> f64 {
  match size {
    SceneBubbleSize::Small => 0.4,
    SceneBubbleSize::Large => 0.65,
  }
}

/// The camera's length in a pair over the screen's.
fn camera_ratio(size: SceneCameraSize) -> f64 {
  match size {
    SceneCameraSize::Third => 0.5,
    SceneCameraSize::TwoThirds => 2.0,
  }
}

/// The panes `preset` lays out on a `canvas` of output pixels, for a screen of
/// `screen_aspect` and a camera of `camera_aspect`, each width over height, as
/// `variant` has it; `None` for a full or screen-only scene, which keep the
/// recording's own place for the screen.
///
/// A margin of a ninth of the canvas's shorter side is kept clear. Side by
/// side and stacked keep each pane at its own shape, half a margin apart, the
/// camera half the screen's length, a third of the pair, or twice it, second
/// unless swapped; each pane is centred on the other across the pair, so the
/// camera is shown whole rather than cropped to match the screen's edge.
/// Picture in picture is the screen at its own aspect with the camera
/// reaching past the corner it sits over by the same share each way. Each
/// pair is scaled to the largest size the margin leaves room for and centred.
/// Camera only fills the canvas.
pub(super) fn preset_panes(
  preset: RecordingScenePreset,
  canvas: (f64, f64),
  (screen_aspect, camera_aspect): (f64, f64),
  variant: SceneVariant,
) -> Option<PresetPanes> {
  let (canvas_width, canvas_height) = canvas;
  let margin = canvas_width.min(canvas_height) / 9.0;
  let room_width = canvas_width - margin * 2.0;
  let room_height = canvas_height - margin * 2.0;
  let gap = margin / 2.0;
  let ratio = camera_ratio(variant.camera_size.unwrap_or_default());
  let (screen, camera) = match preset {
    RecordingScenePreset::Full | RecordingScenePreset::ScreenOnly => return None,
    RecordingScenePreset::CameraOnly => {
      return Some(PresetPanes {
        screen: None,
        camera: Some(Rect {
          x: 0.0,
          y: 0.0,
          width: canvas_width,
          height: canvas_height,
        }),
      })
    }
    RecordingScenePreset::SplitTwoThirds => {
      // The screen's width, held by the pair's width and by the taller pane.
      let width = ((room_width - gap) / (1.0 + ratio))
        .min(room_height / (1.0 / screen_aspect).max(ratio / camera_aspect));
      let height = width / screen_aspect;
      let camera_width = width * ratio;
      let camera_height = camera_width / camera_aspect;
      let x = (canvas_width - (width + camera_width + gap)) / 2.0;
      let (screen_x, camera_x) = if variant.swap {
        (x + camera_width + gap, x)
      } else {
        (x, x + width + gap)
      };
      (
        Rect {
          x: screen_x,
          y: (canvas_height - height) / 2.0,
          width,
          height,
        },
        Rect {
          x: camera_x,
          y: (canvas_height - camera_height) / 2.0,
          width: camera_width,
          height: camera_height,
        },
      )
    }
    RecordingScenePreset::Stacked => {
      // The screen's height, held by the pair's height and by the wider pane.
      let height = ((room_height - gap) / (1.0 + ratio))
        .min(room_width / screen_aspect.max(ratio * camera_aspect));
      let width = height * screen_aspect;
      let camera_height = height * ratio;
      let camera_width = camera_height * camera_aspect;
      let y = (canvas_height - (height + camera_height + gap)) / 2.0;
      let (screen_y, camera_y) = if variant.swap {
        (y + camera_height + gap, y)
      } else {
        (y, y + height + gap)
      };
      (
        Rect {
          x: (canvas_width - width) / 2.0,
          y: screen_y,
          width,
          height,
        },
        Rect {
          x: (canvas_width - camera_width) / 2.0,
          y: camera_y,
          width: camera_width,
          height: camera_height,
        },
      )
    }
    RecordingScenePreset::PictureInPicture => {
      let share = bubble_share(variant.size.unwrap_or_default());
      let reach = share * BUBBLE_OVERHANG;
      let height = (room_width / (screen_aspect + reach)).min(room_height / (1.0 + reach));
      let width = height * screen_aspect;
      let side = height * share;
      let overhang = side * BUBBLE_OVERHANG;
      let x = (canvas_width - (width + overhang)) / 2.0;
      let y = (canvas_height - (height + overhang)) / 2.0;
      let corner = variant.corner.unwrap_or_default();
      let right = matches!(corner, SceneCorner::TopRight | SceneCorner::BottomRight);
      let bottom = matches!(corner, SceneCorner::BottomLeft | SceneCorner::BottomRight);
      let screen_x = if right { x } else { x + overhang };
      let screen_y = if bottom { y } else { y + overhang };
      (
        Rect {
          x: screen_x,
          y: screen_y,
          width,
          height,
        },
        Rect {
          x: if right {
            screen_x + width - (side - overhang)
          } else {
            x
          },
          y: if bottom {
            screen_y + height - (side - overhang)
          } else {
            y
          },
          width: side,
          height: side,
        },
      )
    }
  };
  Some(PresetPanes {
    screen: Some(screen),
    camera: Some(camera),
  })
}
