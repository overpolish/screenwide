// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::deleted::{put_back, set_aside, DELETED_FOLDER};
use super::folder::create_in;
use super::*;

fn project_in(directory: &std::path::Path, title: &str) -> std::path::PathBuf {
  let project = create_in(directory, title).unwrap();
  write(
    &project.file,
    &Manifest::recording(RecordingManifest {
      duration_ms: Some(1_000),
      has_microphone: false,
      has_system_audio: false,
      media: RecordingMedia {
        primary: "media/recording.mov".to_owned(),
        camera: None,
        cursor: None,
        keyboard: None,
        moments: None,
      },
      primary_kind: crate::recording::PrimaryRecordingKind::Screen,
      source_scale_factor: 2.0,
      origin: RecordingOrigin::Capture,
    }),
  )
  .unwrap();
  project.file
}

#[test]
fn deleted_projects_wait_beside_their_folder_under_their_own_names() {
  let file = scratch_project("delete-aside");
  let directory = file.parent().unwrap().parent().unwrap().to_path_buf();
  let deleted = directory.join(DELETED_FOLDER);

  let first = set_aside(&file).unwrap();
  let second = set_aside(&project_in(&directory, "Scratch")).unwrap();

  assert_eq!(first, deleted.join("Scratch/Scratch.screenwide"));
  // A second of the same name takes the next folder but keeps its file's
  // name, so it is listed as the project it was.
  assert_eq!(second, deleted.join("Scratch (2)/Scratch.screenwide"));
  assert!(!directory.join("Scratch").exists());
  // Listing the folder shows neither, nor the deleted folder.
  assert!(in_folder(&directory).is_empty());
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn a_restored_project_returns_to_its_name_or_the_next_free_one() {
  let file = scratch_project("delete-restore");
  let directory = file.parent().unwrap().parent().unwrap().to_path_buf();
  let aside = set_aside(&file).unwrap();
  // Another project took the name meanwhile.
  project_in(&directory, "Scratch");

  let restored = put_back(&aside, &file).unwrap();

  assert_eq!(
    restored,
    directory.join("Scratch (2)/Scratch (2).screenwide")
  );
  assert_eq!(
    read(&restored).unwrap().recorded().unwrap().media.primary,
    "media/recording.mov"
  );
  assert!(!aside.exists());
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn a_project_comes_back_even_when_its_folder_has_gone() {
  let file = scratch_project("delete-folder-gone");
  let directory = file.parent().unwrap().parent().unwrap().to_path_buf();
  let aside = set_aside(&file).unwrap();
  // Restoring to a folder that is no longer there makes it again.
  let elsewhere = directory.join("Removed/Scratch/Scratch.screenwide");

  let restored = put_back(&aside, &elsewhere).unwrap();

  assert_eq!(restored, elsewhere);
  assert!(restored.is_file());
  let _ = std::fs::remove_dir_all(directory);
}
