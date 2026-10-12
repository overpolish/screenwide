// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{Handle, Monitor, Point, Rect, Size};

pub(super) fn resized_region(
  rect: Rect,
  handle: Handle,
  dx: f64,
  dy: f64,
  monitor: Monitor,
  aspect: Option<f64>,
) -> Rect {
  let resized = if aspect.is_some_and(|value| value.is_finite() && value > 0.0) {
    uniform_resize(rect, handle, dx, dy, monitor)
  } else {
    free_resize(rect, handle, dx, dy)
  };
  clamp_resize(resized, monitor, handle).snap()
}

fn free_resize(mut rect: Rect, handle: Handle, dx: f64, dy: f64) -> Rect {
  let (mut left, mut top, mut right, mut bottom) =
    (rect.origin.x, rect.origin.y, rect.right(), rect.bottom());
  if matches!(handle, Handle::West | Handle::NorthWest | Handle::SouthWest) {
    left += dx;
  }
  if matches!(handle, Handle::East | Handle::NorthEast | Handle::SouthEast) {
    right += dx;
  }
  if matches!(
    handle,
    Handle::North | Handle::NorthEast | Handle::NorthWest
  ) {
    top += dy;
  }
  if matches!(
    handle,
    Handle::South | Handle::SouthEast | Handle::SouthWest
  ) {
    bottom += dy;
  }
  if right < left {
    std::mem::swap(&mut right, &mut left);
  }
  if bottom < top {
    std::mem::swap(&mut bottom, &mut top);
  }
  rect.origin = Point { x: left, y: top };
  rect.size = Size {
    width: right - left,
    height: bottom - top,
  };
  rect
}

/// Project pointer movement onto the handle-to-anchor vector and uniformly
/// scale the immutable gesture-start rectangle, matching the export editor.
fn uniform_resize(rect: Rect, handle: Handle, dx: f64, dy: f64, monitor: Monitor) -> Rect {
  let center = Point {
    x: rect.origin.x + rect.size.width / 2.0,
    y: rect.origin.y + rect.size.height / 2.0,
  };
  let west = matches!(handle, Handle::West | Handle::NorthWest | Handle::SouthWest);
  let east = matches!(handle, Handle::East | Handle::NorthEast | Handle::SouthEast);
  let north = matches!(
    handle,
    Handle::North | Handle::NorthEast | Handle::NorthWest
  );
  let south = matches!(
    handle,
    Handle::South | Handle::SouthEast | Handle::SouthWest
  );
  let anchor = Point {
    x: if west {
      rect.right()
    } else if east {
      rect.origin.x
    } else {
      center.x
    },
    y: if north {
      rect.bottom()
    } else if south {
      rect.origin.y
    } else {
      center.y
    },
  };
  let active = Point {
    x: if west {
      rect.origin.x
    } else if east {
      rect.right()
    } else {
      center.x
    },
    y: if north {
      rect.origin.y
    } else if south {
      rect.bottom()
    } else {
      center.y
    },
  };
  let vector = Point {
    x: active.x - anchor.x,
    y: active.y - anchor.y,
  };
  let length_squared = vector.x * vector.x + vector.y * vector.y;
  if length_squared <= f64::EPSILON {
    return rect;
  }

  let projected = ((vector.x + dx) * vector.x + (vector.y + dy) * vector.y) / length_squared;
  let minimum = (1.0 / rect.size.width.max(1.0)).max(1.0 / rect.size.height.max(1.0));
  let maximum = maximum_scale(rect, anchor, monitor).max(minimum);
  let scale = projected.max(minimum).min(maximum);
  Rect {
    origin: Point {
      x: anchor.x + (rect.origin.x - anchor.x) * scale,
      y: anchor.y + (rect.origin.y - anchor.y) * scale,
    },
    size: Size {
      width: rect.size.width * scale,
      height: rect.size.height * scale,
    },
  }
}

fn maximum_scale(rect: Rect, anchor: Point, monitor: Monitor) -> f64 {
  let mut maximum = f64::INFINITY;
  for (offset, anchor_axis, extent) in [
    (rect.origin.x - anchor.x, anchor.x, monitor.size.width),
    (rect.right() - anchor.x, anchor.x, monitor.size.width),
    (rect.origin.y - anchor.y, anchor.y, monitor.size.height),
    (rect.bottom() - anchor.y, anchor.y, monitor.size.height),
  ] {
    if offset < 0.0 {
      maximum = maximum.min(anchor_axis / -offset);
    } else if offset > 0.0 {
      maximum = maximum.min((extent - anchor_axis) / offset);
    }
  }
  maximum
}

/// Clamp while retaining the edge opposite the active handle. A generic rect
/// clamp translates the box, moving the supposedly fixed edge as well.
fn clamp_resize(mut rect: Rect, monitor: Monitor, handle: Handle) -> Rect {
  let max_width = monitor.size.width.max(0.0);
  let max_height = monitor.size.height.max(0.0);
  let right = rect.right();
  let bottom = rect.bottom();
  if matches!(handle, Handle::West | Handle::NorthWest | Handle::SouthWest) {
    rect.origin.x = rect.origin.x.max(0.0);
    rect.size.width = (right - rect.origin.x)
      .max(0.0)
      .min(max_width - rect.origin.x);
  } else {
    rect.origin.x = rect.origin.x.max(0.0).min(max_width);
    rect.size.width = rect.size.width.max(0.0).min(max_width - rect.origin.x);
  }
  if matches!(
    handle,
    Handle::North | Handle::NorthEast | Handle::NorthWest
  ) {
    rect.origin.y = rect.origin.y.max(0.0);
    rect.size.height = (bottom - rect.origin.y)
      .max(0.0)
      .min(max_height - rect.origin.y);
  } else {
    rect.origin.y = rect.origin.y.max(0.0).min(max_height);
    rect.size.height = rect.size.height.max(0.0).min(max_height - rect.origin.y);
  }
  rect
}
