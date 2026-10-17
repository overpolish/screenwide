// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_text::captions::{Caption, CaptionTrack, CaptionWord, RecordingCaptions};
use screenwide_text::TextTemplateId;
use std::process::Command;

fn word(text: &str, start_ms: f64) -> CaptionWord {
  CaptionWord {
    text: text.to_owned(),
    said: text.to_owned(),
    start_ms,
    end_ms: start_ms + 400.0,
    source_start_ms: start_ms,
    source_end_ms: start_ms + 400.0,
    emphasis: false,
  }
}

/// How many bright pixels the bottom quarter of the frame `at_seconds` into
/// `movie` holds, 640 by 360.
fn bright_bottom(movie: &std::path::Path, at_seconds: &str) -> usize {
  let frame = Command::new(screenwide_sidecar::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-ss",
      at_seconds,
      "-i",
    ])
    .arg(movie)
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
  assert_eq!(frame.stdout.len(), 640 * 360 * 3);
  frame
    .stdout
    .as_chunks::<3>()
    .0
    .iter()
    .enumerate()
    .filter(|(index, pixel)| index / 640 > 270 && pixel.iter().all(|channel| *channel > 200))
    .count()
}

#[test]
fn exports_captions_into_the_picture_while_they_show() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-caption-export-test-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let destination = directory.join("output.mp4");
  let status = Command::new(screenwide_sidecar::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "color=c=black:s=640x360:r=30:d=2",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
    ])
    .arg(&source)
    .status()
    .unwrap();
  assert!(status.success());
  let track = CaptionTrack {
    captions: vec![Caption {
      id: 800,
      start_ms: 800.0,
      end_ms: 1_600.0,
      text: "HELLO THERE".to_owned(),
      lines: vec![vec![word("HELLO", 800.0), word("THERE", 1_200.0)]],
      spaced: true,
    }],
    recording: RecordingCaptions {
      enabled: true,
      style: TextTemplateId::from("clean").shipped_style(),
      ..Default::default()
    },
    editing: None,
  };
  let mut output = screenwide_output::fixtures::test_output_settings(640, 360);
  output.background_color = "#000000".to_owned();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    camera_effects: None,
    annotation_track: screenwide_annotations::timing::AnnotationTrack::Primary,
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    captions: Some(&track),
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    destination: &destination,
    duration_ms: 2_000,
    height: 360,
    keyboard: None,
    keyboard_effects: screenwide_keyboard_effects::KeyboardEffectSettings::default(),
    on_progress: &mut |_| {},
    output: &output,
    screen: &source,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: 640,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);
  assert_eq!(bright_bottom(&destination, "0.4"), 0);
  let shown = bright_bottom(&destination, "1.2");
  assert!(shown > 100, "the caption's type should show ({shown})");
  assert_eq!(bright_bottom(&destination, "1.8"), 0);
  let _ = std::fs::remove_dir_all(directory);
}
