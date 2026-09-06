// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn appearance(timestamp_us: u64, style: CursorStyle) -> Appearance {
  Appearance {
    height: 32.0,
    hotspot_x: 1.0,
    hotspot_y: 1.0,
    style,
    timestamp_us,
    width: 24.0,
  }
}

#[test]
fn ignores_brief_style_changes_without_delaying_a_stable_one() {
  let appearances = [
    appearance(0, CursorStyle::Arrow),
    appearance(100_000, CursorStyle::IBeam),
    appearance(320_000, CursorStyle::Arrow),
    appearance(500_000, CursorStyle::IBeam),
    appearance(900_000, CursorStyle::Arrow),
  ];
  let stable = stable_appearances(&appearances, 1_000_000);
  assert_eq!(stable.len(), 2);
  assert_eq!(stable[0].style, CursorStyle::Arrow);
  assert_eq!(stable[1].style, CursorStyle::IBeam);
  assert_eq!(stable[1].timestamp_us, 500_000);
}

#[test]
fn motion_blur_samples_never_leave_visible_gaps() {
  for distance in [2.0, 12.0, 40.0, MAX_BLUR_DISTANCE] {
    let samples = motion_blur_sample_count(distance);
    let spacing = distance / (samples - 1) as f64;
    assert!(spacing <= 2.0, "{distance}px blur left {spacing}px gaps");
  }
}

#[test]
fn custom_cursor_drops_its_recorded_hotspot() {
  let custom = Appearance {
    hotspot_x: 13.0,
    hotspot_y: 13.0,
    ..appearance(0, CursorStyle::Custom)
  };
  // The stand-in arrow carries its own hotspot as its design origin, so the
  // recorded one must not displace it (`custom_gpu_artwork`, raster.rs).
  assert_eq!(output_hotspot(custom), (0.0, 0.0));
}

#[test]
fn custom_arrow_fallback_keeps_the_recorded_arrow_size() {
  let records = [
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "macos".to_owned(),
      source: CursorSource {
        height: 166.0,
        kind: crate::recording::cursor::CursorSourceKind::Region,
        platform_id: "1".to_owned(),
        video_height: 332,
        video_width: 1_008,
        width: 504.0,
        x: 379.0,
        y: 446.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: crate::recording::cursor::FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      height: 40.0,
      hotspot_x: 5.0,
      hotspot_y: 5.0,
      style: CursorStyle::Arrow,
      timestamp_us: 0,
      width: 28.0,
    },
    CursorRecord::Position {
      timestamp_us: 0,
      x: 500.0,
      y: 500.0,
    },
    CursorRecord::Appearance {
      height: 18.0,
      hotspot_x: 4.0,
      hotspot_y: 9.0,
      style: CursorStyle::Custom,
      timestamp_us: 1_000_000,
      width: 9.0,
    },
    CursorRecord::Position {
      timestamp_us: 2_000_000,
      x: 500.0,
      y: 500.0,
    },
  ];
  let compositor = CursorCompositor::from_records(&records).unwrap();
  let cursor = compositor
    .gpu_cursor(
      1_500,
      (1_008, 332),
      CursorEffectSettings {
        size_percent: 400.0,
        ..CursorEffectSettings::default()
      },
    )
    .unwrap();

  assert_eq!(cursor.width, 56.0);
  assert_eq!(cursor.height, 80.0);
  assert_eq!(cursor.scale, 4.0);
}

#[test]
fn gpu_cursor_position_is_rounded_to_the_output_pixel_grid() {
  let records = [
    CursorRecord::Header {
      coordinate_space: "global-logical-points".to_owned(),
      platform: "test".to_owned(),
      source: CursorSource {
        height: 100.0,
        kind: crate::recording::cursor::CursorSourceKind::Region,
        platform_id: "1".to_owned(),
        video_height: 100,
        video_width: 100,
        width: 100.0,
        x: 0.0,
        y: 0.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: crate::recording::cursor::FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      height: 32.0,
      hotspot_x: 1.0,
      hotspot_y: 1.0,
      style: CursorStyle::Arrow,
      timestamp_us: 0,
      width: 24.0,
    },
    CursorRecord::Position {
      timestamp_us: 0,
      x: 10.4,
      y: 10.6,
    },
  ];
  let compositor = CursorCompositor::from_records(&records).unwrap();
  let cursor = compositor
    .gpu_cursor(0, (100, 100), CursorEffectSettings::default())
    .unwrap();

  assert_eq!(cursor.x, 10.0);
  assert_eq!(cursor.y, 11.0);
}

#[cfg(target_os = "windows")]
#[test]
fn windows_standard_cursors_keep_their_recorded_native_hotspots() {
  let assert_hotspot = |actual: (f64, f64), expected: (f64, f64)| {
    assert!((actual.0 - expected.0).abs() < f64::EPSILON * 16.0);
    assert!((actual.1 - expected.1).abs() < f64::EPSILON * 16.0);
  };
  let vector = |style| Appearance {
    height: 32.0,
    hotspot_x: 8.0,
    hotspot_y: 9.0,
    style,
    timestamp_us: 0,
    width: 32.0,
  };
  assert_hotspot(output_hotspot(vector(CursorStyle::IBeam)), (8.0, 9.0));
  assert_hotspot(
    output_hotspot(vector(CursorStyle::ResizeHorizontal)),
    (8.0, 9.0),
  );
  assert_hotspot(
    output_hotspot(vector(CursorStyle::PointingHand)),
    (8.0, 9.0),
  );
}

#[cfg(target_os = "windows")]
#[test]
fn windows_cursor_position_has_no_macos_screen_reaction_delay() {
  assert_eq!(SCREEN_REACTION_US, 0);
}
