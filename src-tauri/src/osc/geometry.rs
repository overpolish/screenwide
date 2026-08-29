// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Monitor-local geometry. Coordinates are logical points; renderers decide
//! how to scale them for their display.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
  pub x: f64,
  pub y: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
  pub width: f64,
  pub height: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
  pub origin: Point,
  pub size: Size,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monitor {
  pub size: Size,
}

impl Point {
  pub fn finite(self) -> bool {
    self.x.is_finite() && self.y.is_finite()
  }
}
impl Size {
  pub fn finite(self) -> bool {
    self.width.is_finite() && self.height.is_finite()
  }
  pub fn valid(self) -> bool {
    self.finite() && self.width >= 0.0 && self.height >= 0.0
  }
}
impl Rect {
  pub fn finite(self) -> bool {
    self.origin.finite() && self.size.finite()
  }
  pub fn valid(self) -> bool {
    self.finite() && self.size.valid()
  }
  pub fn committed(self) -> bool {
    self.valid() && self.size.width > 1.0 && self.size.height > 1.0
  }
  pub fn right(self) -> f64 {
    self.origin.x + self.size.width
  }
  pub fn bottom(self) -> f64 {
    self.origin.y + self.size.height
  }
  pub fn clamp(self, monitor: Monitor) -> Self {
    let w = self.size.width.max(0.0).min(monitor.size.width.max(0.0));
    let h = self.size.height.max(0.0).min(monitor.size.height.max(0.0));
    Self {
      origin: Point {
        x: self.origin.x.max(0.0).min(monitor.size.width - w).max(0.0),
        y: self.origin.y.max(0.0).min(monitor.size.height - h).max(0.0),
      },
      size: Size {
        width: w,
        height: h,
      },
    }
  }
  pub fn snap(self) -> Self {
    Self {
      origin: Point {
        x: self.origin.x.round(),
        y: self.origin.y.round(),
      },
      size: Size {
        width: self.size.width.round().max(1.0),
        height: self.size.height.round().max(1.0),
      },
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
  Body,
  North,
  South,
  East,
  West,
  NorthEast,
  NorthWest,
  SouthEast,
  SouthWest,
}

pub fn drawn_region(monitor: Monitor, start: Point, end: Point, aspect: Option<f64>) -> Rect {
  let clamp = |v: f64, max: f64| v.max(0.0).min(max.max(0.0));
  let s = Point {
    x: clamp(start.x, monitor.size.width),
    y: clamp(start.y, monitor.size.height),
  };
  let e = Point {
    x: clamp(end.x, monitor.size.width),
    y: clamp(end.y, monitor.size.height),
  };
  let mut w = (e.x - s.x).abs();
  let mut h = (e.y - s.y).abs();
  if let Some(ratio) = aspect.filter(|r| r.is_finite() && *r > 0.0) {
    let wide = (if h == 0.0 { f64::INFINITY } else { w / h }) > ratio;
    let (rw, rh) = if wide { (w, w / ratio) } else { (h * ratio, h) };
    let avail_w = if e.x < s.x {
      s.x
    } else {
      monitor.size.width - s.x
    };
    let avail_h = if e.y < s.y {
      s.y
    } else {
      monitor.size.height - s.y
    };
    let fit = 1.0_f64
      .min(if rw > 0.0 { avail_w / rw } else { 1.0 })
      .min(if rh > 0.0 { avail_h / rh } else { 1.0 });
    w = rw * fit;
    h = rh * fit;
  }
  let w = w.round().max(1.0).min(monitor.size.width.max(0.0));
  let h = h.round().max(1.0).min(monitor.size.height.max(0.0));
  Rect {
    origin: Point {
      x: (if e.x < s.x { s.x - w } else { s.x })
        .max(0.0)
        .min(monitor.size.width - w)
        .round(),
      y: (if e.y < s.y { s.y - h } else { s.y })
        .max(0.0)
        .min(monitor.size.height - h)
        .round(),
    },
    size: Size {
      width: w,
      height: h,
    },
  }
}

#[cfg(test)]
pub fn fit_region(rect: Rect, monitor: Monitor) -> Rect {
  let max_width = monitor.size.width.max(0.0);
  let max_height = monitor.size.height.max(0.0);
  let w = rect
    .size
    .width
    .min((max_width - 20.0).max(0.0))
    .round()
    .max(1.0)
    .min(max_width);
  let h = rect
    .size
    .height
    .min((max_height - 20.0).max(0.0))
    .round()
    .max(1.0)
    .min(max_height);
  Rect {
    origin: Point {
      x: rect.origin.x.max(0.0).min(max_width - w).round(),
      y: rect.origin.y.max(0.0).min(max_height - h).round(),
    },
    size: Size {
      width: w,
      height: h,
    },
  }
}
#[cfg(test)]
pub fn centered(size: Size, monitor: Monitor) -> Rect {
  Rect {
    origin: Point {
      x: ((monitor.size.width - size.width) / 2.0).round(),
      y: ((monitor.size.height - size.height) / 2.0).round(),
    },
    size,
  }
  .clamp(monitor)
  .snap()
}

#[cfg(test)]
mod tests {
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
}
