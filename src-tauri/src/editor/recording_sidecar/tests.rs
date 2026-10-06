// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;

use super::*;

fn directory(name: &str) -> PathBuf {
  let directory = std::env::temp_dir().join("screenwide-tests").join(name);
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

fn write_cursor(path: &Path) {
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

fn write_keyboard(path: &Path) {
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
fn opens_a_project_with_only_the_tracks_it_can_read() {
  let directory = directory("recording-sidecar-valid");
  let cursor = directory.join("cursor.jsonl");
  let keyboard = directory.join("keyboard.jsonl");
  write_cursor(&cursor);
  write_keyboard(&keyboard);

  assert_eq!(valid_cursor(cursor.clone()), Some(cursor.clone()));
  assert_eq!(valid_keyboard(keyboard.clone()), Some(keyboard.clone()));

  // What a crash mid-write leaves behind.
  std::fs::write(&cursor, b"invalid").unwrap();
  std::fs::write(&keyboard, b"invalid").unwrap();
  assert_eq!(valid_cursor(cursor), None);
  assert_eq!(valid_keyboard(keyboard), None);
  std::fs::remove_dir_all(directory).unwrap();
}
