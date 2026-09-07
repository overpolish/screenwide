// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::contained_origin;
use tauri::{PhysicalPosition, PhysicalSize};

#[test]
fn preview_is_contained_at_each_work_area_edge() {
  let work_origin = PhysicalPosition::new(-1_920, 40);
  let work_size = PhysicalSize::new(1_920, 1_040);
  let preview_size = PhysicalSize::new(72, 48);

  assert_eq!(
    contained_origin(
      PhysicalPosition::new(-1_956, 16),
      preview_size,
      work_origin,
      work_size,
    ),
    PhysicalPosition::new(-1_920, 40),
  );
  assert_eq!(
    contained_origin(
      PhysicalPosition::new(-20, 1_060),
      preview_size,
      work_origin,
      work_size,
    ),
    PhysicalPosition::new(-72, 1_032),
  );
}

#[test]
fn preview_larger_than_work_area_pins_to_its_origin() {
  assert_eq!(
    contained_origin(
      PhysicalPosition::new(40, 50),
      PhysicalSize::new(400, 300),
      PhysicalPosition::new(100, 80),
      PhysicalSize::new(200, 100),
    ),
    PhysicalPosition::new(100, 80),
  );
}
