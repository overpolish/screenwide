// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Scenes through the real video export: the screen and the baked camera are
//! drawn where the scene puts them, the screen zoomed as the scene frames it,
//! and where the recording puts them outside it.

use super::tests::output;
use super::*;
use crate::editor::scenes::RecordingSceneClip;
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use crate::editor::CameraOverlaySettings;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) const WIDTH: u32 = 640;
pub(super) const HEIGHT: u32 = 360;

/// Encodes the `lavfi` source `graph`, which sets its own size and length.
pub(super) fn clip(path: &Path, graph: &str) {
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
    .arg(graph)
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
    .arg(path)
    .status()
    .unwrap()
    .success());
}

/// The exported frame `seconds` in, as rows of RGB.
pub(super) fn frame_at(video: &Path, seconds: f64) -> Vec<u8> {
  let frame = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss"])
    .arg(seconds.to_string())
    .arg("-i")
    .arg(video)
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
  frame.stdout
}

pub(super) fn pixel(frame: &[u8], x: u32, y: u32) -> [u8; 3] {
  let index = ((y * WIDTH + x) * 3) as usize;
  [frame[index], frame[index + 1], frame[index + 2]]
}

pub(super) fn red(pixel: [u8; 3]) -> bool {
  pixel[0] > 150 && pixel[1] < 80 && pixel[2] < 80
}

pub(super) fn blue(pixel: [u8; 3]) -> bool {
  pixel[2] > 150 && pixel[0] < 80
}

/// A fresh directory for one test's clips and export.
pub(super) fn directory(name: &str) -> PathBuf {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-scene-video-{name}-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  directory
}

/// A two second timeline holding `scene`, a clip as the webview sends it.
pub(super) fn timeline(scene: serde_json::Value) -> TimelinePlan {
  let scene: RecordingSceneClip = serde_json::from_value(scene).unwrap();
  TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      annotation_clips: Vec::new(),
      artifact_id: 1,
      keyboard_deletions: Box::default(),
      silence_cuts: Vec::new(),
      scene_clips: vec![scene],
      next_segment_id: 1,
      segments: vec![RecordingTimelineSegment {
        id: 0,
        playback_rate: 1.0,
        source_end: 1.0,
        source_start: 0.0,
      }],
    },
    2_000,
  )
  .expect("a timeline with a scene is a plan")
}

/// Exports `screen`, with `camera` baked in where given, on `timeline`.
pub(super) fn exported(
  screen: &Path,
  camera: Option<(&Path, BakedVideoExportOptions)>,
  timeline: &TimelinePlan,
  destination: &Path,
) {
  let cancelled = AtomicBool::new(false);
  let video = VideoExportOptions {
    compression: 1,
    resolution_scale_percent: 100,
    source_scale_percent: 100,
  };
  let result = export(CursorExportRequest {
    annotation_track: crate::editor::annotations::timing::AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera,
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination,
    duration_ms: 2_000,
    height: HEIGHT,
    on_progress: &mut |_| {},
    output: &output(WIDTH, HEIGHT),
    screen,
    selection: &TrackSelection::default(),
    timeline: Some(timeline),
    video,
    width: WIDTH,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
}

#[test]
fn exports_the_screen_and_camera_where_the_scene_puts_them() {
  let directory = directory("split");
  let screen = directory.join("screen.mov");
  let camera = directory.join("camera.mov");
  clip(&screen, "color=c=blue:s=640x360:r=30:d=2");
  clip(&camera, "color=c=red:s=320x180:r=30:d=2");
  // Side by side from half a second to 1.9s, settled from 1.1s to 1.3s.
  let timeline = timeline(serde_json::json!({
    "id": "a", "startMs": 500, "endMs": 1_900, "preset": "split-two-thirds"
  }));
  let destination = directory.join("scene.mp4");
  let options = BakedVideoExportOptions {
    camera_drop_shadow: false,
    camera_height: 180,
    camera_width: 320,
    overlay: CameraOverlaySettings {
      camera_width: 80.0,
      camera_x: 100.0,
      camera_y: 70.0,
      frame_height: 45.0,
      frame_width: 80.0,
      frame_x: 60.0,
      frame_y: 47.5,
      radius_percent: 0.0,
    },
    screen_height: HEIGHT,
    screen_width: WIDTH,
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
  };
  exported(&screen, Some((&camera, options)), &timeline, &destination);

  // Before the scene the screen fills the canvas, with the camera in its
  // corner.
  let before = frame_at(&destination, 0.25);
  assert!(blue(pixel(&before, 320, 300)));
  assert!(red(pixel(&before, 100, 70)));
  assert!(blue(pixel(&before, 510, 180)));

  // Halfway through arriving, the frame at 0.8s draws the screen's right edge
  // at x 520, and its shutter opened with that edge near x 539. The screen
  // smears over the stretch between rather than stopping sharp.
  let moving = frame_at(&destination, 0.8);
  let smeared = pixel(&moving, 530, 180)[2];
  assert!(
    (40..200).contains(&smeared),
    "the moving edge is not blurred: {smeared}"
  );
  assert!(blue(pixel(&moving, 400, 180)));

  // Settled, the screen is a 360 by 202 pane from x 40 and the camera a
  // 180 wide pane from x 420, both with the black canvas around them.
  let settled = frame_at(&destination, 1.2);
  assert!(blue(pixel(&settled, 200, 180)));
  assert!(red(pixel(&settled, 510, 180)));
  assert!(pixel(&settled, 20, 20).iter().all(|channel| *channel < 40));
  assert!(!red(pixel(&settled, 100, 70)));
  let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn exports_a_zoom_into_the_screen_without_a_camera() {
  let directory = directory("zoom");
  let screen = directory.join("screen.mov");
  // The screen's left half is red and its right half blue.
  clip(
    &screen,
    "color=c=blue:s=640x360:r=30:d=2,drawbox=x=0:y=0:w=320:h=360:color=red:t=fill",
  );
  // Zoomed twice into the middle of the left half from half a second to the
  // end, settled from 1.1s to 1.4s.
  let timeline = timeline(serde_json::json!({
    "id": "z", "startMs": 500, "endMs": 2_000, "preset": "full",
    "screen": { "focusX": 0.25, "focusY": 0.5, "zoom": 2.0 }
  }));
  let destination = directory.join("zoom.mp4");
  exported(&screen, None, &timeline, &destination);

  let before = frame_at(&destination, 0.25);
  assert!(red(pixel(&before, 40, 180)));
  assert!(blue(pixel(&before, 600, 180)));
  // Settled, the red half fills the whole canvas.
  let zoomed = frame_at(&destination, 1.2);
  assert!(red(pixel(&zoomed, 40, 180)));
  assert!(red(pixel(&zoomed, 600, 180)));
  let _ = std::fs::remove_dir_all(directory);
}
