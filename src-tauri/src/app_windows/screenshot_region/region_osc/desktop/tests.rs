// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn probe(id: u32, x: f64, y: f64, width: f64, height: f64, scale: f64) -> MonitorProbe {
  MonitorProbe {
    id,
    origin: Point { x, y },
    size: Size { width, height },
    scale,
  }
}

#[test]
fn one_monitor_becomes_its_own_logical_desktop() {
  let binding = build_binding(&[probe(7, 0.0, 0.0, 1920.0, 1080.0, 1.0)], 7, None).unwrap();

  assert_eq!(binding.anchor_id, 7);
  assert_eq!(binding.displays.len(), 1);
  assert_eq!(binding.displays[0].origin, Point { x: 0.0, y: 0.0 });
  assert_eq!(
    binding.size,
    Size {
      width: 1920.0,
      height: 1080.0
    }
  );
}

#[test]
fn a_scaled_monitor_reports_its_logical_extent() {
  let binding = build_binding(&[probe(1, 0.0, 0.0, 2880.0, 1620.0, 1.5)], 1, None).unwrap();

  assert_eq!(
    binding.size,
    Size {
      width: 1920.0,
      height: 1080.0
    }
  );
  assert_eq!(binding.displays[0].scale, 1.5);
  assert_eq!(binding.virtual_monitor().size, binding.size);
}

#[test]
fn a_monitor_left_of_the_primary_normalizes_the_union_to_the_origin() {
  let binding = build_binding(
    &[
      probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
      probe(2, -1280.0, -100.0, 1280.0, 1024.0, 1.0),
    ],
    1,
    None,
  )
  .unwrap();

  // The left monitor lands at the origin and the primary shifts right.
  assert_eq!(binding.displays[1].origin, Point { x: 0.0, y: 0.0 });
  assert_eq!(
    binding.displays[0].origin,
    Point {
      x: 1280.0,
      y: 100.0
    }
  );
  assert_eq!(
    binding.size,
    Size {
      width: 3200.0,
      height: 1180.0
    }
  );
  // A region local to the anchor projects past the seam without clamping.
  assert_eq!(
    binding.project_local(Rect::from_xywh(-200.0, 0.0, 400.0, 300.0)),
    Some(Rect::from_xywh(1080.0, 100.0, 400.0, 300.0))
  );
}

#[test]
fn a_lost_anchor_is_replaced_by_the_display_nearest_the_window() {
  let monitors = [
    probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
    probe(2, 1920.0, 0.0, 1920.0, 1080.0, 1.0),
  ];

  let near_right = build_binding(
    &monitors,
    99,
    Some(Rect::from_xywh(2000.0, 100.0, 800.0, 600.0)),
  )
  .unwrap();
  assert_eq!(near_right.anchor_id, 2);

  let near_left = build_binding(
    &monitors,
    99,
    Some(Rect::from_xywh(-400.0, 100.0, 300.0, 200.0)),
  )
  .unwrap();
  assert_eq!(near_left.anchor_id, 1);

  // Without a hint the first display stands in.
  assert_eq!(build_binding(&monitors, 99, None).unwrap().anchor_id, 1);
}

#[test]
fn an_empty_or_degenerate_desktop_is_refused() {
  assert!(build_binding(&[], 1, None).is_err());
  assert!(build_binding(&[probe(1, 0.0, 0.0, 0.0, 1080.0, 1.0)], 1, None).is_err());
  assert!(build_binding(&[probe(1, 0.0, 0.0, 1920.0, 1080.0, 0.0)], 1, None).is_err());
}

#[test]
fn the_anchor_never_gets_a_peer_and_every_other_display_does() {
  let monitors = [
    probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
    probe(2, 1920.0, 0.0, 2560.0, 1440.0, 2.0),
    probe(3, -1280.0, 0.0, 1280.0, 1024.0, 1.0),
  ];
  let binding = build_binding(&monitors, 1, None).unwrap();

  let plan = peer_plan(&binding, &monitors);
  assert_eq!(
    plan.iter().map(|peer| peer.display_id).collect::<Vec<_>>(),
    vec![2, 3]
  );
  // Peer windows are positioned in physical pixels...
  let scaled = plan.iter().find(|peer| peer.display_id == 2).unwrap();
  assert_eq!(scaled.bounds, Rect::from_xywh(1920.0, 0.0, 2560.0, 1440.0));
  assert_eq!(scaled.scale, 2.0);
  // ...but their drawing offset is the normalized desktop plane. Dividing
  // each monitor by its own scale is what makes mixed-DPI adjacency
  // approximate: the 200% display starts at 1920/2 = 960 physical-derived
  // points, so in the plane it overlaps its neighbour instead of abutting it.
  assert_eq!(scaled.offset, Point { x: 2240.0, y: 0.0 });
  let left = plan.iter().find(|peer| peer.display_id == 3).unwrap();
  assert_eq!(left.offset, Point { x: 0.0, y: 0.0 });

  // A single-monitor desktop plans no peers at all, which is what keeps
  // desktop presentation a no-op there.
  let single = build_binding(&monitors[..1], 1, None).unwrap();
  assert!(peer_plan(&single, &monitors).is_empty());
}

#[test]
fn a_display_without_physical_geometry_is_skipped_rather_than_guessed() {
  let monitors = [
    probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0),
    probe(2, 1920.0, 0.0, 1920.0, 1080.0, 1.0),
  ];
  let binding = build_binding(&monitors, 1, None).unwrap();

  assert!(peer_plan(&binding, &monitors[..1]).is_empty());
}

#[test]
fn layout_changes_are_detected_from_the_previous_binding() {
  let first = build_binding(&[probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0)], 1, None).unwrap();
  let same = build_binding(&[probe(1, 0.0, 0.0, 1920.0, 1080.0, 1.0)], 1, None).unwrap();
  let resized = build_binding(&[probe(1, 0.0, 0.0, 2560.0, 1440.0, 1.0)], 1, None).unwrap();

  assert!(super::super::state::layout_changed(None, &first));
  assert!(!super::super::state::layout_changed(
    Some(&(first.displays.clone(), first.anchor_id)),
    &same
  ));
  assert!(super::super::state::layout_changed(
    Some(&(first.displays.clone(), first.anchor_id)),
    &resized
  ));
  assert!(super::super::state::layout_changed(
    Some(&(first.displays.clone(), 9)),
    &first
  ));
}
