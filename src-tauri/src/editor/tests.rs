// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  save::{save_recording_copy, save_selected_recording_copy},
  *,
};

#[test]
fn keeps_a_reasonable_name_untouched() {
  assert_eq!(
    sanitize_file_stem("Screenwide 2026-08-08 at 14.32.05").as_deref(),
    Some("Screenwide 2026-08-08 at 14.32.05")
  );
}

#[test]
fn strips_characters_neither_platform_allows() {
  assert_eq!(
    sanitize_file_stem(r#"a<b>c:d"e/f\g|h?i*j"#).as_deref(),
    Some("abcdefghij")
  );
}

#[test]
fn strips_control_characters() {
  assert_eq!(
    sanitize_file_stem("one\ttwo\nthree").as_deref(),
    Some("onetwothree")
  );
}

#[test]
fn trims_surrounding_whitespace() {
  assert_eq!(sanitize_file_stem("   shot   ").as_deref(), Some("shot"));
}

#[test]
fn drops_trailing_dots_and_spaces_that_windows_would_eat() {
  assert_eq!(sanitize_file_stem("shot. . .").as_deref(), Some("shot"));
  assert_eq!(sanitize_file_stem("shot   ").as_deref(), Some("shot"));
}

#[test]
fn rejects_a_name_with_nothing_left_in_it() {
  assert_eq!(sanitize_file_stem(""), None);
  assert_eq!(sanitize_file_stem("   "), None);
  assert_eq!(sanitize_file_stem("///"), None);
  assert_eq!(sanitize_file_stem("..."), None);
}

#[test]
fn rejects_names_windows_reserves() {
  assert_eq!(sanitize_file_stem("CON"), None);
  assert_eq!(sanitize_file_stem("nul"), None);
  assert_eq!(sanitize_file_stem("Com1"), None);
  assert_eq!(sanitize_file_stem("LPT9"), None);
  // Only the exact stem is reserved.
  assert_eq!(sanitize_file_stem("console").as_deref(), Some("console"));
}

#[test]
fn caps_an_absurdly_long_name() {
  let stem = sanitize_file_stem(&"a".repeat(500)).unwrap();
  assert_eq!(stem.len(), MAX_FILE_STEM);
}

/// A directory of this test module's own, so a test that writes files cannot
/// be confused by anything else on the machine.
fn test_directory(name: &str) -> PathBuf {
  let directory = std::env::temp_dir().join("screenwide-tests").join(name);
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

#[test]
fn describes_a_recording_by_the_file_the_user_will_actually_get() {
  let working = Path::new("/tmp/recording-20260808-143205.000.mov");
  assert_eq!(delivered_extension(working, true), "mp4");
  // Nothing to copy it with, so what is offered is the movie itself - never
  // that movie under a name it does not answer to.
  assert_eq!(delivered_extension(working, false), "mov");
  // A project recorded on Windows is already what it would have been
  // remuxed into.
  assert_eq!(
    delivered_extension(Path::new("/tmp/recording-1.mp4"), false),
    "mp4"
  );
}

#[test]
fn accepts_only_the_camera_resolution_choices_the_window_offers() {
  for scale in [50, 75, 100] {
    assert!(validate_camera_resolution_scale(scale).is_ok());
  }
  for scale in [0, 49, 76, 101] {
    assert!(validate_camera_resolution_scale(scale).is_err());
  }
}

#[test]
fn accepts_only_camera_overlay_values_the_window_can_produce() {
  let canvas = (1_000_u32, 1_000_u32);
  let valid = CameraOverlaySettings {
    camera_x: 500.0,
    camera_y: 500.0,
    camera_width: 600.0,
    frame_height: 400.0,
    frame_width: 600.0,
    frame_x: 400.0,
    frame_y: 300.0,
    radius_percent: 50.0,
  };
  assert!(validate_camera_overlay(valid, canvas).is_ok());
  assert!(validate_camera_overlay(
    CameraOverlaySettings {
      camera_width: 20.0,
      ..valid
    },
    canvas
  )
  .is_err());
  assert!(validate_camera_overlay(
    CameraOverlaySettings {
      frame_width: 8_010.0,
      ..valid
    },
    canvas
  )
  .is_err());
  assert!(validate_camera_overlay(
    CameraOverlaySettings {
      camera_x: -200.0,
      frame_x: -300.0,
      ..valid
    },
    canvas
  )
  .is_ok());
  assert!(validate_camera_overlay(
    CameraOverlaySettings {
      camera_x: f64::NAN,
      ..valid
    },
    canvas
  )
  .is_err());
}

/// Stands in for a stream copy that works, without needing FFmpeg to be on
/// the machine running the test.
fn copies(source: &Path, destination: &Path) -> Result<(), String> {
  std::fs::copy(source, destination)
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn refuses(_: &Path, _: &Path) -> Result<(), String> {
  Err("no".to_owned())
}

fn copies_selected(
  source: &Path,
  destination: &Path,
  _: &track_selection::TrackSelection,
  _: track_selection::AudioLayout,
  _: media_preview::ExportRunOptions<'_>,
) -> Result<media_preview::ExportRunResult, String> {
  copies(source, destination).map(|()| media_preview::ExportRunResult::Completed)
}

fn refuses_selected(
  _: &Path,
  _: &Path,
  _: &track_selection::TrackSelection,
  _: track_selection::AudioLayout,
  _: media_preview::ExportRunOptions<'_>,
) -> Result<media_preview::ExportRunResult, String> {
  Err("no".to_owned())
}

#[test]
fn saves_a_recording_as_an_mp4_when_it_can_be_copied_into_one() {
  let directory = test_directory("save-remuxed");
  let working = directory.join("recording-20260808-143205.000.mov");
  std::fs::write(&working, b"movie").unwrap();

  let saved = save_recording_copy(&working, &directory, "Keeper", Some(copies)).unwrap();

  assert_eq!(saved, directory.join("Keeper.mp4"));
  assert!(saved.is_file());
  // The recording stays in its project, ready to export again.
  assert!(working.is_file());
}

#[test]
fn saves_a_recording_as_the_movie_it_is_when_there_is_nothing_to_copy_it_with() {
  let directory = test_directory("save-without-ffmpeg");
  let working = directory.join("recording-20260808-143205.000.mov");
  std::fs::write(&working, b"movie").unwrap();

  let saved = save_recording_copy(&working, &directory, "Keeper", None).unwrap();

  // A .mov named .mp4 is a file that lies about itself, so the honest name
  // is the one the user gets.
  assert_eq!(saved, directory.join("Keeper.mov"));
  assert!(saved.is_file());
  assert!(!directory.join("Keeper.mp4").exists());
  assert!(working.is_file());
}

#[test]
fn saves_a_recording_as_the_movie_it_is_when_the_copy_fails() {
  let directory = test_directory("save-failed-remux");
  let working = directory.join("recording-20260808-143205.000.mov");
  std::fs::write(&working, b"movie").unwrap();

  let saved = save_recording_copy(&working, &directory, "Keeper", Some(refuses)).unwrap();

  // FFmpeg refusing the file is no reason to lose a recording someone just
  // asked to keep.
  assert_eq!(saved, directory.join("Keeper.mov"));
  assert!(saved.is_file());
  assert!(!directory.join("Keeper.mp4").exists());
}

#[test]
fn saves_beside_a_name_that_is_already_taken() {
  let directory = test_directory("save-collision");
  let working = directory.join("recording-20260808-143205.000.mov");
  std::fs::write(&working, b"movie").unwrap();
  std::fs::write(directory.join("Keeper.mp4"), b"someone else's").unwrap();

  let saved = save_recording_copy(&working, &directory, "Keeper", Some(copies)).unwrap();

  assert_eq!(saved, directory.join("Keeper (2).mp4"));
}

#[test]
fn saves_a_selected_audio_layout_without_changing_the_working_movie() {
  let directory = test_directory("save-selected-audio");
  let working = directory.join("recording-20260808-143205.000.mov");
  std::fs::write(&working, b"movie").unwrap();
  let tracks = recording_audio_tracks(true, true);
  let selection = track_selection::TrackSelection::new(&tracks, &[1]);
  let cancelled = AtomicBool::new(false);
  let mut ignore_progress = |_| {};

  let saved = save_selected_recording_copy(
    &working,
    &directory,
    "Keeper",
    &selection,
    track_selection::AudioLayout::SeparateTracks,
    media_preview::ExportRunOptions {
      cancelled: &cancelled,
      on_progress: &mut ignore_progress,
      timeline: None,
      video: media_preview::VideoExportOptions {
        compression: 0,
        resolution_scale_percent: 200,
        source_scale_percent: 200,
      },
    },
    Some(copies_selected),
  )
  .unwrap();

  assert_eq!(saved, Some(directory.join("Keeper.mp4")));
  assert!(working.is_file());
}

#[test]
fn keeps_the_working_movie_when_a_selected_audio_export_fails() {
  let directory = test_directory("save-selected-audio-failure");
  let working = directory.join("recording-20260808-143205.000.mov");
  std::fs::write(&working, b"movie").unwrap();
  let tracks = recording_audio_tracks(true, true);
  let selection = track_selection::TrackSelection::new(&tracks, &[1]);
  let cancelled = AtomicBool::new(false);
  let mut ignore_progress = |_| {};

  assert!(save_selected_recording_copy(
    &working,
    &directory,
    "Keeper",
    &selection,
    track_selection::AudioLayout::SeparateTracks,
    media_preview::ExportRunOptions {
      cancelled: &cancelled,
      on_progress: &mut ignore_progress,
      timeline: None,
      video: media_preview::VideoExportOptions {
        compression: 0,
        resolution_scale_percent: 200,
        source_scale_percent: 200,
      },
    },
    Some(refuses_selected),
  )
  .is_err());
  assert!(working.exists());
  assert!(!directory.join("Keeper.mp4").exists());
}

/// The one test here that uses the real stream copy. It is skipped rather
/// than failed on a machine without FFmpeg, because that machine is exactly
/// the one the fallback above exists for.
#[test]
fn carries_every_recorded_track_into_the_saved_mp4() {
  let Some(remux) = media_preview::remuxer() else {
    eprintln!("skipped: FFmpeg is not on this machine");
    return;
  };

  let directory = test_directory("save-real-remux");
  let working = directory.join("recording-20260808-143205.000.mov");
  // A picture and two audio tracks, which is what a recording with both
  // system audio and a microphone carries.
  let built = std::process::Command::new("ffmpeg")
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "testsrc=size=320x240:rate=30:duration=1",
      "-f",
      "lavfi",
      "-i",
      "sine=frequency=440:duration=1",
      "-f",
      "lavfi",
      "-i",
      "sine=frequency=880:duration=1",
      "-c:v",
      "libx264",
      "-c:a",
      "aac",
      "-map",
      "0:v",
      "-map",
      "1:a",
      "-map",
      "2:a",
    ])
    .arg(&working)
    .status();
  if !built.is_ok_and(|status| status.success()) {
    eprintln!("skipped: this FFmpeg could not build the source movie");
    return;
  }

  let saved = save_recording_copy(&working, &directory, "Keeper", Some(remux)).unwrap();
  assert_eq!(saved, directory.join("Keeper.mp4"));

  // Three streams in, three streams out. Dropping the second audio track
  // here would be silent data loss, which is the whole reason the copy maps
  // every stream rather than the first of each kind.
  assert_eq!(streams(&saved), 3);
}

/// How many streams a file holds, read out of what FFmpeg prints about it.
fn streams(path: &Path) -> usize {
  let output = std::process::Command::new("ffmpeg")
    .args(["-hide_banner", "-nostdin", "-i"])
    .arg(path)
    .output()
    .unwrap();

  String::from_utf8_lossy(&output.stderr)
    .lines()
    .filter(|line| line.trim_start().starts_with("Stream #"))
    .count()
}

#[test]
fn keeps_a_dot_inside_the_name() {
  assert_eq!(
    sanitize_file_stem("v1.2.3 build").as_deref(),
    Some("v1.2.3 build")
  );
}

#[test]
fn opens_the_project_a_launch_names_relative_to_where_it_ran() {
  let cwd = Path::new("/Users/demo/Desktop");
  assert_eq!(
    project_argument(
      [
        "C:\\Program Files\\Screenwide\\screenwide.exe",
        "--flag",
        "Demo.SCREENWIDE"
      ],
      cwd,
    ),
    Some(cwd.join("Demo.SCREENWIDE"))
  );
  assert_eq!(
    project_argument(["/a/First.screenwide", "/b/Second.screenwide"], cwd),
    Some(PathBuf::from("/b/Second.screenwide"))
  );
  assert_eq!(project_argument(["movie.mov", "-psn_0_1234"], cwd), None);
}
