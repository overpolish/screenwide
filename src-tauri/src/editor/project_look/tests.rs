// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::project::{Manifest, RecordingManifest, RecordingMedia, RecordingOrigin};
use crate::screenshots::test_output_settings;

fn scratch(name: &str) -> PathBuf {
  let directory = std::env::temp_dir()
    .join("screenwide-tests")
    .join(format!("look-{name}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

/// A project holding only its manifest, in a folder of its own.
fn project(directory: &Path) -> PathBuf {
  let root = directory.join("Demo");
  std::fs::create_dir_all(root.join("media")).unwrap();
  let file = root.join("Demo.screenwide");
  crate::project::write(
    &file,
    &Manifest::recording(RecordingManifest {
      duration_ms: Some(4_000),
      has_microphone: false,
      has_system_audio: false,
      media: RecordingMedia {
        primary: "media/recording.mov".to_owned(),
        camera: None,
        cursor: None,
        keyboard: None,
        moments: None,
      },
      primary_kind: PrimaryRecordingKind::Screen,
      source_scale_factor: 2.0,
      origin: RecordingOrigin::Capture,
    }),
  )
  .unwrap();
  file
}

/// A look whose background shows the picture at `background`, if any.
fn look(background: Option<&Path>) -> ProjectLook {
  let mut primary = test_output_settings(1_920, 1_080);
  primary.background_image_path = background.map(|path| path.to_string_lossy().into_owned());
  if background.is_some() {
    primary.background_type = "image".to_owned();
  }
  ProjectLook {
    bake_camera: true,
    camera_overlay: CameraOverlaySettings {
      camera_x: 1_400.0,
      camera_y: 750.0,
      camera_width: 320.0,
      frame_height: 180.0,
      frame_width: 180.0,
      frame_x: 1_310.0,
      frame_y: 660.0,
      radius_percent: 50.0,
    },
    cursor_effects: cursor_effects::CursorEffectSettings::default(),
    keyboard_effects: keyboard_effects::KeyboardEffectSettings::default(),
    recording_output: RecordingOutputSettings {
      camera: test_output_settings(320, 320),
      primary,
    },
    audio_track_volumes: None,
    enabled_stream_indices: Some(vec![1]),
    enabled_video_tracks: Some(vec![VideoTrack::Primary]),
  }
}

#[test]
fn a_background_picture_travels_with_the_project() {
  let directory = scratch("travels");
  let file = project(&directory);
  let picture = directory.join("Wallpaper.PNG");
  std::fs::write(&picture, b"picture").unwrap();

  persist(&file, 1, look(Some(&picture))).unwrap();
  std::fs::remove_file(&picture).unwrap();
  let moved = directory.join("Moved");
  std::fs::rename(file.parent().unwrap(), &moved).unwrap();
  let saved = for_project(&moved.join("Demo.screenwide")).unwrap();

  let background = saved
    .recording_output
    .primary
    .background_image_path
    .expect("the background is kept");
  assert!(background.starts_with(&*moved.to_string_lossy()));
  assert!(background.ends_with(".png"));
  assert_eq!(std::fs::read(background).unwrap(), b"picture");
  assert_eq!(saved.enabled_stream_indices, Some(vec![1]));
}

#[test]
fn a_picture_the_background_does_not_show_is_not_copied_in() {
  let directory = scratch("unshown");
  let file = project(&directory);
  let picture = directory.join("Wallpaper.png");
  std::fs::write(&picture, b"picture").unwrap();
  let mut colour = look(Some(&picture));
  colour.recording_output.primary.background_type = "solid".to_owned();

  persist(&file, 1, colour).unwrap();

  let saved = for_project(&file).unwrap();
  assert_eq!(
    saved
      .recording_output
      .primary
      .background_image_path
      .as_deref(),
    Some(&*picture.to_string_lossy())
  );
  assert!(!file.parent().unwrap().join("media/pictures").exists());
}

#[test]
fn an_older_save_does_not_replace_a_newer_one() {
  let directory = scratch("order");
  let file = project(&directory);
  let mut newer = look(None);
  newer.bake_camera = false;

  persist(&file, 2, newer).unwrap();
  persist(&file, 1, look(None)).unwrap();

  assert!(!for_project(&file).unwrap().bake_camera);
}

#[test]
fn a_background_named_outside_the_project_is_dropped() {
  let directory = scratch("outside");
  let file = project(&directory);
  persist(&file, 1, look(None)).unwrap();
  crate::project::update(&file, |manifest| {
    manifest.look.as_mut().unwrap()["look"]["recordingOutput"]["primary"]["backgroundImagePath"] =
      "../../secret.png".into();
    true
  })
  .unwrap();

  let saved = for_project(&file).unwrap();
  assert_eq!(saved.recording_output.primary.background_image_path, None);
}

#[test]
fn a_look_this_version_cannot_read_leaves_the_project_opening() {
  let directory = scratch("unreadable");
  let file = project(&directory);
  crate::project::update(&file, |manifest| {
    manifest.look = Some(serde_json::json!({ "version": 99, "anything": true }));
    true
  })
  .unwrap();

  assert!(crate::project::read(&file).is_ok());
  assert_eq!(for_project(&file), None);
}

fn pictures_in(file: &Path) -> Vec<Vec<u8>> {
  let folder = file.parent().unwrap().join("media/pictures");
  let mut pictures = std::fs::read_dir(folder)
    .map(|entries| {
      entries
        .flatten()
        .map(|entry| std::fs::read(entry.path()).unwrap())
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  pictures.sort();
  pictures
}

#[test]
fn pictures_shown_in_turn_leave_only_the_one_shown() {
  let directory = scratch("in-turn");
  let file = project(&directory);
  let app = directory.join("app");
  for (revision, name) in [(1, "first.png"), (2, "second.png")] {
    let picture = directory.join(name);
    std::fs::write(&picture, name).unwrap();
    persist(&file, revision, look(Some(&picture))).unwrap();
  }
  assert_eq!(
    pictures_in(&file).len(),
    2,
    "both stay while the editor is open"
  );

  cleanup::clean(&file, Some(&app)).unwrap();

  assert_eq!(pictures_in(&file), vec![b"second.png".to_vec()]);
  assert!(
    !app.exists(),
    "a picture nothing remembers is not kept anywhere"
  );
}

#[test]
fn a_picture_behind_a_colour_moves_out_of_the_project() {
  let directory = scratch("behind");
  let file = project(&directory);
  let app = directory.join("app");
  let picture = directory.join("Wallpaper.png");
  std::fs::write(&picture, b"picture").unwrap();
  persist(&file, 1, look(Some(&picture))).unwrap();
  // Reopened, the window names the picture inside the project; then a colour
  // is chosen, which keeps the picture to switch back to.
  let mut colour = for_project(&file).unwrap();
  colour.recording_output.primary.background_type = "solid".to_owned();
  persist(&file, 2, colour).unwrap();

  cleanup::clean(&file, Some(&app)).unwrap();

  assert!(pictures_in(&file).is_empty());
  let remembered = for_project(&file)
    .unwrap()
    .recording_output
    .primary
    .background_image_path
    .unwrap();
  assert!(Path::new(&remembered).starts_with(&app));
  assert_eq!(std::fs::read(remembered).unwrap(), b"picture");
}

#[test]
fn a_remembered_look_outlives_the_project_it_came_from() {
  let directory = scratch("remembered");
  let file = project(&directory);
  let app = directory.join("app");
  let picture = directory.join("Wallpaper.png");
  std::fs::write(&picture, b"picture").unwrap();
  persist(&file, 1, look(Some(&picture))).unwrap();
  // Exported from the reopened project, whose canvas names the picture
  // inside it; then the project is deleted after the export.
  let mut remembered = for_project(&file).unwrap().recording_output.primary;
  pictures::keep_in(&app, &mut remembered);
  std::fs::remove_dir_all(file.parent().unwrap()).unwrap();

  let background = remembered.background_image_path.unwrap();
  assert!(Path::new(&background).starts_with(&app));
  assert_eq!(std::fs::read(background).unwrap(), b"picture");
}
