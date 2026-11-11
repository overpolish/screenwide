// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_osc::desktop::DesktopDisplay;
use screenwide_osc::geometry::{Point, Size};

use super::*;

fn display(id: u32, x: f64) -> DesktopDisplay {
  DesktopDisplay {
    id,
    origin: Point { x, y: 0.0 },
    size: Size {
      width: 100.0,
      height: 80.0,
    },
    scale: 2.0,
  }
}

#[test]
fn an_area_across_two_displays_belongs_to_the_one_holding_most_of_it() {
  let binding = DesktopBinding {
    displays: vec![display(1, 0.0), display(2, 100.0)],
    anchor_id: 1,
    size: Size {
      width: 200.0,
      height: 80.0,
    },
    layout_changed: false,
  };
  let (id, rect) = local_box(&binding, Rect::from_xywh(90.0, 10.0, 40.0, 20.0)).unwrap();
  assert_eq!(id, 2);
  assert_eq!(
    rect,
    LocalRect {
      x: 0.0,
      y: 10.0,
      width: 30.0,
      height: 20.0,
    }
  );
}
