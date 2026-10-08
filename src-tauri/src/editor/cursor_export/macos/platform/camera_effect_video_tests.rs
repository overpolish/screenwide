// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Annotations drawn on the camera through the real One video export: a
//! redaction covers the camera's pixels and ends at its edge, and a
//! magnifier's loupe, set beside the camera, shows the camera enlarged with
//! what was drawn into it.

use super::*;
use crate::editor::annotations::magnify::model::new_magnify;
use crate::editor::annotations::redact::new_redact;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::{
  Annotation, AnnotationPoint, AnnotationRedaction, AnnotationShape,
};
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use std::path::Path;
use std::process::Command;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
/// Where the 160 by 120 camera is drawn, one pixel to one: x, y, width and
/// height on the canvas.
const CAMERA: (u32, u32, u32, u32) = (80, 30, 160, 120);

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A flat-coloured movie two seconds long at `size`.
fn movie(path: &Path, colour: &str, size: (u32, u32)) {
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
    .arg(format!("color=c={colour}:s={}x{}:r=10:d=2", size.0, size.1))
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
    .arg(path)
    .status()
    .unwrap()
    .success());
}

/// A green box from `from` to `to`, in the camera's own pixels.
fn green_box(from: AnnotationPoint, to: AnnotationPoint) -> Annotation {
  let mut annotation = new_redact("redaction".to_owned(), from, None);
  if let AnnotationShape::Redact { end, .. } = &mut annotation.shape {
    *end = to;
  }
  annotation.style.redaction = AnnotationRedaction::Color;
  annotation.style.color = "#00ff00".to_owned();
  annotation
}

/// The frame a second into a One video of a black screen with a red camera
/// over its middle, `camera` drawn on the camera, as RGB, one pixel to one.
fn exported(name: &str, camera: Vec<Annotation>) -> impl Fn(u32, u32) -> [u8; 3] {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-camera-effect-{name}-{}",
    std::process::id()
  ));
  std::fs::create_dir_all(&directory).unwrap();
  let (screen, camera_path, destination) = (
    directory.join("screen.mov"),
    directory.join("camera.mov"),
    directory.join("output.mp4"),
  );
  movie(&screen, "black", (WIDTH, HEIGHT));
  movie(&camera_path, "red", (CAMERA.2, CAMERA.3));
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 1,
      keyboard_deletions: Box::default(),
      silence_cuts: Vec::new(),
      scene_clips: Vec::new(),
      annotation_clips: camera
        .into_iter()
        .map(|annotation| RecordingAnnotationClip {
          path_ms: None,
          pin: None,
          annotation,
          track_id: AnnotationTrack::Camera,
          start_ms: 0,
          end_ms: 2_000,
        })
        .collect(),
      segments: vec![RecordingTimelineSegment {
        id: 0,
        source_start: 0.0,
        source_end: 1.0,
        playback_rate: 1.0,
      }],
    },
    2_000,
  );
  let mut output = crate::screenshots::test_output_settings(WIDTH, HEIGHT);
  output.background_color = "#000000".into();
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
  let video = VideoExportOptions {
    compression: 0,
    resolution_scale_percent: 100,
    source_scale_percent: 100,
  };
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
        video,
      },
    )),
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
    timeline: timeline.as_ref(),
    video,
    width: WIDTH,
  });
  assert_eq!(result.unwrap(), ExportRunResult::Completed);
  let frame = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", "1.0", "-i"])
    .arg(&destination)
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
  std::fs::remove_dir_all(directory).unwrap();
  let rgb = frame.stdout;
  move |x, y| {
    let at = ((y * WIDTH + x) * 3) as usize;
    [rgb[at], rgb[at + 1], rgb[at + 2]]
  }
}

fn green([r, g, b]: [u8; 3]) -> bool {
  g > 180 && r < 80 && b < 80
}

fn red([r, g, b]: [u8; 3]) -> bool {
  r > 180 && g < 80 && b < 80
}

#[test]
fn a_camera_redaction_covers_the_camera_and_ends_at_its_edge() {
  // From well outside the camera's top-left corner to its middle.
  let pixel = exported(
    "redaction",
    vec![green_box(point(-60.0, -40.0), point(80.0, 60.0))],
  );
  let (x, y, width, height) = CAMERA;
  // Inside the camera, under the box: covered.
  assert!(green(pixel(x + 20, y + 20)), "{:?}", pixel(x + 20, y + 20));
  // Inside the camera, past the box: the camera as it was.
  assert!(red(pixel(x + width - 20, y + height - 20)));
  // Outside the camera, still under the box: the screen as it was.
  assert!(!green(pixel(x - 20, y + 20)), "{:?}", pixel(x - 20, y + 20));
  assert!(!green(pixel(x + 20, y - 15)), "{:?}", pixel(x + 20, y - 15));
}

#[test]
fn a_camera_magnifier_shows_the_camera_beside_it() {
  // A green box in the camera's top-left corner, and a magnifier whose zoom
  // area lies over the box and whose loupe sits on the screen left of the
  // camera, in the camera's own pixels.
  let mut magnifier = new_magnify("magnify".to_owned(), point(10.0, 10.0), None);
  magnifier.shape = AnnotationShape::Magnify {
    start: point(0.0, 0.0),
    end: point(20.0, 20.0),
    loupe: point(-50.0, 30.0),
    size: 40.0,
  };
  let pixel = exported(
    "magnifier",
    vec![green_box(point(0.0, 0.0), point(30.0, 30.0)), magnifier],
  );
  let (x, y, _, _) = CAMERA;
  // The loupe's middle, on the screen beside the camera: the green box the
  // zoom area covers, enlarged.
  assert!(green(pixel(x - 50, y + 30)), "{:?}", pixel(x - 50, y + 30));
  // The screen around it is still the screen.
  assert!(!green(pixel(x - 75, y + 85)) && !red(pixel(x - 75, y + 85)));
}

/// An erase fills its box with the surface around it, read from the
/// camera's own frames: red, where a fill the camera never handed over would
/// leave the box flat in its own green.
#[test]
fn a_camera_erase_fills_from_the_camera() {
  let mut erase = green_box(point(20.0, 20.0), point(60.0, 60.0));
  erase.style.redaction = AnnotationRedaction::Erase;
  let pixel = exported("erase", vec![erase]);
  let (x, y, _, _) = CAMERA;
  assert!(red(pixel(x + 40, y + 40)), "{:?}", pixel(x + 40, y + 40));
}
