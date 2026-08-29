// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{LogicalPosition, LogicalSize};

use crate::recording::Region;

use super::*;

const DISPLAYS: [DesktopDisplay; 2] = [
  DesktopDisplay {
    id: 1,
    x: 0.0,
    y: 0.0,
    width: 1800.0,
    height: 1169.0,
    scale: 2.0,
  },
  DesktopDisplay {
    id: 2,
    x: 1800.0,
    y: 89.0,
    width: 1920.0,
    height: 1080.0,
    scale: 1.0,
  },
];

#[test]
fn plans_mixed_scale_pieces_in_shared_desktop_coordinates() {
  let plan = plan(
    &DISPLAYS,
    2,
    Region {
      position: LogicalPosition::new(-200.0, 11.0),
      size: LogicalSize::new(500.0, 300.0),
    },
    OutputLimits::UNBOUNDED,
  )
  .unwrap();
  assert_eq!(
    (plan.width, plan.height, plan.output_scale),
    (1000, 600, 2.0)
  );
  assert_eq!(
    (plan.desktop_region.x, plan.desktop_region.y),
    (1600.0, 100.0)
  );
  assert_eq!(plan.pieces.len(), 2);
  assert_eq!(plan.pieces[0].source_pixels.width, 400);
  assert_eq!(plan.pieces[1].source_pixels.width, 300);
  assert_eq!(plan.pieces[0].destination.width, 400);
  assert_eq!(plan.pieces[1].destination.x, 400);
}

#[test]
fn rounds_a_shared_seam_to_one_output_edge() {
  let plan = plan(
    &DISPLAYS,
    1,
    Region {
      position: LogicalPosition::new(1699.75, 100.0),
      size: LogicalSize::new(300.5, 100.0),
    },
    OutputLimits::UNBOUNDED,
  )
  .unwrap();
  let left = plan.pieces[0].destination;
  let right = plan.pieces[1].destination;
  assert_eq!(left.x + left.width, right.x);
  assert_eq!(right.x + right.width, plan.width);
}

#[test]
fn bounds_video_by_dimensions_area_and_alignment() {
  let plan = plan(
    &DISPLAYS,
    1,
    Region {
      position: LogicalPosition::new(0.0, 0.0),
      size: LogicalSize::new(3720.0, 1169.0),
    },
    OutputLimits::VIDEO,
  )
  .unwrap();
  assert_eq!(plan.width % 2, 0);
  assert_eq!(plan.height % 2, 0);
  assert!(u64::from(plan.width) * u64::from(plan.height) <= 3840 * 2160);
  assert!(plan.output_scale < 2.0);
}

#[test]
fn waits_for_every_source_then_holds_each_latest_frame() {
  let mut sync = FrameSynchronizer::new(2).unwrap();
  assert_eq!(sync.update(0, 100).unwrap(), None);
  assert_eq!(
    sync.update(1, 105).unwrap(),
    Some(CompositionTick {
      output_ns: 105,
      source_ns: vec![100, 105],
    })
  );
  assert_eq!(sync.update(0, 103).unwrap(), None);
  assert_eq!(
    sync.update(0, 110).unwrap(),
    Some(CompositionTick {
      output_ns: 110,
      source_ns: vec![110, 105],
    })
  );
}

#[test]
fn rejects_unknown_sources_and_stale_or_invalid_timestamps() {
  let mut sync = FrameSynchronizer::new(1).unwrap();
  assert!(sync.update(1, 0).is_err());
  assert!(sync.update(0, -1).is_err());
  assert!(sync.update(0, 10).unwrap().is_some());
  assert_eq!(sync.update(0, 10).unwrap(), None);
  assert_eq!(sync.update(0, 9).unwrap(), None);
}
