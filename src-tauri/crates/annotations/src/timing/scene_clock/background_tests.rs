// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const VIDEO_MS: Option<u64> = Some(30_000);

fn scene([start_ms, end_ms]: [u64; 2], own: bool) -> RecordingSceneClip {
  let mut clip: RecordingSceneClip = serde_json::from_value(serde_json::json!({
    "id": format!("{start_ms}"), "startMs": start_ms, "endMs": end_ms, "preset": "screen-only",
  }))
  .unwrap();
  clip.background = own.then(|| screenwide_background::Background::Solid {
    color: format!("#{start_ms:06}"),
  });
  clip
}

fn blend(clips: &[RecordingSceneClip], at: u64) -> BackgroundBlend {
  let own = |place: Option<usize>| place.and_then(|index| clips[index].background.as_ref());
  background_blend(clips, (at, VIDEO_MS), |from, to| own(from) != own(to))
}

fn settled(place: Option<usize>) -> BackgroundBlend {
  BackgroundBlend::settled(place)
}

fn between(blend: BackgroundBlend, from: Option<usize>, to: Option<usize>) {
  assert_eq!((blend.from, blend.to), (from, to), "{blend:?}");
  assert!(blend.share > 0.0 && blend.share < 1.0, "{blend:?}");
}

#[test]
fn a_scene_of_its_own_blooms_in_ahead_of_its_panes_and_out_again() {
  let clips = [scene([4_000, 10_000], true)];
  assert_eq!(blend(&clips, 3_800), settled(None));
  // A hundred milliseconds before the clip, as the panes are about to move.
  between(blend(&clips, 3_950), None, Some(0));
  between(blend(&clips, 4_600), None, Some(0));
  assert_eq!(blend(&clips, 4_700), settled(Some(0)));
  // The panes leave over the clip's last 600 ms; the background a hundred
  // before them, running on past the clip's end.
  assert_eq!(blend(&clips, 9_250), settled(Some(0)));
  between(blend(&clips, 9_350), Some(0), None);
  between(blend(&clips, 10_050), Some(0), None);
  assert_eq!(blend(&clips, 10_100), settled(None));
}

#[test]
fn a_scene_showing_the_recording_background_changes_nothing() {
  let clips = [scene([4_000, 10_000], false)];
  for at in [3_950, 4_300, 9_700, 10_050] {
    assert_eq!(blend(&clips, at).share, 1.0, "{at}");
  }
}

#[test]
fn butting_scenes_hand_their_backgrounds_straight_on() {
  let clips = [scene([2_000, 6_000], true), scene([6_000, 12_000], true)];
  assert_eq!(blend(&clips, 5_800), settled(Some(0)));
  between(blend(&clips, 6_000), Some(0), Some(1));
  assert_eq!(blend(&clips, 6_700), settled(Some(1)));
}

#[test]
fn a_change_cut_short_by_the_next_finishes_as_it_begins() {
  // Its panes leave over its last 250 ms, so its background starts to
  // leave 250 ms after it started to arrive.
  let clips = [scene([4_000, 4_500], true)];
  let arrived = blend(&clips, 4_149);
  assert_eq!((arrived.from, arrived.to), (None, Some(0)));
  assert!(arrived.share > 0.99, "{arrived:?}");
  between(blend(&clips, 4_200), Some(0), None);
  assert_eq!(blend(&clips, 4_950), settled(None));
}

#[test]
fn a_scene_starting_the_video_holds_its_background_from_the_first_frame() {
  let clips = [scene([0, 5_000], true)];
  assert_eq!(blend(&clips, 0), settled(Some(0)));
  assert_eq!(blend(&clips, 3_000), settled(Some(0)));
}
