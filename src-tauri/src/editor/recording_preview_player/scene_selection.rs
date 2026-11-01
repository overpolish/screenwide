// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The selection chrome over a pane a scene places, worked out from the
//! arrangement the frame on screen was drawn in. The webview only learns the
//! playhead once that frame is shown, so a box it placed would trail the
//! picture by a frame or more as a scrub or a scene's ease moves the pane.

use super::{PlayerSources, PreviewCompositionSettings};
use crate::editor::preview_workspace_model::output_canvas;
use screenwide_scenes::SceneRotation;
use screenwide_workspace::editing::PreviewSelection;

/// The selection the webview last sent, the moment of the frame on screen,
/// source milliseconds and how much source time it covers, and the
/// selection last drawn over it.
#[derive(Default)]
pub(super) struct DrawnSelection {
  given: Option<PreviewSelection>,
  moment: Option<(u64, f32)>,
  shown: Option<PreviewSelection>,
}

/// `given` placed as the frame drawn in `arranged` has its pane, where a
/// scene placed that frame or `given` was sent for one that did. The screen
/// and a camera drawn into it take their box, picture and radius from the
/// arrangement, and the screen its turn; a crop window, the frame handle, an
/// overlay and a camera pane of its own are the webview's to place.
/// `sources` are the screen's and the camera's source sizes, which the
/// pictures' heights follow.
pub(super) fn placed_selection(
  given: PreviewSelection,
  arranged: &PreviewCompositionSettings,
  placed: bool,
  sources: [Option<(u32, u32)>; 2],
) -> PreviewSelection {
  if given.crop_mode != 0 || (!placed && given.framed == 0) {
    return given;
  }
  let screen = &arranged.recording_output.primary;
  let (width, height) = output_canvas(screen);
  let share = |x: f64, y: f64, box_width: f64, box_height: f64| {
    [
      x / width,
      y / height,
      box_width / width,
      box_height / height,
    ]
  };
  let aspect = |(source_width, source_height): (u32, u32)| {
    f64::from(source_height) / f64::from(source_width.max(1))
  };
  let (rect, image, radius_percent) = match (given.pane_index, given.layer_id) {
    (0, 0) => {
      let Some(source) = sources[0] else {
        return given;
      };
      let image_width = screen.image_width.max(1.0);
      (
        share(
          screen.crop_x,
          screen.crop_y,
          screen.crop_width,
          screen.crop_height,
        ),
        share(
          screen.image_x,
          screen.image_y,
          image_width,
          image_width * aspect(source),
        ),
        screen.radius_percent,
      )
    }
    (0, 1) if arranged.bake_camera => {
      let Some(source) = sources[1] else {
        return given;
      };
      let camera = arranged.camera_overlay;
      let camera_height = camera.camera_width * aspect(source);
      (
        share(
          camera.frame_x,
          camera.frame_y,
          camera.frame_width,
          camera.frame_height,
        ),
        share(
          camera.camera_x - camera.camera_width / 2.0,
          camera.camera_y - camera_height / 2.0,
          camera.camera_width,
          camera_height,
        ),
        camera.radius_percent,
      )
    }
    _ => return given,
  };
  let selection = PreviewSelection {
    framed: if placed { given.framed } else { 0 },
    x: rect[0],
    y: rect[1],
    width: rect[2],
    height: rect[3],
    image_x: image[0],
    image_y: image[1],
    image_width: image[2],
    image_height: image[3],
    radius_percent,
    ..given
  };
  if given.turnable == 0 {
    selection
  } else {
    selection.turned_by(Some(screen.rotation.unwrap_or(SceneRotation::UPRIGHT)))
  }
}

impl PlayerSources {
  fn pane_source_sizes(&self) -> [Option<(u32, u32)>; 2] {
    [
      self
        .playback_layout
        .panes
        .first()
        .map(|pane| (pane.source_width, pane.source_height)),
      self.camera_source_size(),
    ]
  }

  /// The selection to draw over the frame `moment` shows, laid out as
  /// `arranged`, `placed` by a scene or not; `None` where it is already
  /// drawn or nothing is selected.
  pub(super) fn selection_for_frame(
    &self,
    arranged: &PreviewCompositionSettings,
    placed: bool,
    moment: (u64, f32),
  ) -> Option<PreviewSelection> {
    let mut drawn = self
      .scenes
      .selection
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    drawn.moment = Some(moment);
    let selection = placed_selection(drawn.given?, arranged, placed, self.pane_source_sizes());
    if drawn.shown == Some(selection) {
      return None;
    }
    drawn.shown = Some(selection);
    Some(selection)
  }

  /// Takes `given` as the webview's selection and answers it placed over
  /// the frame on screen, or over `position_ms` before one is.
  pub(super) fn selection_for_layout(
    &self,
    given: Option<PreviewSelection>,
    position_ms: u64,
  ) -> Option<PreviewSelection> {
    let mut drawn = self
      .scenes
      .selection
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    drawn.given = given;
    let (source_ms, frame_ms) = drawn.moment.unwrap_or((position_ms, 0.0));
    let composition = self
      .composition_settings
      .as_ref()
      .and_then(|settings| settings.read().ok().map(|settings| settings.clone()));
    let selection = given.map(|given| {
      let Some(mut arranged) = composition else {
        return given;
      };
      let placed = self.arrange_scene(&mut arranged, source_ms, frame_ms);
      placed_selection(given, &arranged, placed, self.pane_source_sizes())
    });
    drawn.shown = selection;
    selection
  }
}

#[cfg(test)]
mod tests {
  use super::super::{CameraOverlaySettings, RecordingOutputSettings};
  use super::*;
  use screenwide_workspace::editing::FRAMED_FIXED;

  fn composition() -> PreviewCompositionSettings {
    let camera = screenwide_output::fixtures::test_output_settings(640, 480);
    let mut primary = screenwide_output::fixtures::test_output_settings(1_600, 900);
    primary.crop_x = 400.0;
    primary.crop_y = 225.0;
    primary.crop_width = 800.0;
    primary.crop_height = 450.0;
    primary.image_x = 400.0;
    primary.image_y = 225.0;
    primary.image_width = 800.0;
    primary.rotation = Some(SceneRotation {
      x: 0.0,
      y: 20.0,
      z: 0.0,
    });
    PreviewCompositionSettings {
      bake_camera: false,
      camera_overlay: CameraOverlaySettings {
        camera_x: 0.0,
        camera_y: 0.0,
        camera_width: 0.0,
        frame_height: 0.0,
        frame_width: 0.0,
        frame_x: 0.0,
        frame_y: 0.0,
        radius_percent: 0.0,
        follow_face: false,
      },
      recording_output: RecordingOutputSettings { camera, primary },
    }
  }

  fn screen() -> PreviewSelection {
    PreviewSelection {
      framed: FRAMED_FIXED,
      width: 1.0,
      height: 1.0,
      ..PreviewSelection::default()
    }
    .turned_by(Some(SceneRotation::UPRIGHT))
  }

  #[test]
  fn a_placed_screen_takes_the_box_and_turn_its_frame_was_drawn_with() {
    let placed = placed_selection(screen(), &composition(), true, [Some((1_920, 1_080)), None]);
    assert_eq!(
      (placed.x, placed.y, placed.width, placed.height),
      (0.25, 0.25, 0.5, 0.5)
    );
    assert_eq!((placed.image_width, placed.image_height), (0.5, 0.5));
    assert_eq!(placed.framed, FRAMED_FIXED);
    assert_eq!(placed.rotation_y, 20.0);
  }

  #[test]
  fn a_frame_past_the_scene_lets_the_box_go_free() {
    let placed = placed_selection(
      screen(),
      &composition(),
      false,
      [Some((1_920, 1_080)), None],
    );
    assert_eq!(placed.framed, 0);
    assert_eq!(placed.width, 0.5);
  }

  #[test]
  fn a_selection_outside_every_scene_and_a_crop_stay_as_sent() {
    let free = PreviewSelection {
      framed: 0,
      ..screen()
    };
    assert_eq!(
      placed_selection(free, &composition(), false, [Some((1_920, 1_080)), None]),
      free
    );
    let crop = PreviewSelection {
      crop_mode: 1,
      ..screen()
    };
    assert_eq!(
      placed_selection(crop, &composition(), true, [Some((1_920, 1_080)), None]),
      crop
    );
  }

  #[test]
  fn a_scene_padding_its_screen_places_the_padded_box() {
    let clip: screenwide_scenes::RecordingSceneClip = serde_json::from_value(serde_json::json!({
      "id": "p", "startMs": 0, "endMs": 10_000, "preset": "full",
      "screen": { "focusX": 0.5, "focusY": 0.5, "zoom": 2.0 }, "inset": 0.1
    }))
    .unwrap();
    let mut arranged = composition();
    let output = &mut arranged.recording_output.primary;
    let placed = screenwide_export::scenes::arrange(&[clip], &[], 5_000, 0.0, output, None);
    let [x, y, width, height] = output.scene_window.expect("the scene pads its box");
    let pad = 0.1 * width.min(height);
    let shown = placed_selection(screen(), &arranged, placed, [Some((1_920, 1_080)), None]);
    // The handles go round the padding, where the webview's hit target is.
    let padded = [(x - pad) / 1_600.0, (y - pad) / 900.0];
    assert!([shown.x, shown.y]
      .iter()
      .zip(padded)
      .all(|(a, b)| (a - b).abs() < 1e-9));
  }
}
