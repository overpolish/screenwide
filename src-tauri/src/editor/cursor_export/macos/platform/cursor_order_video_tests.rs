// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cursor among the annotations through the real video export, where the
//! screen layer's annotation pass draws it rather than the cursor's own pass.

use super::*;
use crate::editor::annotations::spotlight::model::new_spotlight;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::AnnotationPoint;
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use crate::recording::cursor::{
  CursorRecord, CursorSource, CursorSourceKind, CursorStyle, FORMAT_VERSION,
};
use std::path::Path;
use std::process::Command;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
/// Where the cursor rests, well outside the spotlight's light.
const REST: (f64, f64) = (260.0, 140.0);

/// A two-second black recording, and a cursor resting at `REST` through it.
fn recording(directory: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
  let source = directory.join("black.mov");
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
    .arg(format!("color=c=black:s={WIDTH}x{HEIGHT}:r=10:d=2"))
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "8"])
    .arg(&source)
    .status()
    .unwrap()
    .success());
  let mut records = vec![
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "macos".to_owned(),
      source: CursorSource {
        height: f64::from(HEIGHT),
        kind: CursorSourceKind::Screen,
        platform_id: "test".to_owned(),
        video_height: HEIGHT,
        video_width: WIDTH,
        width: f64::from(WIDTH),
        x: 0.0,
        y: 0.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      height: 24.0,
      hotspot_x: 1.0,
      hotspot_y: 1.0,
      style: CursorStyle::Arrow,
      timestamp_us: 0,
      width: 16.0,
    },
  ];
  records.extend((0..=100).map(|step| CursorRecord::Position {
    timestamp_us: step * 20_000,
    x: REST.0,
    y: REST.1,
  }));
  let cursor = directory.join("black.cursor.jsonl");
  let json = records
    .iter()
    .map(|record| serde_json::to_string(record).unwrap())
    .collect::<Vec<_>>()
    .join("\n");
  std::fs::write(&cursor, format!("{json}\n")).unwrap();
  (source, cursor)
}

/// A spotlight over the frame's left half, for the whole recording.
fn spotlight() -> RecordingAnnotationClip {
  let point = |x, y| AnnotationPoint { x, y };
  let mut annotation = new_spotlight(
    "spotlight".to_owned(),
    [point(10.0, 10.0), point(150.0, 170.0)],
    None,
  );
  annotation.style.softness = 0.0;
  annotation.style.radius = 0.0;
  RecordingAnnotationClip {
    path_ms: None,
    pin: None,
    annotation,
    track_id: AnnotationTrack::Primary,
    start_ms: 0,
    end_ms: 2_000,
  }
}

/// The brightest green level round the resting cursor, one second into the
/// export of `source` and `cursor` with `clips` over it.
fn brightest_cursor(
  directory: &Path,
  name: &str,
  (source, cursor): &(std::path::PathBuf, std::path::PathBuf),
  clips: &[RecordingAnnotationClip],
) -> u8 {
  let destination = directory.join(name);
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 1,
      keyboard_deletions: Box::default(),
      annotation_clips: clips.to_vec(),
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
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    annotation_track: AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: Some(cursor),
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 2_000,
    height: HEIGHT,
    on_progress: &mut |_| {},
    output: &output,
    screen: source,
    selection: &TrackSelection::default(),
    timeline: timeline.as_ref(),
    video: VideoExportOptions {
      compression: 0,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
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
  let mut brightest = 0;
  for y in 130..175 {
    for x in 250..290 {
      brightest = brightest.max(frame.stdout[((y * WIDTH + x) * 3 + 1) as usize]);
    }
  }
  brightest
}

#[test]
fn an_exported_cursor_outside_a_spotlights_light_is_shaded() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-cursor-order-video-{}",
    std::process::id()
  ));
  std::fs::create_dir_all(&directory).unwrap();
  let recording = recording(&directory);
  let bare = brightest_cursor(&directory, "bare.mp4", &recording, &[]);
  let shaded = brightest_cursor(&directory, "shaded.mp4", &recording, &[spotlight()]);
  let _ = std::fs::remove_dir_all(&directory);
  // The cursor's white outline, drawn on its own, and then under the shade.
  assert!(bare > 200, "{bare}");
  assert!(shaded > 100 && shaded < 180, "{shaded}");
}
