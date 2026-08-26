// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::exports::keyboard_effects::KeyboardEffectSettings;
use std::process::Command;

fn keyboard_output(width: u32, height: u32) -> crate::screenshots::ScreenshotOutputSettings {
  let mut output = crate::screenshots::test_output_settings(width, height);
  output.background_color = "#000000".to_owned();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  output.mesh_seed = 0;
  output.mesh_warp_percent = 0.0;
  output.screenshot_crop_height_percent = 100.0;
  output.screenshot_crop_width_percent = 100.0;
  output.screenshot_crop_x_percent = 0.0;
  output.screenshot_crop_y_percent = 0.0;
  output.screenshot_image_width_percent = 100.0;
  output
}

#[test]
fn keeps_a_five_hundred_percent_keyboard_edge_sharp_at_4k() {
  use crate::exports::keyboard_effects::KeyboardOverlay;

  let (width, height) = (3840_u32, 2160_u32);
  let source = crate::screenshots::CapturedImage {
    rgba: [0, 0, 0, 255].repeat((width * height) as usize),
    width,
    height,
  };
  let mut keyboard = KeyboardOverlay {
    key_count: 1,
    animation: KeyboardOverlay::ANIMATION_POP,
    appearance: KeyboardOverlay::APPEARANCE_LIGHT,
    scale: 5.0,
    progress: 1.0,
    ..Default::default()
  };
  keyboard.keys[0].key_code = 55;
  keyboard.keys[0].visible = 1;
  keyboard.keys[0].progress = 1.0;
  keyboard.keys[0].alpha = 1.0;
  keyboard.keys[0].scale = 5.0;
  keyboard.keys[0].layout_progress = 1.0;
  keyboard.keys[0].layout_from_mask = 1;
  keyboard.keys[0].layout_to_mask = 1;
  let composed = crate::screenshots::compose_output_layers(
    &source,
    &keyboard_output(width, height),
    0.0,
    false,
    None,
    None,
    None,
    Some(&keyboard),
    false,
    false,
  )
  .unwrap();
  let x = width as usize / 2;
  let luminance = (0..height as usize)
    .map(|y| composed.rgba[(y * width as usize + x) * 4])
    .collect::<Vec<_>>();
  let strongest_edge = luminance
    .windows(2)
    .map(|pair| u8::abs_diff(pair[0], pair[1]))
    .max()
    .unwrap_or_default();
  assert!(
    strongest_edge >= 160,
    "the 4K keyboard edge remained soft ({strongest_edge}/255 contrast per pixel)"
  );
}

#[test]
fn exports_keyboard_shortcuts_into_a_real_movie() {
  let directory = std::env::temp_dir().join(format!(
    "screenwide-keyboard-export-test-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&directory);
  std::fs::create_dir_all(&directory).unwrap();
  let source = directory.join("source.mov");
  let keyboard = directory.join("source.keyboard.jsonl");
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
  std::fs::write(
    &keyboard,
    concat!(
      "{\"type\":\"header\",\"platform\":\"macos\",\"timebase\":\"recording-microseconds\",\"version\":2}\n",
      "{\"type\":\"keyDown\",\"keyCode\":55,\"modifiers\":[\"command\"],\"timestampUs\":200000}\n",
      "{\"type\":\"keyDown\",\"keyCode\":0,\"modifiers\":[\"command\"],\"timestampUs\":250000}\n",
      "{\"type\":\"keyUp\",\"keyCode\":0,\"modifiers\":[\"command\"],\"timestampUs\":300000}\n",
      "{\"type\":\"keyUp\",\"keyCode\":55,\"modifiers\":[],\"timestampUs\":320000}\n"
    ),
  )
  .unwrap();
  let mut output = crate::screenshots::test_output_settings(640, 360);
  output.background_color = "#000000".to_owned();
  output.mesh_colors.clear();
  output.mesh_locked_colors.clear();
  output.mesh_points.clear();
  let cancelled = AtomicBool::new(false);
  let result = export(CursorExportRequest {
    audio_layout: AudioLayout::SeparateTracks,
    audio_source: None,
    camera: None,
    camera_on_top: true,
    cancelled: &cancelled,
    cursor: None,
    cursor_effects: CursorEffectSettings::default(),
    destination: &destination,
    duration_ms: 2_000,
    height: 360,
    keyboard: Some(&keyboard),
    keyboard_effects: KeyboardEffectSettings::default(),
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
  assert_eq!(frame.stdout.len(), 640 * 360 * 3);
  let bright_bottom = frame
    .stdout
    .chunks_exact(3)
    .enumerate()
    .filter(|(index, pixel)| index / 640 > 300 && pixel.iter().all(|channel| *channel > 170))
    .count();
  assert!(
    bright_bottom > 200,
    "the rendered frame should contain the light shortcut near the bottom"
  );
  let _ = std::fs::remove_dir_all(directory);
}
