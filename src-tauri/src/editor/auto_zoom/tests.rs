// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{clips, Signals, VisibleArea};
use crate::editor::scenes::{validate_clips, RecordingSceneClip, RecordingScenePreset};
use crate::recording::cursor::{
  ButtonState, CursorButton, CursorRecord, CursorSource, CursorSourceKind,
};
use crate::recording::keyboard::KeyboardRecord;

const DURATION_MS: u64 = 120_000;

fn header() -> CursorRecord {
  CursorRecord::Header {
    coordinate_space: "global-points".to_owned(),
    platform: "test".to_owned(),
    source: CursorSource {
      height: 1_000.0,
      kind: CursorSourceKind::Screen,
      platform_id: String::new(),
      video_height: 1_000,
      video_width: 1_000,
      width: 1_000.0,
      x: 0.0,
      y: 0.0,
    },
    timebase: "recording-microseconds".to_owned(),
    version: 1,
  }
}

fn button(ms: u64, (x, y): (f64, f64), state: ButtonState) -> CursorRecord {
  CursorRecord::Button {
    button: CursorButton::Left,
    click_count: 1,
    state,
    timestamp_us: ms * 1_000,
    x,
    y,
  }
}

/// A click at `ms`, in points on a 1000 point square screen.
fn click(ms: u64, at: (f64, f64)) -> [CursorRecord; 2] {
  [
    button(ms, at, ButtonState::Down),
    button(ms + 80, at, ButtonState::Up),
  ]
}

fn key(ms: u64) -> KeyboardRecord {
  KeyboardRecord::KeyDown {
    key_code: 51,
    modifiers: Vec::new(),
    timestamp_us: ms * 1_000,
  }
}

fn planned(
  clicks: &[(u64, (f64, f64))],
  keys: &[u64],
  visible: VisibleArea,
) -> Vec<RecordingSceneClip> {
  planned_across(clicks, keys, &[], visible)
}

/// What clicks and keys make with another app coming to the front at each
/// of `switches`.
fn planned_across(
  clicks: &[(u64, (f64, f64))],
  keys: &[u64],
  switches: &[u64],
  visible: VisibleArea,
) -> Vec<RecordingSceneClip> {
  let mut cursor = vec![header()];
  cursor.extend(clicks.iter().flat_map(|&(ms, at)| click(ms, at)));
  cursor.extend(switches.iter().map(|&ms| CursorRecord::AppSwitch {
    timestamp_us: ms * 1_000,
  }));
  let keyboard: Vec<_> = keys.iter().map(|&ms| key(ms)).collect();
  clips(&Signals::read(&cursor, &keyboard, visible), DURATION_MS)
}

/// The zoom on screen `at`, if any.
fn zoom_at(clips: &[RecordingSceneClip], at: u64) -> Option<&RecordingSceneClip> {
  clips
    .iter()
    .find(|clip| clip.start_ms <= at && at < clip.end_ms)
}

fn zoom_of(clip: &RecordingSceneClip) -> f64 {
  clip.screen.map_or(1.0, |framing| framing.zoom)
}

/// Whatever the clicks, the zooms are scenes that can play, made as auto
/// zooms, and the whole screen never shows only for a moment between two.
fn assert_playable(clips: &[RecordingSceneClip]) {
  assert!(validate_clips(clips).is_ok(), "{clips:#?}");
  for clip in clips {
    assert!(clip.auto && clip.preset == RecordingScenePreset::Full);
    assert!(zoom_of(clip) > 1.0);
    assert!(clip.end_ms <= DURATION_MS);
  }
  for pair in clips.windows(2) {
    let gap = pair[1].start_ms - pair[0].end_ms;
    assert!(gap == 0 || gap >= 2_000, "a {gap} ms zoom out pumps");
  }
}

#[test]
fn a_click_is_zoomed_into_before_it_lands_and_held_after() {
  let clips = planned(&[(10_000, (300.0, 400.0))], &[], VisibleArea::WHOLE);
  assert_playable(&clips);
  let [clip] = clips.as_slice() else {
    panic!("one click makes one zoom: {clips:#?}");
  };
  // Arrived before the click; still there well after it.
  assert!(clip.start_ms + 600 <= 10_000);
  assert!(clip.end_ms >= 10_000 + 2_000);
  let framing = clip.screen.unwrap();
  let half = 0.5 / framing.zoom;
  assert!((framing.focus_x - half..=framing.focus_x + half).contains(&0.3));
  assert!((framing.focus_y - half..=framing.focus_y + half).contains(&0.4));
}

#[test]
fn clicks_close_together_share_one_zoom() {
  let clicks = [
    (10_000, (400.0, 300.0)),
    (11_000, (440.0, 320.0)),
    (12_500, (420.0, 350.0)),
    (14_000, (450.0, 310.0)),
  ];
  let clips = planned(&clicks, &[], VisibleArea::WHOLE);
  assert_playable(&clips);
  assert_eq!(clips.len(), 1, "{clips:#?}");
  assert!(clips[0].start_ms < 10_000 && clips[0].end_ms > 14_000);
}

#[test]
fn clicks_far_apart_in_time_zoom_out_between() {
  let clicks = [(10_000, (200.0, 200.0)), (40_000, (800.0, 700.0))];
  let clips = planned(&clicks, &[], VisibleArea::WHOLE);
  assert_playable(&clips);
  assert_eq!(clips.len(), 2, "{clips:#?}");
  assert!(clips[1].start_ms - clips[0].end_ms >= 10_000);
}

#[test]
fn clicks_flitting_across_the_screen_are_left_on_the_whole_screen() {
  let corners = [(80.0, 80.0), (920.0, 920.0), (920.0, 80.0), (80.0, 920.0)];
  let clicks: Vec<_> = (0..24)
    .map(|index| (10_000 + index * 400, corners[index as usize % 4]))
    .collect();
  let clips = planned(&clicks, &[], VisibleArea::WHOLE);
  assert_playable(&clips);
  assert!(clips.is_empty(), "{clips:#?}");
}

#[test]
fn typing_after_a_click_holds_the_zoom_on() {
  let keys: Vec<_> = (0..30).map(|index| 10_500 + index * 300).collect();
  let clips = planned(&[(10_000, (600.0, 500.0))], &keys, VisibleArea::WHOLE);
  assert_playable(&clips);
  let [clip] = clips.as_slice() else {
    panic!("one field typed into makes one zoom: {clips:#?}");
  };
  assert!(clip.end_ms >= keys[keys.len() - 1] + 900);
}

#[test]
fn a_click_outside_the_visible_picture_is_not_zoomed_into() {
  let right_half = VisibleArea {
    x: 0.5,
    y: 0.0,
    width: 0.5,
    height: 1.0,
  };
  assert!(planned(&[(10_000, (200.0, 500.0))], &[], right_half).is_empty());
  let clips = planned(&[(10_000, (750.0, 500.0))], &[], right_half);
  let [clip] = clips.as_slice() else {
    panic!("{clips:#?}");
  };
  // The middle of the right half is the middle of what is shown.
  assert!((clip.screen.unwrap().focus_x - 0.5).abs() < 1e-9);
}

#[test]
fn a_drag_across_most_of_the_screen_is_not_zoomed_into() {
  let cursor = vec![
    header(),
    button(10_000, (50.0, 500.0), ButtonState::Down),
    button(11_000, (950.0, 520.0), ButtonState::Up),
  ];
  let signals = Signals::read(&cursor, &[], VisibleArea::WHOLE);
  assert!(clips(&signals, DURATION_MS).is_empty());
}

#[test]
fn a_sweep_across_the_screen_between_two_clicks_never_flashes_the_whole_screen() {
  let cursor = [
    vec![header()],
    click(10_000, (300.0, 300.0)).to_vec(),
    vec![
      button(13_300, (50.0, 500.0), ButtonState::Down),
      button(13_500, (950.0, 520.0), ButtonState::Up),
    ],
    click(14_200, (320.0, 310.0)).to_vec(),
  ]
  .concat();
  let signals = Signals::read(&cursor, &[], VisibleArea::WHOLE);
  assert_playable(&clips(&signals, DURATION_MS));
}

#[test]
fn typing_in_one_place_then_clicking_another_pans_between_them() {
  let first: Vec<_> = (0..10).map(|index| 10_300 + index * 300).collect();
  let clicked_ms = first[first.len() - 1] + 1_000;
  let second: Vec<_> = (0..10)
    .map(|index| clicked_ms + 300 + index * 300)
    .collect();
  let clips = planned(
    &[(10_000, (150.0, 150.0)), (clicked_ms, (800.0, 750.0))],
    &[first, second].concat(),
    VisibleArea::WHOLE,
  );
  assert_playable(&clips);
  let [typed, clicked] = clips.as_slice() else {
    panic!("each field typed into is zoomed into: {clips:#?}");
  };
  assert_eq!(typed.end_ms, clicked.start_ms);
  assert!(clicked.start_ms + 600 <= clicked_ms);
}

/// A long, uneven session: bursts of clicks all over the screen, some
/// followed by typing, with lulls between.
#[test]
fn any_session_makes_zooms_that_play_without_pumping() {
  let mut seed = 0x2545_f491_u64;
  let mut next = |range: u64| {
    seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    (seed >> 33) % range
  };
  let mut ms = 500;
  let mut clicks = Vec::new();
  let mut keys = Vec::new();
  while ms < DURATION_MS - 10_000 {
    let at = (next(1_000) as f64, next(1_000) as f64);
    for _ in 0..next(5) + 1 {
      let jitter = (next(80) as f64 - 40.0, next(80) as f64 - 40.0);
      clicks.push((ms, (at.0 + jitter.0, at.1 + jitter.1)));
      ms += 200 + next(2_500);
    }
    if next(3) == 0 {
      for _ in 0..next(20) {
        keys.push(ms);
        ms += 120 + next(400);
      }
    }
    ms += next(9_000);
  }
  let clips = planned(&clicks, &keys, VisibleArea::WHOLE);
  assert_playable(&clips);
  assert!(!clips.is_empty());
}

/// The session that showed it: clicks in a browser on the left, then a
/// click into a terminal on the right that brings it to the front, work in
/// the terminal, and a click on its tab bar.
#[test]
fn switching_apps_zooms_out_to_show_the_switch() {
  let switched_ms = 12_760;
  let clicks = [
    (1_755, (251.0, 325.0)),
    (6_220, (108.0, 418.0)),
    (7_575, (102.0, 496.0)),
    (9_219, (216.0, 316.0)),
    (11_329, (435.0, 361.0)),
    (12_729, (809.0, 301.0)),
    (15_055, (693.0, 695.0)),
    (16_203, (504.0, 677.0)),
    (18_734, (573.0, 99.0)),
  ];
  let clips = planned_across(&clicks, &[], &[switched_ms], VisibleArea::WHOLE);
  assert_playable(&clips);
  assert!(zoom_at(&clips, switched_ms).is_none(), "{clips:#?}");
  let before = clips
    .iter()
    .rfind(|clip| clip.end_ms <= switched_ms)
    .expect("the browser work is zoomed into");
  let after = clips
    .iter()
    .find(|clip| clip.start_ms >= switched_ms)
    .expect("the terminal work is zoomed into");
  assert!(after.start_ms - before.end_ms >= 2_000);
  assert!(zoom_at(&clips, 6_220).is_some());
  assert!(zoom_at(&clips, 15_100).is_some());
}

#[test]
fn typing_straight_after_switching_apps_is_zoomed_into_once_the_switch_shows() {
  let switched_ms = 10_050;
  let keys: Vec<_> = (0..20).map(|index| 10_300 + index * 300).collect();
  let clips = planned_across(
    &[(5_000, (200.0, 200.0)), (10_000, (800.0, 700.0))],
    &keys,
    &[switched_ms],
    VisibleArea::WHOLE,
  );
  assert_playable(&clips);
  let [first, typed] = clips.as_slice() else {
    panic!("both apps' work is zoomed into: {clips:#?}");
  };
  assert!(first.end_ms <= switched_ms);
  assert!(typed.start_ms - first.end_ms >= 2_000);
  assert!(typed.end_ms >= keys[keys.len() - 1]);
}

#[test]
fn no_zoom_carries_across_a_switch_even_to_the_same_place() {
  let clicks = [(10_000, (300.0, 300.0)), (12_000, (310.0, 305.0))];
  let clips = planned_across(&clicks, &[], &[11_000], VisibleArea::WHOLE);
  assert_playable(&clips);
  assert!(zoom_at(&clips, 11_000).is_none(), "{clips:#?}");
}

/// The recording that showed it: a click on a window's top corner brings it
/// forward, and seconds later text is selected in the middle of it. The zoom
/// is about the selection, not the corner the window was caught by.
#[test]
fn the_press_that_switched_apps_does_not_frame_the_work_after_it() {
  let switched_ms = 17_946;
  let cursor = [
    vec![header()],
    click(17_937, (900.0, 144.0)).to_vec(),
    vec![
      button(21_283, (516.0, 553.0), ButtonState::Down),
      button(22_095, (539.0, 550.0), ButtonState::Up),
      CursorRecord::AppSwitch {
        timestamp_us: switched_ms * 1_000,
      },
    ],
  ]
  .concat();
  let signals = Signals::read(&cursor, &[], VisibleArea::WHOLE);
  let clips = clips(&signals, DURATION_MS);
  assert_playable(&clips);
  let selected = zoom_at(&clips, 21_500).expect("the selection is zoomed into");
  let framing = selected.screen.unwrap();
  assert!((framing.focus_x - 0.53).abs() < 0.05, "{framing:?}");
  assert!((framing.focus_y - 0.55).abs() < 0.05, "{framing:?}");
}
