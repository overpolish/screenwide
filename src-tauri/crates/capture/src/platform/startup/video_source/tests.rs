// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use tauri::{LogicalPosition, LogicalSize};

fn display(id: u32, x: f64) -> DesktopDisplay {
  DesktopDisplay {
    id,
    x,
    y: 0.0,
    width: 1_000.0,
    height: 800.0,
    scale: 1.0,
  }
}

#[test]
fn trims_window_dimensions_to_encoder_safe_pixels() {
  assert_eq!(pixel_dimension(801.4, 2.0), Some(1_602));
  assert_eq!(pixel_dimension(800.6, 1.0), Some(800));
}

#[test]
fn rejects_empty_or_invalid_window_dimensions() {
  assert_eq!(pixel_dimension(0.0, 2.0), None);
  assert_eq!(pixel_dimension(f64::NAN, 2.0), None);
  assert_eq!(pixel_dimension(100.0, 0.0), None);
}

#[test]
fn window_cursor_coordinates_keep_the_global_window_origin() {
  let source = window_cursor_source(
    42,
    cg::Rect::new(0.0, 0.0, 900.0, 600.0),
    cg::Rect::new(125.0, 80.0, 900.0, 600.0),
    1_800,
    1_200,
  );
  assert_eq!((source.x, source.y), (125.0, 80.0));
  assert_eq!((source.width, source.height), (900.0, 600.0));
}

#[test]
fn only_regions_intersecting_multiple_displays_use_composition() {
  let displays = [display(1, 0.0), display(2, 1_000.0)];
  let contained = Region {
    position: LogicalPosition::new(100.0, 100.0),
    size: LogicalSize::new(500.0, 400.0),
  };
  assert!(composed_region_plan(&displays, 1, contained)
    .unwrap()
    .is_none());

  let crossing = Region {
    position: LogicalPosition::new(800.0, 100.0),
    size: LogicalSize::new(500.0, 400.0),
  };
  let plan = composed_region_plan(&displays, 1, crossing)
    .unwrap()
    .expect("a cross-display plan");
  assert_eq!(plan.pieces.len(), 2);
  assert_eq!((plan.width, plan.height), (500, 400));
}
