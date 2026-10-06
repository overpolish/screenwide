// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use super::folder::create_in;
use super::*;
use crate::recording::PrimaryRecordingKind;

fn scratch(name: &str) -> PathBuf {
  let directory = std::env::temp_dir()
    .join("screenwide-tests")
    .join(format!("project-{name}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&directory);
  directory
}

fn recording(primary: &str) -> RecordingManifest {
  RecordingManifest {
    duration_ms: Some(4_000),
    has_microphone: true,
    has_system_audio: false,
    media: RecordingMedia {
      primary: primary.to_owned(),
      camera: None,
      cursor: None,
      keyboard: None,
    },
    primary_kind: PrimaryRecordingKind::Screen,
    source_scale_factor: 2.0,
    origin: crate::project::RecordingOrigin::Capture,
  }
}

#[test]
fn a_second_project_with_the_same_title_gets_its_own_folder() {
  let directory = scratch("same-title");

  let first = create_in(&directory, "Screenwide 2026-10-06 at 14.47.12").unwrap();
  let second = create_in(&directory, "Screenwide 2026-10-06 at 14.47.12").unwrap();

  assert_ne!(first.root, second.root);
  assert!(second.media.is_dir());
  // The manifest is named after its own folder, so the copy says which it is.
  assert_eq!(
    second.file.file_name().unwrap(),
    "Screenwide 2026-10-06 at 14.47.12 (2).screenwide"
  );
  assert_eq!(second.file.parent(), Some(second.root.as_path()));
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn media_named_in_a_project_still_resolves_after_the_folder_moves() {
  let directory = scratch("moved");
  let project = create_in(&directory, "Demo").unwrap();
  let movie = project.media.join("recording.mov");
  std::fs::write(&movie, b"movie").unwrap();
  let media = RecordingMedia::relative_to(&project.file, &movie, None, None, None).unwrap();
  write(
    &project.file,
    &Manifest::recording(RecordingManifest {
      media,
      ..recording("")
    }),
  )
  .unwrap();

  let moved = directory.join("Renamed elsewhere");
  std::fs::rename(&project.root, &moved).unwrap();
  let file = moved.join("Demo.screenwide");
  let manifest = read(&file).unwrap();

  let recorded = manifest.recorded().unwrap();
  assert_eq!(recorded.media.primary, "media/recording.mov");
  assert_eq!(
    std::fs::read(resolve(&file, &recorded.media.primary).unwrap()).unwrap(),
    b"movie"
  );
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn refuses_media_names_that_reach_outside_the_project() {
  let file = Path::new("/projects/Demo/Demo.screenwide");
  for name in [
    "../secret.mov",
    "media/../../secret.mov",
    "/etc/passwd",
    "",
    "media//x.mov",
  ] {
    assert!(resolve(file, name).is_err(), "{name} resolved");
  }
  assert_eq!(
    resolve(file, "media/recording.mov").unwrap(),
    Path::new("/projects/Demo/media/recording.mov")
  );
}

#[test]
fn refuses_a_project_from_a_newer_version() {
  let directory = scratch("newer");
  std::fs::create_dir_all(&directory).unwrap();
  let file = directory.join("Future.screenwide");
  std::fs::write(&file, br#"{"version":999,"kind":"hologram"}"#).unwrap();

  assert_eq!(
    read(&file).err().as_deref(),
    Some("This project was made by a newer version of Screenwide")
  );
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn renaming_a_project_renames_its_folder_and_manifest_and_keeps_it_whole() {
  let file = scratch_project("rename");
  let directory = file.parent().unwrap().parent().unwrap().to_path_buf();
  create_in(&directory, "Taken").unwrap();

  assert!(rename(&file, "Taken").is_err());
  assert!(file.is_file());

  let renamed = rename(&file, "Product demo").unwrap();
  assert_eq!(
    renamed,
    directory.join("Product demo/Product demo.screenwide")
  );
  assert_eq!(
    read(&renamed).unwrap().recorded().unwrap().media.primary,
    "media/recording.mov"
  );
  // A change of case only is a rename too, even on a disk that ignores case.
  let recased = rename(&renamed, "product demo").unwrap();
  assert_eq!(recased.file_name().unwrap(), "product demo.screenwide");
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn a_projects_size_counts_everything_in_its_folder() {
  let file = scratch_project("size");
  let media = file.parent().unwrap().join("media");
  std::fs::write(media.join("recording.mov"), vec![0; 1_000]).unwrap();
  std::fs::create_dir_all(media.join("nested")).unwrap();
  std::fs::write(media.join("nested/cursor.jsonl"), vec![0; 24]).unwrap();
  let manifest = std::fs::metadata(&file).unwrap().len();

  assert_eq!(summarize(&file).size_bytes, Some(manifest + 1_024));
  let _ = std::fs::remove_dir_all(file.parent().unwrap().parent().unwrap());
}

#[test]
fn lists_a_folders_projects_newest_first_and_skips_discarded_ones() {
  let directory = scratch("listing");
  let older = create_in(&directory, "Older").unwrap();
  write(
    &older.file,
    &Manifest::recording(recording("media/recording.mov")),
  )
  .unwrap();
  std::thread::sleep(std::time::Duration::from_millis(20));
  let newer = create_in(&directory, "Newer").unwrap();
  write(
    &newer.file,
    &Manifest::recording(recording("media/recording.mov")),
  )
  .unwrap();
  let discarded = create_in(&directory, "Discarded").unwrap();
  discarded.mark_cancelled().unwrap();
  // A folder renamed by hand still lists the manifest it holds.
  std::fs::rename(&older.root, directory.join("Renamed by hand")).unwrap();

  let titles: Vec<_> = in_folder(&directory)
    .into_iter()
    .map(|project| (project.title, project.available, project.duration_ms))
    .collect();
  assert_eq!(
    titles,
    [
      ("Newer".to_owned(), true, Some(4_000)),
      ("Older".to_owned(), true, Some(4_000))
    ]
  );
  let _ = std::fs::remove_dir_all(directory);
}

/// A recording project with a manifest and no media, for tests of what is
/// saved in one. Returns the manifest.
pub(crate) fn scratch_project(name: &str) -> PathBuf {
  let directory = scratch(name);
  let project = create_in(&directory, "Scratch").unwrap();
  write(
    &project.file,
    &Manifest::recording(recording("media/recording.mov")),
  )
  .unwrap();
  project.file
}
