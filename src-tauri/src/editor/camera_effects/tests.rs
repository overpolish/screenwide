// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::sync::Arc;

use super::balance;
use super::live::{LiveCameraEffects, MatteFile};
use super::look::{self, to_display};
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

/// `frames` frames of a 64 by 36 checkerboard of two greys, `dark` and
/// `light` in linear light, lit by `cast`, as the camera's bytes.
fn checkerboard(frames: usize, (dark, light): (f32, f32), cast: [f32; 3]) -> Vec<u8> {
  let mut bytes = Vec::new();
  for _ in 0..frames {
    for y in 0..36 {
      for x in 0..64 {
        let grey = if (x / 4 + y / 4) % 2 == 0 {
          dark
        } else {
          light
        };
        bytes.extend(cast.map(|cast| (to_display(grey * cast) * 255.0).round() as u8));
      }
    }
  }
  bytes
}

#[test]
fn enhance_takes_a_cast_out_of_a_grey_room() {
  let gains = balance::correction(
    &checkerboard(3, (0.2, 0.5), [0.8, 1.0, 1.25]),
    None,
    (64, 36),
  );
  assert!(
    gains[0] > gains[1] && gains[1] > gains[2],
    "red is lifted against blue: {gains:?}"
  );
  // With no skin to judge by, half the way: a cast of 1.56 between red and
  // blue is brought to about 1.25.
  let left = 1.5625 * gains[2] / gains[0];
  assert!((1.1..1.4).contains(&left), "part of the way: {left}");
}

#[test]
fn enhances_exposure_lifts_a_dark_camera_by_at_most_a_third_and_never_darkens() {
  let dark = balance::correction(&checkerboard(2, (0.05, 0.2), [1.0; 3]), None, (64, 36));
  assert!((dark[1] - 1.33).abs() < 1e-9);
  let bright = balance::correction(&checkerboard(2, (0.3, 1.0), [1.0; 3]), None, (64, 36));
  assert!((bright[1] - 1.0).abs() < 1e-9);
}

#[test]
fn enhances_look_keeps_greys_neutral_and_skin_its_hue() {
  let hue = |rgb: [f32; 3]| {
    let [cb, cr] = look::cbcr(rgb);
    cr.atan2(cb).to_degrees()
  };
  let grey = look::corrected([0.5; 3], [1.0; 3]);
  assert!(
    (grey[0] - grey[2]).abs() < 1e-4 && (grey[1] - grey[2]).abs() < 1e-4,
    "{grey:?}"
  );
  // A light and a dark skin tone each keep their hue to within a degree.
  for skin in [[0.80, 0.60, 0.50], [0.45, 0.30, 0.22]] {
    let looked = look::corrected(skin, [1.0; 3]);
    assert!((hue(looked) - hue(skin)).abs() < 1.0, "{skin:?} {looked:?}");
  }
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
