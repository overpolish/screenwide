// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::process::Command;

use super::platform;
use crate::editor::{
  cursor_effects::CursorEffectSettings,
  cursor_export::{export, CursorExportRequest},
  media_preview::{self, ExportRunResult, VideoExportOptions},
  track_selection::{AudioLayout, TrackSelection},
};
use std::sync::atomic::AtomicBool;

#[test]
fn recenter_detected_dark_background_matches_native_composition() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-recenter-colour-test-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let destination = directory.join("output.mp4");
  let status = Command::new(media_preview::ffmpeg_path())
    .args([
      "-hide_banner",
      "-loglevel",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      "color=c=0x222222:s=320x180:r=30:d=1",
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
    ])
    .arg(&source)
    .status()
    .unwrap();
  assert!(status.success());

  let mut settings = crate::screenshots::test_output_settings(320, 180);
  settings.background_color = "#000000".to_owned();
  settings.drop_shadow = false;
  let decoded = platform::source_frame_image(&source, 500, 1000).unwrap();
  let analysis = crate::editor::commands::recenter::analyse(
    &decoded.rgba,
    decoded.width,
    decoded.height,
    crate::screenshots::NormalizedSourceRect {
      x: 0.0,
      y: 0.0,
      width: 1.0,
      height: 1.0,
    },
    24,
  )
  .unwrap();
  let detected: serde_json::Value = serde_json::to_value(analysis).unwrap();
  settings.recenter_inset_color = Some(detected["backgroundColor"].as_str().unwrap().to_owned());
  settings.crop_height = 180.0;
  settings.crop_width = 320.0;
  settings.crop_x = 0.0;
  settings.crop_y = 0.0;
  settings.image_width = 320.0;
  settings.image_x = 0.0;
  settings.image_y = 0.0;
  settings.source_crop = crate::screenshots::NormalizedSourceRect {
    height: 0.5,
    width: 0.5,
    x: 0.25,
    y: 0.25,
  };
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    keyboard: None,
    keyboard_effects: crate::editor::keyboard_effects::KeyboardEffectSettings::default(),
    destination: &destination,
    duration_ms: 1_000,
    height: 180,
    on_progress: &mut |_| {},
    output: &settings,
    screen: &source,
    selection: &TrackSelection::default(),
    timeline: None,
    video: VideoExportOptions {
      compression: 1,
      resolution_scale_percent: 100,
      source_scale_percent: 100,
    },
    width: 320,
  })
  .unwrap();
  assert_eq!(result, ExportRunResult::Completed);

  let frame = Command::new(media_preview::ffmpeg_path())
    .args(["-hide_banner", "-loglevel", "error", "-ss", "0.5", "-i"])
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
  let pixel = |x: usize, y: usize| &frame.stdout[(y * 320 + x) * 3..][..3];
  let inset = pixel(20, 20);
  let content = pixel(160, 90);
  assert!(
    inset.iter().zip(content).all(|(a, b)| a.abs_diff(*b) <= 2),
    "detected {:?}, inset {inset:?}, source {content:?}",
    settings.recenter_inset_color
  );
  let _ = std::fs::remove_dir_all(directory);
}
