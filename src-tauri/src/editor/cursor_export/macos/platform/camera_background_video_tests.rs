// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's background through the real export: where the matte finds
//! no person the room is replaced by the picture on a baked-in camera, and
//! shows the canvas through where the camera is the recording itself.

use super::*;
use crate::editor::annotations::timing::AnnotationTrack;
use crate::editor::camera_effects::live::{LiveCameraEffects, MatteFile};
use crate::editor::camera_effects::CameraBackground;
use crate::editor::cursor_export::ExportCameraEffects;
use screenwide_transcriber::MatteTimings;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
/// Where the 160 by 120 camera is drawn, one pixel to one: x, y, width and
/// height on the canvas.
const CAMERA: (u32, u32, u32, u32) = (80, 30, 160, 120);

/// A movie two seconds long at `size`, drawn by the lavfi `filter` after a
/// black frame.
fn movie(path: &Path, filter: &str, size: (u32, u32)) {
  assert!(Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i"
    ])
    .arg(format!(
      "color=c=black:s={}x{}:r=10:d=2,{filter}",
      size.0, size.1
    ))
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
    .arg(path)
    .status()
    .unwrap()
    .success());
}

/// The frame a second into the export of `screen`, as RGB.
fn frame_of(destination: &Path) -> impl Fn(u32, u32) -> [u8; 3] {
  let frame = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", "1.0", "-i"])
    .arg(destination)
    .args([
      "-frames:v",
      "1",
      "-f",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      "pipe:1",
    ])
    .output()
    .unwrap();
  assert!(frame.status.success());
  let rgb = frame.stdout;
  move |x, y| {
    let at = ((y * WIDTH + x) * 3) as usize;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
  }
}

/// A red camera with a matte holding the person on its left half only, and
/// the effects `background` reading it, with a blue picture to hand.
fn camera_with_matte(directory: &Path, background: CameraBackground) -> ExportCameraEffects {
  let (camera, matte) = (directory.join("camera.mov"), directory.join("matte.mp4"));
  movie(&camera, "drawbox=c=red:t=fill", (CAMERA.2, CAMERA.3));
  movie(
    &matte,
    &format!(
      "drawbox=x=0:y=0:w={}:h={}:c=white:t=fill",
      CAMERA.2 / 2,
      CAMERA.3
    ),
    (CAMERA.2, CAMERA.3),
  );
  let blue = ::image::RgbaImage::from_pixel(64, 48, ::image::Rgba([0, 0, 255, 255]));
  ExportCameraEffects {
    live: Arc::new(LiveCameraEffects {
      background,
      grade: None,
      picture: Some(Arc::new(blue)),
      enhance: false,
      matte: Some(MatteFile {
        path: matte,
        timings: Arc::new(MatteTimings {
          format: MatteTimings::FORMAT,
          frames_per_second: 10,
          frames_ms: (0..20).map(|frame| f64::from(frame) * 100.0).collect(),
        }),
      }),
      generation: 1,
    }),
    on_screen: false,
  }
}

fn output_settings() -> crate::screenshots::ScreenshotOutputSettings {
  let mut output = crate::screenshots::test_output_settings(WIDTH, HEIGHT);
  output.background_color = "#00ff00".into();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.crop_width = f64::from(WIDTH);
  output.crop_height = f64::from(HEIGHT);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(WIDTH);
  output.image_x = 0.0;
  output.image_y = 0.0;
  output
}

fn directory(name: &str) -> std::path::PathBuf {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-camera-background-{name}-{}",
    std::process::id()
  ));
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

const VIDEO: VideoExportOptions = VideoExportOptions {
  compression: 0,
  resolution_scale_percent: 100,
  source_scale_percent: 100,
};

fn near([r, g, b]: [u8; 3], [er, eg, eb]: [u8; 3]) -> bool {
  r.abs_diff(er) < 60 && g.abs_diff(eg) < 60 && b.abs_diff(eb) < 60
}

#[test]
fn a_baked_camera_shows_its_picture_where_the_matte_finds_no_person() {
  let directory = directory("baked");
  let (screen, destination) = (directory.join("screen.mov"), directory.join("output.mp4"));
  movie(&screen, "null", (WIDTH, HEIGHT));
  let effects = camera_with_matte(&directory, CameraBackground::Image);
  let camera_path = directory.join("camera.mov");
  let output = output_settings();
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    annotation_track: AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: Some((
      &camera_path,
      BakedVideoExportOptions {
        camera_drop_shadow: false,
        camera_width: CAMERA.2,
        camera_height: CAMERA.3,
        screen_width: WIDTH,
        screen_height: HEIGHT,
        overlay: crate::editor::CameraOverlaySettings {
          camera_width: f64::from(CAMERA.2),
          camera_x: f64::from(CAMERA.0 + CAMERA.2 / 2),
          camera_y: f64::from(CAMERA.1 + CAMERA.3 / 2),
          frame_width: f64::from(CAMERA.2),
          frame_height: f64::from(CAMERA.3),
          frame_x: f64::from(CAMERA.0),
          frame_y: f64::from(CAMERA.1),
          radius_percent: 0.0,
        },
        video: VIDEO,
      },
    )),
    camera_effects: Some(effects),
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 2_000,
    height: HEIGHT,
    on_progress: &mut |_| {},
    output: &output,
    screen: &screen,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VIDEO,
    width: WIDTH,
  });
  assert_eq!(result.unwrap(), ExportRunResult::Completed);
  let pixel = frame_of(&destination);
  std::fs::remove_dir_all(directory).unwrap();
  let middle = CAMERA.1 + CAMERA.3 / 2;
  let person = pixel(CAMERA.0 + CAMERA.2 / 4, middle);
  let room = pixel(CAMERA.0 + CAMERA.2 * 3 / 4, middle);
  assert!(near(person, [255, 0, 0]), "the person stays: {person:?}");
  assert!(near(room, [0, 0, 255]), "the room is the picture: {room:?}");
}

#[test]
fn a_camera_recording_without_a_background_shows_the_canvas_behind_the_person() {
  let directory = directory("transparent");
  let destination = directory.join("output.mp4");
  let mut effects = camera_with_matte(&directory, CameraBackground::Transparent);
  effects.on_screen = true;
  let camera_path = directory.join("camera.mov");
  // The camera fills the canvas, inset to show the green canvas around it.
  let mut output = output_settings();
  output.crop_width = f64::from(CAMERA.2);
  output.crop_height = f64::from(CAMERA.3);
  output.image_width = f64::from(CAMERA.2);
  output.image_x = f64::from(CAMERA.0);
  output.image_y = f64::from(CAMERA.1);
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    annotation_track: AnnotationTrack::Camera,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_effects: Some(effects),
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 2_000,
    height: CAMERA.3,
    on_progress: &mut |_| {},
    output: &output,
    screen: &camera_path,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VIDEO,
    width: CAMERA.2,
  });
  assert_eq!(result.unwrap(), ExportRunResult::Completed);
  let pixel = frame_of(&destination);
  std::fs::remove_dir_all(directory).unwrap();
  let middle = CAMERA.1 + CAMERA.3 / 2;
  let person = pixel(CAMERA.0 + CAMERA.2 / 4, middle);
  let room = pixel(CAMERA.0 + CAMERA.2 * 3 / 4, middle);
  assert!(near(person, [255, 0, 0]), "the person stays: {person:?}");
  assert!(
    near(room, [0, 255, 0]),
    "the canvas shows through: {room:?}"
  );
}
