// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{choice, image, keep, make, matte, project_folder, tidy, Choice};
use crate::editor::timeline_edit::{persist, untouched};
use screenwide_timeline_edit::{CameraBackgroundRange, CameraRangeBackground};

fn range(id: &str, background: CameraRangeBackground) -> CameraBackgroundRange {
  CameraBackgroundRange {
    id: id.to_owned(),
    start_ms: 0,
    end_ms: 1_000,
    background,
  }
}

#[test]
fn closing_keeps_only_the_files_the_effects_and_ranges_as_they_stand_read() {
  let project = screenwide_project::scratch::scratch_project("camera-effects-tidy");
  let folder = project_folder(&project);
  let touch = |name: &str| std::fs::write(folder.join(name), b"made").unwrap();
  for name in [
    "camera-background.png",
    "camera-composed-1a2b.mp4",
    "camera-enhanced.mov",
    "camera.mov",
  ] {
    touch(name);
  }
  std::fs::write(matte::path(&folder), b"matte").unwrap();
  std::fs::write(make::measured_path(&folder), b"{}").unwrap();
  for id in ["shown", "blurred-now", "gone"] {
    std::fs::write(image::path(&folder, id), b"picture").unwrap();
  }

  // One range shows its picture, another blurs the room, and a third range
  // that had a picture is gone: the matte and the shown picture stay, the
  // others go, as do the colour's measure and what earlier makes left.
  let mut edit = untouched(7);
  edit.camera_background_ranges = vec![
    range("shown", CameraRangeBackground::Image),
    CameraBackgroundRange {
      start_ms: 1_000,
      end_ms: 2_000,
      ..range("blurred-now", CameraRangeBackground::Blurred)
    },
  ];
  persist(&project, 7, 1, edit).unwrap();
  keep(&folder, &Choice { enhance: false }).unwrap();
  tidy::on_close(&project);
  assert!(matte::path(&folder).is_file());
  assert!(!make::measured_path(&folder).exists());
  assert_eq!(image::chosen(&folder), ["shown"]);
  for name in [
    "camera-background.png",
    "camera-composed-1a2b.mp4",
    "camera-enhanced.mov",
  ] {
    assert!(!folder.join(name).exists(), "{name}");
  }
  assert!(folder.join("camera.mov").is_file(), "the recording stays");

  // No ranges left: the matte goes too.
  persist(&project, 7, 2, untouched(7)).unwrap();
  tidy::on_close(&project);
  assert!(!matte::path(&folder).exists());
  assert!(image::chosen(&folder).is_empty());
}

#[test]
fn a_background_kept_for_the_whole_camera_becomes_one_range_with_its_picture() {
  let project = screenwide_project::scratch::scratch_project("camera-effects-carry");
  let folder = project_folder(&project);
  std::fs::write(
    folder.join("camera-effects.json"),
    r#"{"version":1,"choice":{"background":"image","hasImage":true,"enhance":true}}"#,
  )
  .unwrap();
  std::fs::write(image::legacy_path(&folder), b"picture").unwrap();
  let camera = super::CameraRecording {
    path: folder.join("camera.mov"),
    duration_ms: 8_000,
    recording_ms: 8_000,
    project: project.clone(),
  };

  super::carry::carry(&camera, 7).unwrap();
  let ranges = crate::editor::timeline_edit::saved_camera_background_ranges(&project);
  assert_eq!(ranges.len(), 1);
  assert_eq!((ranges[0].start_ms, ranges[0].end_ms), (0, 8_000));
  assert_eq!(ranges[0].background, CameraRangeBackground::Image);
  assert_eq!(image::chosen(&folder), [ranges[0].id.clone()]);
  assert!(!image::legacy_path(&folder).exists());
  assert!(choice(&folder).enhance, "Enhance stays as it was");

  // Carried once: the range removed since does not come back.
  persist(&project, 7, 5, untouched(7)).unwrap();
  super::carry::carry(&camera, 7).unwrap();
  assert!(crate::editor::timeline_edit::saved_camera_background_ranges(&project).is_empty());
}

#[test]
fn a_project_kept_with_the_old_colour_switch_on_reads_as_enhanced() {
  let folder = std::env::temp_dir().join(format!(
    "screenwide-camera-effects-colour-{}",
    std::process::id()
  ));
  std::fs::create_dir_all(&folder).unwrap();
  let read = |kept: &str| {
    let file = format!(r#"{{"version":1,"choice":{kept}}}"#);
    std::fs::write(folder.join("camera-effects.json"), file).unwrap();
    choice(&folder).enhance
  };
  assert!(read(
    r#"{"background":"kept","hasImage":false,"colour":true,"enhance":false}"#
  ));
  assert!(read(r#"{"background":"blurred","enhance":true}"#));
  assert!(!read(
    r#"{"background":"kept","hasImage":false,"colour":false}"#
  ));
  assert!(read(r#"{"enhance":true}"#));
  std::fs::remove_dir_all(folder).unwrap();
}
