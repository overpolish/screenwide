// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn packet(x: f64, y: f64, width: f64, height: f64, kind: u8) -> OcrRectPacket {
  OcrRectPacket {
    x,
    y,
    width,
    height,
    kind,
    padding: [0; 7],
  }
}

const BOUNDS: Size = Size {
  width: 1920.0,
  height: 1080.0,
};

#[test]
fn highlight_kinds_follow_the_shared_visual_kinds() {
  assert_eq!(rect_kind(1), 17);
  assert_eq!(rect_kind(2), 18);
  assert_eq!(rect_kind(3), 19);
  assert_eq!(rect_kind(4), 20);
  // An unknown kind falls back to a text line, as the macOS ladder did.
  assert_eq!(rect_kind(9), 17);
}

#[test]
fn rects_are_translated_into_the_surface_and_filtered_to_it() {
  let packets = [
    packet(2000.0, 100.0, 200.0, 40.0, 1),
    packet(1000.0, 100.0, 200.0, 40.0, 2),
    // Entirely on the display to the left of this one.
    packet(100.0, 100.0, 200.0, 40.0, 1),
  ];
  let offset = Point { x: 1920.0, y: 0.0 };

  let kept = local_rects(&packets, offset, BOUNDS);
  assert_eq!(kept.len(), 1);
  assert_eq!(kept[0].rect, Rect::from_xywh(80.0, 100.0, 200.0, 40.0));
  assert_eq!(kept[0].kind, 1);

  // Without an offset the first display keeps the other two.
  let anchor = local_rects(&packets, Point::default(), BOUNDS);
  assert_eq!(anchor.len(), 2);
  assert_eq!(anchor[0].kind, 2);
}

#[test]
fn a_zero_area_rect_never_lands_on_a_surface() {
  let packets = [packet(10.0, 10.0, 0.0, 40.0, 1)];
  assert!(local_rects(&packets, Point::default(), BOUNDS).is_empty());
}

#[test]
fn the_sheen_runs_on_over_the_result_until_its_crossing_ends() {
  let view = Size {
    width: 800.0,
    height: 600.0,
  };
  let region = Rect::from_xywh(100.0, 100.0, 200.0, 100.0);
  let kinds = |chrome: &Chrome, after_ms: u64| {
    let since = chrome.sheen.map_or_else(Instant::now, |sheen| sheen.since);
    let mut out = Vec::new();
    let now = since + std::time::Duration::from_millis(after_ms);
    chrome.add_world_vertices(&mut out, view, region, 1.0, now);
    out.chunks(6).map(|quad| quad[0].kind).collect::<Vec<_>>()
  };
  let mut chrome = Chrome::default();
  chrome.apply(PHASE_LOADING, &[], Point::default(), view);
  assert!(chrome.is_animating());
  assert_eq!(kinds(&chrome, 500), vec![18, 18, 18, 18, 69]);

  // Read at once: the first crossing carries on over the highlights.
  chrome.apply(
    PHASE_READY,
    &[packet(120.0, 120.0, 40.0, 12.0, 1)],
    Point::default(),
    view,
  );
  assert!(chrome.is_animating());
  assert_eq!(kinds(&chrome, 500), vec![18, 18, 18, 18, 17, 69]);
  assert_eq!(kinds(&chrome, 1200), vec![18, 18, 18, 18, 17]);

  // The idle phase keeps the highlights out of the frame entirely.
  chrome.apply(0, &[], Point::default(), view);
  assert!(!chrome.is_animating());
  assert!(kinds(&chrome, 500).is_empty());
}
