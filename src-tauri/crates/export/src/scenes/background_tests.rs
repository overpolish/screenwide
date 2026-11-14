// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_background::Background;
use screenwide_output::fixtures::test_output_settings;
use screenwide_output::ScreenshotOutputSettings;
use screenwide_scenes::RecordingSceneClip;

const GREEN: &str = "#00AA44";

/// A scene from 2 s to 8 s with a green background of its own, its screen
/// in a box `width` shares of the canvas wide, its left edge at `x`. The
/// recording's own screen is 0.8 of the 1600 by 900 canvas wide, 2:1.
fn green_scene(x: f64, width: f64) -> RecordingSceneClip {
  serde_json::from_value(serde_json::json!({
    "id": "green", "startMs": 2_000, "endMs": 8_000, "preset": "full",
    "boxes": { "screen": {
      "x": x, "y": 0.1, "width": width, "height": width * 1_600.0 / 2.0 / 900.0,
    } },
    "background": { "kind": "solid", "color": GREEN },
  }))
  .unwrap()
}

fn arranged(clip: &RecordingSceneClip, source_ms: u64) -> ScreenshotOutputSettings {
  let mut output = test_output_settings(1_600, 900);
  arrange(
    std::slice::from_ref(clip),
    &[],
    source_ms,
    0.0,
    &mut output,
    None,
  );
  output
}

fn green() -> Background {
  Background::Solid {
    color: GREEN.to_owned(),
  }
}

#[test]
fn a_scene_wears_its_own_background_and_the_recording_its_own_outside() {
  let clip = green_scene(0.05, 0.9);
  let inside = arranged(&clip, 5_000);
  assert_eq!(inside.background(), green());
  assert_eq!(inside.background_turn, None);
  let outside = arranged(&clip, 1_000);
  assert_eq!(
    outside.background(),
    test_output_settings(1_600, 900).background()
  );
  assert_eq!(outside.background_turn, None);
}

#[test]
fn the_background_blooms_from_where_the_screen_lands_before_the_clip_starts() {
  let clip = green_scene(0.05, 0.9);
  // The background starts a hundred milliseconds ahead of the panes.
  let output = arranged(&clip, 1_950);
  assert_eq!(output.background(), green());
  let turn = output.background_turn.expect("the background is changing");
  assert_eq!(
    turn.outgoing.background_color,
    test_output_settings(1_600, 900).background_color
  );
  assert!(turn.share > 0.0 && turn.share < 1.0, "{turn:?}");
  assert!((turn.centre.0 - 0.5).abs() < 1e-9, "{turn:?}");
}

#[test]
fn the_push_follows_the_screen_zoom() {
  // Wider than the recording's screen by more than a tenth: a whole push in.
  let grows = arranged(&green_scene(0.05, 0.9), 2_200)
    .background_turn
    .unwrap();
  assert!((grows.push - 0.06).abs() < 1e-9, "{grows:?}");
  let shrinks = arranged(&green_scene(0.25, 0.5), 2_200)
    .background_turn
    .unwrap();
  assert!((shrinks.push + 0.06).abs() < 1e-9, "{shrinks:?}");
  // The same size moved aside: the backgrounds only bloom.
  let moves = arranged(&green_scene(0.0, 0.8), 2_200)
    .background_turn
    .unwrap();
  assert!(moves.push.abs() < 1e-9, "{moves:?}");
}
