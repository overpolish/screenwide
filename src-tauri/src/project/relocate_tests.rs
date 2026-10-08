// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use super::folder::create_in;
use super::*;

fn location(name: &str) -> PathBuf {
  let directory = std::env::temp_dir()
    .join("screenwide-tests")
    .join(format!("location-{name}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

fn cleanup(file: &std::path::Path) {
  let _ = std::fs::remove_dir_all(file.parent().unwrap().parent().unwrap());
}

#[test]
fn a_move_onto_a_taken_name_takes_the_next_free_one_and_its_file_follows() {
  let file = scratch_project("move");
  let target = location("move");
  create_in(&target, "Scratch").unwrap();
  let size = folder_size(file.parent().unwrap());
  let mut heard = 0;

  let moved = move_into(&file, &target, size, &mut |bytes| heard += bytes).unwrap();

  assert_eq!(moved, target.join("Scratch (2)/Scratch (2).screenwide"));
  assert!(!file.parent().unwrap().exists());
  assert_eq!(
    read(&moved).unwrap().recorded().unwrap().media.primary,
    "media/recording.mov"
  );
  // A rename is reported whole, so a bar over several projects still fills.
  assert_eq!(heard, size);
  cleanup(&file);
  let _ = std::fs::remove_dir_all(target);
}

#[test]
fn a_move_to_where_the_project_already_is_leaves_it_alone() {
  let file = scratch_project("move-here");
  let here = file.parent().unwrap().parent().unwrap().to_path_buf();

  let moved = move_into(&file, &here, 0, &mut |_| {}).unwrap();

  assert_eq!(moved, file);
  assert!(file.is_file());
  cleanup(&file);
}

#[test]
fn a_duplicate_holds_everything_keeps_edit_times_and_drops_a_name_given_while_open() {
  let file = scratch_project("duplicate");
  let media = file.parent().unwrap().join("media");
  std::fs::create_dir_all(media.join("nested")).unwrap();
  std::fs::write(media.join("recording.mov"), vec![7; 5_000]).unwrap();
  std::fs::write(media.join("nested/cursor.jsonl"), b"cursor").unwrap();
  update(&file, |manifest| {
    manifest.title = Some("Renamed while open".to_owned());
    true
  })
  .unwrap();
  let edited = std::fs::metadata(media.join("recording.mov"))
    .unwrap()
    .modified()
    .unwrap();
  let mut heard = 0;

  let copy = duplicate(&file, "Scratch", &mut |bytes| heard += bytes).unwrap();

  let name = if cfg!(target_os = "windows") {
    "Scratch - Copy"
  } else {
    "Scratch copy"
  };
  assert_eq!(
    copy,
    file
      .parent()
      .unwrap()
      .parent()
      .unwrap()
      .join(format!("{name}/{name}.screenwide"))
  );
  let copied_media = copy.parent().unwrap().join("media");
  assert_eq!(
    std::fs::read(copied_media.join("recording.mov")).unwrap(),
    vec![7; 5_000]
  );
  assert_eq!(
    std::fs::read(copied_media.join("nested/cursor.jsonl")).unwrap(),
    b"cursor"
  );
  assert_eq!(
    std::fs::metadata(copied_media.join("recording.mov"))
      .unwrap()
      .modified()
      .unwrap(),
    edited
  );
  assert_eq!(read(&copy).unwrap().title, None);
  assert_eq!(
    read(&file).unwrap().title.as_deref(),
    Some("Renamed while open")
  );
  assert_eq!(heard, folder_size(file.parent().unwrap()));
  cleanup(&file);
}
