// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
fn m() -> Monitor {
  Monitor {
    size: Size {
      width: 100.0,
      height: 80.0,
    },
  }
}
#[test]
fn reverse_draw() {
  let r = drawn_region(
    m(),
    Point { x: 80.2, y: 60.2 },
    Point { x: 10.1, y: 5.1 },
    None,
  );
  assert_eq!(
    r,
    Rect {
      origin: Point { x: 10., y: 5. },
      size: Size {
        width: 70.,
        height: 55.
      }
    }
  );
}
#[test]
fn aspect_fits_edge() {
  let r = drawn_region(
    m(),
    Point { x: 90., y: 70. },
    Point { x: 100., y: 80. },
    Some(2.),
  );
  assert_eq!(
    r.size,
    Size {
      width: 10.,
      height: 5.
    }
  );
  assert!(r.right() <= 100. && r.bottom() <= 80.);
}
#[test]
fn fit_and_center_round() {
  assert_eq!(
    fit_region(
      Rect {
        origin: Point { x: 90.4, y: 90. },
        size: Size {
          width: 50.,
          height: 50.
        }
      },
      m()
    )
    .origin,
    Point { x: 50., y: 30. }
  );
  assert_eq!(
    centered(
      Size {
        width: 21.,
        height: 11.
      },
      m()
    )
    .origin,
    Point { x: 40., y: 35. }
  );
}
#[test]
fn invalid_and_uncommitted_geometry_are_distinct() {
  let invalid = Rect {
    origin: Point { x: f64::NAN, y: 0. },
    size: Size {
      width: 2.,
      height: 2.,
    },
  };
  assert!(!invalid.finite());
  assert!(!invalid.valid());
  assert!(!invalid.committed());
  let draft = Rect {
    origin: Point::default(),
    size: Size {
      width: 1.,
      height: 2.,
    },
  };
  assert!(draft.valid());
  assert!(!draft.committed());
}
