// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use super::recorder::MomentRecorder;
use super::settings::{validate, MomentKind, MomentSettings};

fn kind(id: &str, shortcut: Option<&str>) -> MomentKind {
  MomentKind {
    color: "#ffcc00".to_owned(),
    custom_color: None,
    id: id.to_owned(),
    name: id.to_owned(),
    shortcut: shortcut.map(str::to_owned),
  }
}

fn scratch(name: &str) -> PathBuf {
  std::env::temp_dir().join(format!(
    "screenwide-moments-{name}-{}.jsonl",
    std::process::id()
  ))
}

fn recorder(name: &str) -> (MomentRecorder, Instant, PathBuf) {
  let origin = Instant::now();
  let clock = Arc::new(OnceLock::new());
  clock.set(origin).unwrap();
  let path = scratch(name);
  (MomentRecorder::detached(path.clone(), clock), origin, path)
}

fn placed_ms(path: &Path) -> Vec<u64> {
  super::read(path)
    .unwrap()
    .iter()
    .map(|moment| moment.timestamp_us / 1_000)
    .collect()
}

#[test]
fn a_moment_lands_two_seconds_before_its_press_and_never_before_the_start() {
  let (recorder, origin, path) = recorder("look-back");
  let funny = kind("funny", None);
  assert!(recorder.record_for_test(&funny, origin + Duration::from_millis(500)));
  assert!(recorder.record_for_test(&funny, origin + Duration::from_secs(10)));
  let kept = recorder.finish_for_test().expect("moments were placed");
  assert_eq!(placed_ms(&kept), vec![0, 8_000]);
  let _ = std::fs::remove_file(path);
}

#[test]
fn a_press_while_paused_places_nothing_and_the_pause_is_left_out() {
  let (recorder, origin, path) = recorder("paused");
  let funny = kind("funny", None);
  recorder.pause(origin + Duration::from_secs(5));
  assert!(!recorder.record_for_test(&funny, origin + Duration::from_secs(8)));
  recorder.resume(origin + Duration::from_secs(15));
  assert!(recorder.record_for_test(&funny, origin + Duration::from_secs(20)));
  let kept = recorder.finish_for_test().expect("one moment was placed");
  assert_eq!(placed_ms(&kept), vec![8_000]);
  let _ = std::fs::remove_file(path);
}

#[test]
fn a_press_before_the_first_frame_places_nothing() {
  let path = scratch("before-start");
  let recorder = MomentRecorder::detached(path.clone(), Arc::new(OnceLock::new()));
  assert!(!recorder.record_for_test(&kind("funny", None), Instant::now()));
  assert_eq!(recorder.finish_for_test(), None);
  assert!(!path.exists());
}

#[test]
fn a_recording_without_moments_leaves_no_file() {
  let (recorder, _, path) = recorder("empty");
  assert_eq!(recorder.finish_for_test(), None);
  assert!(!path.exists());
}

#[test]
fn moments_before_a_line_cut_short_by_a_crash_are_kept() {
  let (recorder, origin, path) = recorder("truncated");
  assert!(recorder.record_for_test(&kind("notable", None), origin + Duration::from_secs(4)));
  let kept = recorder.finish_for_test().unwrap();
  let mut file = std::fs::OpenOptions::new()
    .append(true)
    .open(&kept)
    .unwrap();
  file.write_all(b"{\"type\":\"moment\",\"col").unwrap();
  let moments = super::read(&kept).unwrap();
  let _ = std::fs::remove_file(path);
  assert_eq!(moments.len(), 1);
  assert_eq!(moments[0].kind_id, "notable");
  assert_eq!(moments[0].timestamp_us, 2_000_000);
}

#[test]
fn kinds_may_not_share_a_shortcut() {
  let settings = MomentSettings {
    kinds: vec![
      kind("funny", Some("CommandOrControl+Shift+Digit1")),
      kind("notable", Some("CommandOrControl+Shift+Digit1")),
    ],
  };
  assert!(validate(settings).is_err());
}

#[test]
fn a_kind_needs_a_name_and_an_id_of_its_own() {
  let mut unnamed = kind("funny", None);
  unnamed.name = "   ".to_owned();
  assert!(validate(MomentSettings {
    kinds: vec![unnamed]
  })
  .is_err());
  assert!(validate(MomentSettings {
    kinds: vec![kind("funny", None), kind("funny", None)]
  })
  .is_err());
}

#[test]
fn a_blank_shortcut_is_kept_as_none() {
  let settings = validate(MomentSettings {
    kinds: vec![kind("funny", Some(" "))],
  })
  .unwrap();
  assert_eq!(settings.kinds[0].shortcut, None);
}

#[test]
fn the_default_kinds_are_valid() {
  assert!(validate(MomentSettings::default()).is_ok());
}
