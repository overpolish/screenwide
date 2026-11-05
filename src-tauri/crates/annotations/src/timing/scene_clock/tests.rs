// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn scene(id: &str, [start_ms, end_ms]: [u64; 2], cursor: Option<bool>) -> RecordingSceneClip {
  let mut clip: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": id, "startMs": start_ms, "endMs": end_ms, "preset": "screen-only",
  }))
  .unwrap();
  clip.cursor = cursor;
  clip
}

fn shown(clips: &[RecordingSceneClip], recording: bool, at: u64) -> f32 {
  cursor_in_scenes(clips, recording, (at, Some(30_000)))
}

#[test]
fn a_scene_that_hides_the_cursor_fades_it_out_and_back() {
  let clips = [scene("game", [4_000, 10_000], Some(false))];
  assert_eq!(shown(&clips, true, 2_000), 1.0);
  let leaving = shown(&clips, true, 4_300);
  assert!(leaving > 0.0 && leaving < 1.0, "{leaving}");
  assert_eq!(shown(&clips, true, 7_000), 0.0);
  let returning = shown(&clips, true, 9_700);
  assert!(returning > 0.0 && returning < 1.0, "{returning}");
  assert_eq!(shown(&clips, true, 12_000), 1.0);
}

#[test]
fn a_scene_of_its_own_choice_shows_it_where_the_recording_hides_it() {
  let clips = [
    scene("talk", [2_000, 6_000], Some(true)),
    scene("plain", [6_000, 12_000], None),
  ];
  assert_eq!(shown(&clips, false, 1_000), 0.0);
  // Handed straight on, it fades out as the next scene arrives rather than
  // before the first one ends.
  assert_eq!(shown(&clips, false, 5_900), 1.0);
  let handing = shown(&clips, false, 6_300);
  assert!(handing > 0.0 && handing < 1.0, "{handing}");
  // A scene with no choice of its own does as the recording does.
  assert_eq!(shown(&clips, false, 9_000), 0.0);
  assert!(cursor_shown_anywhere(&clips, false));
  assert!(!cursor_shown_anywhere(&clips[1..], false));
}
