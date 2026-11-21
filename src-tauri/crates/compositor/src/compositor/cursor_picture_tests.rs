// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An app's own cursor, kept with its recording, drawn from its picture by
//! a compositor made before the recording was opened, as the editor's is.

use image::{Rgba, RgbaImage};
use screenwide_cursor_effects::{CursorCompositor, CursorEffectSettings};
use screenwide_output::fixtures::test_output_settings;
use screenwide_records::cursor::{
  picture_path, CursorRecord, CursorSource, CursorSourceKind, CursorStyle, FORMAT_VERSION,
};

use super::render_test_helpers::{compositor, read_pixel, target, view, white_source, OUTPUT};
use super::*;

/// A recording whose pointer is an app's own cursor, its picture solid
/// `colour`, resting at the middle of a 320 by 180 point screen.
fn recording(colour: [u8; 4]) -> std::path::PathBuf {
  let folder = std::env::temp_dir().join(format!(
    "screenwide-compositor-cursor-picture-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  let sidecar = folder.join("cursor.jsonl");
  let records = [
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "test".to_owned(),
      source: CursorSource {
        height: 180.0,
        kind: CursorSourceKind::Screen,
        platform_id: "test".to_owned(),
        video_height: OUTPUT.1,
        video_width: OUTPUT.0,
        width: 320.0,
        x: 0.0,
        y: 0.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      height: 20.0,
      hotspot_x: 0.0,
      hotspot_y: 0.0,
      style: CursorStyle::Custom,
      timestamp_us: 0,
      width: 30.0,
      picture: Some("5eed".to_owned()),
    },
    CursorRecord::Position {
      timestamp_us: 0,
      x: 160.0,
      y: 90.0,
    },
  ];
  let lines: Vec<String> = records
    .iter()
    .map(|record| serde_json::to_string(record).unwrap())
    .collect();
  std::fs::write(&sidecar, lines.join("\n")).unwrap();
  let picture = picture_path(&sidecar, "5eed").unwrap();
  std::fs::create_dir_all(picture.parent().unwrap()).unwrap();
  RgbaImage::from_pixel(12, 8, Rgba(colour))
    .save(&picture)
    .unwrap();
  sidecar
}

#[test]
fn a_compositor_draws_an_app_s_cursor_opened_after_it() {
  let compositor = compositor();
  let cursor = CursorCompositor::open(&recording([20, 180, 60, 255]), None).unwrap();
  let settings = test_output_settings(OUTPUT.0, OUTPUT.1);
  let drawn = cursor.gpu_cursor(
    0,
    (f64::from(OUTPUT.0), f64::from(OUTPUT.1)),
    CursorEffectSettings::default(),
  );
  let output = target();
  compositor
    .draw_with_camera(
      &view(&output),
      &white_source(&compositor, OUTPUT),
      &settings,
      ComposedFrame {
        cursor: drawn,
        keyboard: None,
        foreground_only: false,
        seconds: 0.0,
      },
      None,
      None,
      &Default::default(),
    )
    .expect("the canvas draws");
  let drawn = drawn.unwrap();
  // Inside the picture, a little past the hotspot at its top-left corner,
  // BGRA; the GPU's colour conversion may round a channel by one.
  let pixel = read_pixel(&output, drawn.x as u32 + 20, drawn.y as u32 + 20);
  let expected = [60, 180, 20, 255];
  assert!(
    pixel
      .iter()
      .zip(expected)
      .all(|(channel, wanted)| channel.abs_diff(wanted) <= 1),
    "the cursor was drawn {pixel:?}, not its picture's {expected:?}"
  );
}
