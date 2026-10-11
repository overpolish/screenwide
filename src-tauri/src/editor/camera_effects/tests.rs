// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::sync::Arc;

use super::grade::{self, balanced, towards};
use super::live::{LiveCameraEffects, MatteFile};
use super::{cpu, image, keep, make, matte, tidy, CameraBackground, CameraEffects};
use crate::editor::annotations::pin::LumaFrame;
use crate::screenshots::CapturedImage;

#[test]
fn the_matte_of_the_frame_on_screen_is_found_however_unsteady_the_camera() {
  // A camera whose frames come early and late around 30 a second, as a
  // webcam's do, matted into a file at a steady 30.
  let file = MatteFile {
    path: PathBuf::new(),
    timings: Arc::new(screenwide_transcriber::MatteTimings {
      format: screenwide_transcriber::MatteTimings::FORMAT,
      frames_per_second: 30,
      frames_ms: vec![0.0, 5.0, 38.333, 71.667, 140.0, 141.5],
    }),
  };
  let slot = |frame: u64| (frame * 1_000).div_ceil(30);
  // Each moment shows the last frame stamped by then, as the decoders hand
  // it over, up to 2 ms early.
  for (moment, frame) in [
    (0, 0),
    (4, 1),
    (37, 2),
    (70, 3),
    (137, 3),
    (139, 4),
    (140, 5),
    (500, 5),
  ] {
    assert_eq!(file.matte_ms(moment), slot(frame), "at {moment} ms");
  }
}

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
      colour: false,
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

/// A 2 by 1 red frame, and a matte holding the person on its left only.
fn frame_and_matte() -> (CapturedImage, LumaFrame) {
  let frame = CapturedImage {
    width: 2,
    height: 1,
    rgba: vec![200, 0, 0, 255, 200, 0, 0, 255],
  };
  let matte = LumaFrame {
    ms: 0,
    width: 2,
    height: 1,
    pixels: vec![255, 0],
  };
  (frame, matte)
}

fn effects(background: CameraBackground, picture: Option<::image::RgbaImage>) -> LiveCameraEffects {
  LiveCameraEffects {
    background,
    grade: None,
    picture: picture.map(Arc::new),
    matte: None,
    enhance: false,
    generation: 0,
  }
}

#[test]
fn the_background_is_taken_from_where_the_matte_finds_no_person() {
  // No background: the room becomes see-through and the person stays.
  let (mut frame, matte) = frame_and_matte();
  cpu::apply(
    &mut frame,
    Some(&matte),
    &effects(CameraBackground::Transparent, None),
  );
  assert_eq!(frame.rgba, [200, 0, 0, 255, 200, 0, 0, 0]);

  // A picture: the room is the picture, still solid.
  let (mut frame, matte) = frame_and_matte();
  let blue = ::image::RgbaImage::from_pixel(4, 2, ::image::Rgba([0, 0, 255, 255]));
  cpu::apply(
    &mut frame,
    Some(&matte),
    &effects(CameraBackground::Image, Some(blue)),
  );
  assert_eq!(frame.rgba, [200, 0, 0, 255, 0, 0, 255, 255]);

  // Without its matte the camera is drawn as recorded.
  let (mut frame, _) = frame_and_matte();
  cpu::apply(
    &mut frame,
    None,
    &effects(CameraBackground::Transparent, None),
  );
  assert_eq!(frame.rgba, [200, 0, 0, 255, 200, 0, 0, 255]);
}

#[test]
fn a_balanced_grade_takes_out_the_rooms_cast_and_lifts_a_dark_subject() {
  // A dim camera in a blue-lit room.
  let frame = grade::stats([0.30, 0.32, 0.45], 0.15);
  let subject = grade::stats([0.28, 0.25, 0.30], 0.12);
  let grade = balanced(&frame, &subject);
  assert!(
    grade.gain[2] < grade.gain[0],
    "blue is held back against red: {grade:?}"
  );
  let lifted = |channel: usize| subject.mean[channel] * grade.gain[channel] + grade.offset[channel];
  let luma = [0.2126, 0.7152, 0.0722]
    .iter()
    .enumerate()
    .map(|(channel, weight)| lifted(channel) * weight)
    .sum::<f64>();
  assert!(
    luma > subject.luma_mean && luma < 0.48,
    "lifted part of the way: {luma}"
  );
  // No channel is pushed past the bounds, however far off the camera is.
  let black = grade::stats([0.0, 0.0, 0.0], 0.0);
  for gain in balanced(&black, &black).gain {
    assert!((0.7..=1.9).contains(&gain), "{gain}");
  }
}

#[test]
fn a_grade_towards_a_picture_goes_only_part_of_the_way() {
  let subject = grade::stats([0.5, 0.45, 0.4], 0.2);
  let sunset = grade::stats([0.8, 0.5, 0.3], 0.2);
  let grade = towards(&subject, &sunset);
  let red = subject.mean[0] * grade.gain[0] + grade.offset[0];
  let blue = subject.mean[2] * grade.gain[2] + grade.offset[2];
  assert!(red > subject.mean[0] && red < sunset.mean[0], "{red}");
  assert!(blue < subject.mean[2] && blue > sunset.mean[2], "{blue}");
}
