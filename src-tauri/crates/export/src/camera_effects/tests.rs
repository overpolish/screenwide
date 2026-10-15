// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::sync::Arc;

use super::balance;
use super::live::{LiveBackground, LiveCameraEffects, MatteFile};
use super::look::{self, to_display};
use super::{cpu, CameraBackground};
use screenwide_annotations::pin::LumaFrame;
use screenwide_imaging::CapturedImage;

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
    covers: (0, u64::MAX),
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

const VIDEO_MS: u64 = 10_000;

/// Effects taking the background away over `ranges` of a ten-second video,
/// each `[start_ms, end_ms)` with what is behind the person there.
fn ranged(
  ranges: &[(u64, u64, CameraBackground)],
  picture: Option<::image::RgbaImage>,
) -> LiveCameraEffects {
  let picture = picture.map(Arc::new);
  LiveCameraEffects {
    backgrounds: ranges
      .iter()
      .map(|&(start_ms, end_ms, background)| LiveBackground {
        start_ms,
        end_ms,
        background,
        picture: picture.clone(),
      })
      .collect(),
    timeline: Vec::new(),
    video_ms: VIDEO_MS,
    grade: None,
    matte: Vec::new(),
    enhance: false,
    generation: 0,
  }
}

fn effects(background: CameraBackground, picture: Option<::image::RgbaImage>) -> LiveCameraEffects {
  ranged(&[(0, VIDEO_MS, background)], picture)
}

#[test]
fn the_background_is_taken_from_where_the_matte_finds_no_person() {
  // No background: the room becomes see-through and the person stays.
  let (mut frame, matte) = frame_and_matte();
  cpu::apply(
    &mut frame,
    (Some(&matte), 5_000),
    &effects(CameraBackground::Transparent, None),
  );
  assert_eq!(frame.rgba, [200, 0, 0, 255, 200, 0, 0, 0]);

  // A picture: the room is the picture, still solid.
  let (mut frame, matte) = frame_and_matte();
  let blue = ::image::RgbaImage::from_pixel(4, 2, ::image::Rgba([0, 0, 255, 255]));
  cpu::apply(
    &mut frame,
    (Some(&matte), 5_000),
    &effects(CameraBackground::Image, Some(blue)),
  );
  assert_eq!(frame.rgba, [200, 0, 0, 255, 0, 0, 255, 255]);

  // Without its matte the camera is drawn as recorded.
  let (mut frame, _) = frame_and_matte();
  cpu::apply(
    &mut frame,
    (None, 5_000),
    &effects(CameraBackground::Transparent, None),
  );
  assert_eq!(frame.rgba, [200, 0, 0, 255, 200, 0, 0, 255]);
}

/// How see-through the room is at `ms` with the background taken away over
/// `ranges`: 0 as recorded, 255 gone.
fn room_gone_at(ranges: &[(u64, u64, CameraBackground)], ms: u64) -> u8 {
  let (mut frame, matte) = frame_and_matte();
  cpu::apply(&mut frame, (Some(&matte), ms), &ranged(ranges, None));
  255 - frame.rgba[7]
}

#[test]
fn the_room_stays_outside_a_range_and_fades_at_its_edges() {
  let ranges = [(2_000, 6_000, CameraBackground::Transparent)];
  assert_eq!(room_gone_at(&ranges, 1_000), 0);
  assert_eq!(room_gone_at(&ranges, 7_000), 0);
  assert_eq!(room_gone_at(&ranges, 4_000), 255);
  // Halfway through the 600 ms the room takes to go, and to come back.
  let going = room_gone_at(&ranges, 2_300);
  let coming = room_gone_at(&ranges, 5_700);
  assert!((100..156).contains(&going), "{going}");
  assert!((100..156).contains(&coming), "{coming}");
}

#[test]
fn butted_ranges_hand_over_without_the_room_coming_back() {
  let ranges = [
    (2_000, 4_000, CameraBackground::Transparent),
    (4_000, 6_000, CameraBackground::Transparent),
  ];
  for ms in [3_900, 4_000, 4_100] {
    assert_eq!(room_gone_at(&ranges, ms), 255, "at {ms} ms");
  }
}

#[test]
fn a_range_at_the_start_of_the_video_holds_from_its_first_frame() {
  let ranges = [(0, 3_000, CameraBackground::Transparent)];
  assert_eq!(room_gone_at(&ranges, 0), 255);
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
