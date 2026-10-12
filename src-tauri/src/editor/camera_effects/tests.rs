// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use super::{image, keep, make, matte, tidy, CameraEffects};
use screenwide_export::camera_effects::CameraBackground;

fn folder(name: &str) -> PathBuf {
  let folder = std::env::temp_dir().join(format!(
    "screenwide-camera-effects-{name}-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  folder
}

#[test]
fn closing_keeps_only_the_files_the_effects_as_they_stand_read() {
  let folder = folder("tidy");
  let project = folder.join("Project.screenwide");
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
  std::fs::write(image::path(&folder), b"picture").unwrap();

  // The background blurred: its matte stays, the picture and the colour's
  // measure go, and so does what earlier makes left.
  keep(
    &folder,
    &CameraEffects {
      background: CameraBackground::Blurred,
      has_image: true,
      enhance: false,
    },
  )
  .unwrap();
  tidy::on_close(&project);
  assert!(matte::path(&folder).is_file());
  assert!(!make::measured_path(&folder).exists());
  assert!(!image::path(&folder).exists());
  assert!(!super::choice(&folder).has_image);
  for name in [
    "camera-background.png",
    "camera-composed-1a2b.mp4",
    "camera-enhanced.mov",
  ] {
    assert!(!folder.join(name).exists(), "{name}");
  }
  assert!(folder.join("camera.mov").is_file(), "the recording stays");

  // The background kept: its matte goes too.
  keep(&folder, &CameraEffects::default()).unwrap();
  tidy::on_close(&project);
  assert!(!matte::path(&folder).exists());

  std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn a_project_kept_with_the_old_colour_switch_on_reads_as_enhanced() {
  let read = |kept: &str| serde_json::from_str::<CameraEffects>(kept).unwrap().enhance;
  assert!(read(
    r#"{"background":"kept","hasImage":false,"colour":true,"enhance":false}"#
  ));
  assert!(read(
    r#"{"background":"kept","hasImage":false,"colour":false,"enhance":true}"#
  ));
  assert!(!read(
    r#"{"background":"kept","hasImage":false,"colour":false}"#
  ));
  assert!(read(
    r#"{"background":"kept","hasImage":false,"enhance":true}"#
  ));
}
