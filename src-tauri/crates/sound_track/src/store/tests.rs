// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// One library for every test in the process, as the app has one.
fn library() -> &'static Path {
  static DATA: OnceLock<PathBuf> = OnceLock::new();
  let data = DATA.get_or_init(|| {
    let data =
      std::env::temp_dir().join(format!("screenwide-sound-library-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    initialize(&data);
    data
  });
  data
}

fn scratch(name: &str) -> PathBuf {
  let folder = library().join(format!("scratch-{name}"));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  folder
}

fn tone(folder: &Path, name: &str, frequency: u32, title: Option<&str>) -> PathBuf {
  let path = folder.join(name);
  let mut command = screenwide_sidecar::ffmpeg_command();
  command
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-nostdin",
      "-y",
      "-f",
      "lavfi",
      "-i",
    ])
    .arg(format!("sine=frequency={frequency}:duration=2"));
  if let Some(title) = title {
    command.args(["-metadata", &format!("title={title}")]);
  }
  assert!(command.arg(&path).status().unwrap().success());
  path
}

#[test]
fn a_sound_is_kept_once_by_its_contents_and_measured() {
  let folder = scratch("import");
  let file = tone(&folder, "Morning.wav", 440, None);
  let sound = import(&file).unwrap();
  assert!(is_name(&sound.name) && sound.name.ends_with(".wav"));
  assert_eq!(sound.title, "Morning");
  assert!((1_990..=2_010).contains(&sound.duration_ms));
  assert_eq!(sound.levels.len(), 20);
  assert!(sound.loudness_lufs.is_some());
  let copy = folder.join("Copy of morning.wav");
  std::fs::copy(&file, &copy).unwrap();
  assert_eq!(import(&copy).unwrap(), sound);
  assert!(sounds().contains(&sound));
}

#[test]
fn a_title_tag_names_the_sound() {
  let folder = scratch("title");
  let sound = import(&tone(&folder, "track01.m4a", 523, Some("Evening"))).unwrap();
  assert_eq!(sound.title, "Evening");
}

#[test]
fn a_file_with_no_sound_is_refused_and_not_kept() {
  let folder = scratch("silent");
  let file = folder.join("notes.mp3");
  std::fs::write(&file, b"not a sound").unwrap();
  assert!(import(&file).is_err());
  assert!(!sounds().iter().any(|sound| sound.title == "notes"));
}

#[test]
fn a_project_carries_its_sounds_and_hands_them_to_another_library() {
  let folder = scratch("carry");
  let sound = import(&tone(&folder, "Carried.wav", 330, None)).unwrap();
  let project = folder.join("project");
  carry(&project, [sound.name.as_str()]);
  remove(&sound.name).unwrap();
  assert!(!sounds().contains(&sound));
  let (path, resolved) = resolve(Some(&project), &sound.name).unwrap();
  assert_eq!((path, resolved), (project.join(&sound.name), sound.clone()));
  adopt(&project);
  assert!(sounds().contains(&sound));
}

#[test]
fn names_that_could_reach_outside_the_library_are_not_names() {
  assert!(!is_name("../0123456789abcdef0123456789abcdef.mp3"));
  assert!(!is_name("0123456789abcdef0123456789abcdef.exe"));
  assert!(is_name("0123456789abcdef0123456789abcdef.m4a"));
}
