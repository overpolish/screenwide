// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Redactions through the real video export: two recordings that differ
//! only under the box export to the same movie, whatever moves there, from
//! the clip's first frame to its last.

use super::*;
use crate::editor::annotations::redact::new_redact;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::{AnnotationPoint, AnnotationRedaction, AnnotationShape};
use crate::editor::timeline_edit::{RecordingTimelineEdit, RecordingTimelineSegment, TimelinePlan};
use std::path::{Path, PathBuf};
use std::process::Command;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
/// Where the secret moves: a pattern this size, placed here, on whole
/// sixteen-pixel blocks. The recordings are compressed, and a block that
/// held part of the secret would carry traces of it into grey the box does
/// not cover, which is the recording's own picture rather than a leak.
const SECRET: (u32, u32, u32, u32) = (112, 64, 64, 48);
/// The box, over the secret's blocks with room to spare, with odd corners
/// the export snaps out.
const BOX: [f64; 4] = [101.3, 53.6, 186.7, 122.2];

/// A grey recording two seconds long with `pattern`, an ffmpeg source, moving
/// under the box.
fn recording(directory: &Path, name: &str, pattern: &str) -> PathBuf {
  let path = directory.join(format!("{name}.mov"));
  let (x, y, width, height) = SECRET;
  let filter = format!(
    "[1:v]scale={width}:{height}[secret];[0:v][secret]overlay={x}:{y}:shortest=1,format=yuv420p"
  );
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
    .arg(format!("color=c=gray:s={WIDTH}x{HEIGHT}:r=10:d=2"))
    .args(["-f", "lavfi", "-i"])
    .arg(format!("{pattern}=s={width}x{height}:r=10:d=2"))
    .args([
      "-filter_complex",
      &filter,
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p"
    ])
    .arg(&path)
    .status()
    .unwrap()
    .success());
  path
}

/// `source` exported one pixel to one, a clip over the whole recording
/// holding `clip` if one is given. Without one there is no edit at all.
fn exported(source: &Path, destination: &Path, clip: Option<RecordingAnnotationClip>) -> PathBuf {
  let timeline = TimelinePlan::from_edit(
    &RecordingTimelineEdit {
      artifact_id: 1,
      next_segment_id: 1,
      keyboard_deletions: Box::default(),
      annotation_clips: clip.into_iter().collect(),
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

/// One frame of the movie at `path`, `time` seconds in, as RGB.
fn frame(path: &Path, time: &str) -> Vec<u8> {
  let output = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", time, "-i"])
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
  assert_eq!(output.stdout.len(), (WIDTH * HEIGHT * 3) as usize);
  output.stdout
}

/// How far apart two frames are at their furthest channel.
fn furthest(a: &[u8], b: &[u8]) -> u8 {
  a.iter()
    .zip(b)
    .map(|(a, b)| a.abs_diff(*b))
    .max()
    .unwrap_or(0)
}

fn redaction(mode: AnnotationRedaction) -> RecordingAnnotationClip {
  let point = |x, y| AnnotationPoint { x, y };
  let mut annotation = new_redact("redaction".to_owned(), point(BOX[0], BOX[1]), None);
  if let AnnotationShape::Redact { end, .. } = &mut annotation.shape {
    *end = point(BOX[2], BOX[3]);
  }
  annotation.style.redaction = mode;
  annotation.style.color = "#123456".to_owned();
  RecordingAnnotationClip {
    annotation,
    track_id: AnnotationTrack::Primary,
    start_ms: 0,
    end_ms: 2_000,
  }
}

#[test]
fn an_exported_redaction_carries_nothing_of_what_moves_under_it() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-redact-video-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let first = recording(&directory, "first", "testsrc");
  let second = recording(&directory, "second", "testsrc2");
  // Unredacted, the two movies differ where the secret is.
  let plain = [
    exported(&first, &directory.join("first-plain.mp4"), None),
    exported(&second, &directory.join("second-plain.mp4"), None),
  ];
  assert!(furthest(&frame(&plain[0], "1.0"), &frame(&plain[1], "1.0")) > 60);
  for mode in [AnnotationRedaction::Erase, AnnotationRedaction::Color] {
    let movies = [
      exported(&first, &directory.join("first.mp4"), Some(redaction(mode))),
      exported(
        &second,
        &directory.join("second.mp4"),
        Some(redaction(mode)),
      ),
    ];
    for time in ["0.0", "0.55", "1.9"] {
      let apart = furthest(&frame(&movies[0], time), &frame(&movies[1], time));
      // The encoder's own rounding is all that may differ.
      assert!(
        apart <= 3,
        "{mode:?} at {time}s shows what it covers: {apart}"
      );
    }
  }
  std::fs::remove_dir_all(directory).unwrap();
}

/// A recording whose background is blue for its first second and white for
/// its second, with nothing else in it.
fn changing_background(directory: &Path) -> PathBuf {
  let path = directory.join("changing.mov");
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
    .arg(format!("color=c=0x1e46c8:s={WIDTH}x{HEIGHT}:r=10:d=1"))
    .args(["-f", "lavfi", "-i"])
    .arg(format!("color=c=white:s={WIDTH}x{HEIGHT}:r=10:d=1"))
    .args([
      "-filter_complex",
      "[0:v][1:v]concat=n=2:v=1,format=yuv420p",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
    ])
    .arg(&path)
    .status()
    .unwrap()
    .success());
  path
}

#[test]
fn an_exported_erase_follows_the_background_it_sits_on() {
  let directory =
    std::env::temp_dir().join(format!("screenwide-redact-surface-{}", std::process::id()));
  std::fs::create_dir_all(&directory).unwrap();
  let source = changing_background(&directory);
  let movie = exported(
    &source,
    &directory.join("erased.mp4"),
    Some(redaction(AnnotationRedaction::Erase)),
  );
  let middle =
    (((BOX[1] + BOX[3]) / 2.0) as u32 * WIDTH + ((BOX[0] + BOX[2]) / 2.0) as u32) as usize * 3;
  let at = |time: &str| {
    let rgb = frame(&movie, time);
    [rgb[middle], rgb[middle + 1], rgb[middle + 2]]
  };
  let (blue, white) = (at("0.5"), at("1.6"));
  assert!(blue[2] > 150 && blue[0] < 80, "{blue:?}");
  assert!(white.iter().all(|channel| *channel > 220), "{white:?}");
  std::fs::remove_dir_all(directory).unwrap();
}
