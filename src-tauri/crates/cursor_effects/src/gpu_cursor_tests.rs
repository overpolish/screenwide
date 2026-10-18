// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The cursor a frame is drawn with: its blur, its click and its artwork.

use super::*;
use screenwide_records::cursor::{CursorSourceKind, FORMAT_VERSION};

fn header() -> CursorRecord {
  CursorRecord::Header {
    coordinate_space: "global-logical-points".to_owned(),
    platform: "test".to_owned(),
    source: CursorSource {
      height: 180.0,
      kind: CursorSourceKind::Screen,
      platform_id: "test".to_owned(),
      video_height: 180,
      video_width: 320,
      width: 320.0,
      x: 0.0,
      y: 0.0,
    },
    timebase: "recording-microseconds".to_owned(),
    version: FORMAT_VERSION,
  }
}

fn appearance(timestamp_us: u64, style: CursorStyle) -> CursorRecord {
  CursorRecord::Appearance {
    height: 24.0,
    hotspot_x: 1.0,
    hotspot_y: 1.0,
    style,
    timestamp_us,
    width: 16.0,
  }
}

fn position(timestamp_us: u64, x: f64) -> CursorRecord {
  CursorRecord::Position {
    timestamp_us,
    x,
    y: 80.0,
  }
}

#[test]
fn a_moving_cursor_carries_its_frame_travel_as_its_blur() {
  // Positions must stay inside one motion segment, so the sampled travel
  // has to be denser than POSITION_SEGMENT_GAP_US.
  let mut records = vec![header(), appearance(0, CursorStyle::Arrow)];
  records.extend((0..=50).map(|step| position(step * 20_000, 80.0 + f64::from(step as u32) * 3.2)));
  let cursor = CursorCompositor::from_records(&records, &[]).unwrap();
  let settings = CursorEffectSettings::default();
  let moving = cursor.gpu_cursor(500, (320, 180), settings).unwrap();
  assert!(moving.blur_delta_y.abs() < 0.001);
  // 160 output pixels per second at 60 Hz is a fraction under three pixels
  // of travel per frame.
  assert!(
    (moving.blur_delta_x - 160.0 / 60.0).abs() < 0.5,
    "unexpected frame travel {}",
    moving.blur_delta_x
  );
  let still = CursorEffectSettings {
    motion_blur: false,
    ..settings
  };
  let unblurred = cursor.gpu_cursor(500, (320, 180), still).unwrap();
  assert_eq!((unblurred.blur_delta_x, unblurred.blur_delta_y), (0.0, 0.0));
}

#[test]
fn a_held_button_shrinks_the_cursor_only_with_the_click_animation() {
  let button = |state, timestamp_us| CursorRecord::Button {
    button: CursorButton::Left,
    click_count: 1,
    state,
    timestamp_us,
    x: 80.0,
    y: 80.0,
  };
  let records = [
    header(),
    appearance(0, CursorStyle::Arrow),
    position(0, 80.0),
    button(ButtonState::Down, 500_000),
    button(ButtonState::Up, 700_000),
    position(2_000_000, 80.0),
  ];
  let cursor = CursorCompositor::from_records(&records, &[]).unwrap();
  let settings = CursorEffectSettings::default();
  let resting = cursor.gpu_cursor(100, (320, 180), settings).unwrap();
  let clicked = cursor.gpu_cursor(600, (320, 180), settings).unwrap();
  assert!(clicked.scale < resting.scale);
  let without_animation = CursorEffectSettings {
    click_animation: false,
    ..settings
  };
  let unanimated = cursor
    .gpu_cursor(600, (320, 180), without_animation)
    .unwrap();
  assert_eq!(unanimated.scale, resting.scale);
}

#[test]
fn every_style_indexes_an_artwork_that_fills_its_bitmap() {
  let artworks = gpu_artworks();
  for style in raster::GPU_ARTWORK_STYLES
    .into_iter()
    .chain([CursorStyle::Custom])
  {
    let records = [
      header(),
      appearance(0, style),
      position(0, 80.0),
      position(1_000_000, 80.0),
    ];
    let cursor = CursorCompositor::from_records(&records, &[]).unwrap();
    let evaluated = cursor
      .gpu_cursor(100, (320, 180), CursorEffectSettings::default())
      .unwrap();
    let artwork = &artworks[evaluated.style as usize];
    assert!(artwork.width > 0 && artwork.height > 0, "{style:?}");
    assert_eq!(
      artwork.pixels.len(),
      artwork.width as usize * artwork.height as usize * 4
    );
  }
}
