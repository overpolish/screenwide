// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Magnifiers through the real video export: the loupe reads the frame's own
//! planes, after its redactions.

use super::*;
use crate::editor::annotations::magnify::model::new_magnify;
use crate::editor::annotations::redact::model::new_redact;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::{
  Annotation, AnnotationPoint, AnnotationRedaction, AnnotationShape,
};
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use std::path::{Path, PathBuf};
use std::process::Command;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

/// A two-second recording of a still frame: dark on the left half, bright
/// on the right.
fn recording(directory: &Path) -> PathBuf {
  let path = directory.join("halves.mov");
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
    .arg(format!("color=c=0x202020:s={WIDTH}x{HEIGHT}:r=10:d=2"))
    .args(["-vf", "drawbox=x=160:y=0:w=160:h=180:color=0xf0f0f0:t=fill"])
    .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "8"])
    .arg(&path)
    .status()
    .unwrap()
    .success());
  path
}

fn clip(annotation: Annotation) -> RecordingAnnotationClip {
  RecordingAnnotationClip {
    path_ms: None,
    pin: None,
    annotation,
    track_id: AnnotationTrack::Primary,
    start_ms: 0,
    end_ms: 2_000,
  }
}

/// A clear loupe 80 across at (80, 90), enlarging twice the zoom area on the
/// bright half at (240, 90).
fn loupe() -> Annotation {
  let mut annotation = new_magnify("magnify".to_owned(), point(240.0, 90.0), None);
  annotation.shape = AnnotationShape::Magnify {
    start: point(220.0, 70.0),
    end: point(260.0, 110.0),
    loupe: point(80.0, 90.0),
    size: 80.0,
  };
  // Every exported clip carries a usable width, so the rim is a fine one; it
  // stays well clear of the middle the tests read.
  annotation.style.width = 2.0;
  annotation.style.shadow = false;
  annotation
}

/// `source` exported one pixel to one, with `clips` over it.
fn exported(source: &Path, destination: &Path, clips: &[RecordingAnnotationClip]) -> PathBuf {
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 1,
      keyboard_deletions: Box::default(),
      scene_clips: Vec::new(),
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
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination,
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
  destination.to_path_buf()
}

/// The mean green level over `[x0, y0, x1, y1)` of the frame one second into
/// the movie at `path`.
fn level(path: &Path, [x0, y0, x1, y1]: [u32; 4]) -> f64 {
  let output = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", "1.0", "-i"])
    .arg(path)
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
  assert!(output.status.success());
  let image = output.stdout;
  let mut total = 0.0;
  for y in y0..y1 {
    for x in x0..x1 {
      total += f64::from(image[((y * WIDTH + x) * 3 + 1) as usize]);
    }
  }
  total / f64::from((x1 - x0) * (y1 - y0))
}

#[test]
fn an_exported_loupe_shows_the_frame_under_its_zoom_area_and_not_what_is_redacted() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-magnify-video-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let source = recording(&directory);
  let inside = [64, 74, 96, 106];
  // Over the dark half, the loupe shows the bright half its zoom area covers.
  let magnified = exported(&source, &directory.join("magnified.mp4"), &[clip(loupe())]);
  assert!(level(&magnified, inside) > 200.0);
  // A redaction over the zoom area is what the loupe enlarges, not what it
  // hides.
  let mut cover = new_redact("redact".to_owned(), point(200.0, 50.0), None);
  cover.shape = AnnotationShape::Redact {
    start: point(200.0, 50.0),
    end: point(280.0, 130.0),
    seed: 1,
  };
  cover.style.redaction = AnnotationRedaction::Color;
  cover.style.color = "#000000".to_owned();
  let redacted = exported(
    &source,
    &directory.join("redacted.mp4"),
    &[clip(cover), clip(loupe())],
  );
  assert!(level(&redacted, inside) < 40.0);
  std::fs::remove_dir_all(directory).unwrap();
}
