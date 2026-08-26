// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::process::Command;

use super::*;

#[test]
fn exports_recenter_inset_pixels_outside_the_source_crop() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-recenter-inset-export-test-{}",
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
      "color=c=black:s=320x180:r=30:d=1",
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
  settings.recenter_inset_color = Some("#00ff00".to_owned());
  settings.screenshot_crop_height_percent = 100.0;
  settings.screenshot_crop_width_percent = 100.0;
  settings.screenshot_crop_x_percent = 0.0;
  settings.screenshot_crop_y_percent = 0.0;
  settings.screenshot_image_width_percent = 100.0;
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
    keyboard_effects: crate::exports::keyboard_effects::KeyboardEffectSettings::default(),
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
  assert!(
    inset[1] > 200 && inset[0] < 80 && inset[2] < 80,
    "the inset pixel was {inset:?} instead of green"
  );
  let content = pixel(160, 90);
  assert!(
    content.iter().all(|channel| *channel < 40),
    "the source content pixel was {content:?} instead of black"
  );
  let _ = std::fs::remove_dir_all(directory);
}
