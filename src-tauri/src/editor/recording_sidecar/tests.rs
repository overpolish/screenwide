// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn directory(name: &str) -> PathBuf {
  let directory = std::env::temp_dir().join("screenwide-tests").join(name);
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

fn valid_cursor(path: &Path) {
  let header = crate::recording::cursor::CursorRecord::Header {
    coordinate_space: "global-logical-points".to_owned(),
    platform: "test".to_owned(),
    source: crate::recording::cursor::CursorSource {
      height: 100.0,
      kind: crate::recording::cursor::CursorSourceKind::Screen,
      platform_id: "1".to_owned(),
      video_height: 200,
      video_width: 200,
      width: 100.0,
      x: 0.0,
      y: 0.0,
    },
    timebase: "recording-microseconds".to_owned(),
    version: crate::recording::cursor::FORMAT_VERSION,
  };
  std::fs::write(
    path,
    format!("{}\n", serde_json::to_string(&header).unwrap()),
  )
  .unwrap();
}

fn valid_keyboard(path: &Path) {
  let header = crate::recording::keyboard::KeyboardRecord::Header {
    platform: "test".to_owned(),
    timebase: "recording-microseconds".to_owned(),
    version: crate::recording::keyboard::FORMAT_VERSION,
  };
  std::fs::write(
    path,
    format!("{}\n", serde_json::to_string(&header).unwrap()),
  )
  .unwrap();
}

#[test]
fn pairs_only_valid_sidecars_with_their_recording() {
  let directory = directory("recording-sidecar-pair");
  let recording = directory.join("recording-20260809-060151.000.mov");
  let cursor = directory.join("recording-20260809-060151.000.cursor.jsonl");
  let keyboard = directory.join("recording-20260809-060151.000.keyboard.jsonl");
  std::fs::write(&recording, b"screen").unwrap();
  valid_cursor(&cursor);
  valid_keyboard(&keyboard);

  assert_eq!(
    cursor_for_recording(&recording).as_deref(),
    Some(cursor.as_path())
  );
  assert_eq!(
    keyboard_for_recording(&recording).as_deref(),
    Some(keyboard.as_path())
  );

  std::fs::write(&cursor, b"invalid").unwrap();
  std::fs::write(&keyboard, b"invalid").unwrap();
  assert_eq!(cursor_for_recording(&recording), None);
  assert_eq!(keyboard_for_recording(&recording), None);
  std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn sweeps_only_unclaimed_sidecars_of_each_kind() {
  let directory = directory("recording-sidecar-sweep");
  let kept_cursor = directory.join("recording-kept.cursor.jsonl");
  let abandoned_cursor = directory.join("recording-abandoned.cursor.jsonl");
  let kept_keyboard = directory.join("recording-kept.keyboard.jsonl");
  let abandoned_keyboard = directory.join("recording-abandoned.keyboard.jsonl");
  let unrelated = directory.join("notes.jsonl");
  for path in [
    &kept_cursor,
    &abandoned_cursor,
    &kept_keyboard,
    &abandoned_keyboard,
    &unrelated,
  ] {
    std::fs::write(path, b"data").unwrap();
  }

  sweep_unclaimed_cursors(&directory, Some(&kept_cursor));
  sweep_unclaimed_keyboards(&directory, Some(&kept_keyboard));

  assert!(kept_cursor.exists());
  assert!(!abandoned_cursor.exists());
  assert!(kept_keyboard.exists());
  assert!(!abandoned_keyboard.exists());
  assert!(unrelated.exists());
  std::fs::remove_dir_all(directory).unwrap();
}
