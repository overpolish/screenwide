// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An app's own cursor, kept with its recording, drawn from its picture.

use std::path::PathBuf;

use image::{Rgba, RgbaImage};
use screenwide_records::cursor::{picture_path, CursorSourceKind, FORMAT_VERSION};

use super::*;

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

fn custom(timestamp_us: u64, picture: Option<&str>) -> CursorRecord {
  CursorRecord::Appearance {
    height: 20.0,
    hotspot_x: 5.0,
    hotspot_y: 15.0,
    style: CursorStyle::Custom,
    timestamp_us,
    width: 30.0,
    picture: picture.map(str::to_owned),
  }
}

/// A sidecar of `records` with the picture `name` of `colour` beside it.
fn recording(test: &str, records: &[CursorRecord], name: &str, colour: [u8; 4]) -> PathBuf {
  let folder = std::env::temp_dir().join(format!(
    "screenwide-cursor-picture-{test}-{}",
    std::process::id()
  ));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  let sidecar = folder.join("cursor.jsonl");
  let lines: Vec<String> = records
    .iter()
    .map(|record| serde_json::to_string(record).unwrap())
    .collect();
  std::fs::write(&sidecar, lines.join("\n")).unwrap();
  let picture = picture_path(&sidecar, name).unwrap();
  std::fs::create_dir_all(picture.parent().unwrap()).unwrap();
  RgbaImage::from_pixel(12, 8, Rgba(colour))
    .save(&picture)
    .unwrap();
  sidecar
}

fn position(timestamp_us: u64) -> CursorRecord {
  CursorRecord::Position {
    timestamp_us,
    x: 100.0,
    y: 80.0,
  }
}

#[test]
fn an_app_s_own_cursor_is_drawn_from_its_picture_about_its_own_hotspot() {
  let records = [header(), custom(0, Some("c0ffee")), position(0)];
  let sidecar = recording("own", &records, "c0ffee", [10, 200, 30, 255]);
  let cursor = CursorCompositor::open(&sidecar, None).unwrap();
  let drawn = cursor
    .gpu_cursor(0, (320.0, 180.0), CursorEffectSettings::default())
    .unwrap();
  assert!(drawn.style > raster::GPU_CUSTOM_ARTWORK_INDEX);
  let artwork = &gpu_artworks()[drawn.style as usize];
  assert_eq!((artwork.width, artwork.height), (12, 8));
  assert_eq!(&artwork.pixels[..4], &[10, 200, 30, 255]);
  // Its own box and hotspot, not the arrow's that stands in for a custom
  // cursor with no picture.
  assert_eq!((drawn.width, drawn.height), (30.0, 20.0));
  assert_eq!((drawn.hotspot_x, drawn.hotspot_y), (5.0, 15.0));
}

#[test]
fn two_of_an_app_s_cursors_each_show_their_own_picture() {
  let records = [
    header(),
    custom(0, Some("aa01")),
    position(0),
    custom(1_000_000, Some("bb02")),
    position(2_000_000),
  ];
  let sidecar = recording("two", &records, "aa01", [255, 0, 0, 255]);
  RgbaImage::from_pixel(12, 8, Rgba([0, 0, 255, 255]))
    .save(picture_path(&sidecar, "bb02").unwrap())
    .unwrap();
  let cursor = CursorCompositor::open(&sidecar, None).unwrap();
  let style_at = |ms| {
    cursor
      .gpu_cursor(ms, (320.0, 180.0), CursorEffectSettings::default())
      .unwrap()
      .style
  };
  let (first, second) = (style_at(500), style_at(1_500));
  assert_ne!(first, second);
  assert_eq!(
    &gpu_artworks()[second as usize].pixels[..4],
    &[0, 0, 255, 255]
  );
}

#[test]
fn a_custom_cursor_without_its_picture_is_drawn_as_the_arrow() {
  let records = [header(), custom(0, Some("dead01")), position(0)];
  let sidecar = recording("missing", &records, "beef02", [0, 0, 0, 255]);
  let cursor = CursorCompositor::open(&sidecar, None).unwrap();
  let drawn = cursor
    .gpu_cursor(0, (320.0, 180.0), CursorEffectSettings::default())
    .unwrap();
  assert_eq!(drawn.style, raster::GPU_CUSTOM_ARTWORK_INDEX);
}
