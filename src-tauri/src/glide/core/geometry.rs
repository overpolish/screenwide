// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral frame policy shared by native placement adapters.

use crate::glide::region_rect::{Gravity, RegionGravity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlideFrame {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

pub fn frame_fits(achieved: GlideFrame, destination: GlideFrame, epsilon: f64) -> bool {
  (achieved.width - destination.width).abs() <= epsilon
    && (achieved.height - destination.height).abs() <= epsilon
}

pub fn frames_match(a: GlideFrame, b: GlideFrame, epsilon: f64) -> bool {
  (a.x - b.x).abs() <= epsilon && (a.y - b.y).abs() <= epsilon && frame_fits(a, b, epsilon)
}

pub fn corrected_origin(
  destination: GlideFrame,
  achieved: GlideFrame,
  gravity: RegionGravity,
) -> (f64, f64) {
  (
    axis_origin(
      destination.x,
      destination.width,
      achieved.width,
      gravity.horizontal,
    ),
    axis_origin(
      destination.y,
      destination.height,
      achieved.height,
      gravity.vertical,
    ),
  )
}

pub fn frame_fractions(
  frame: GlideFrame,
  work_origin: (f64, f64),
  work_size: (f64, f64),
) -> Option<GlideFrame> {
  if work_size.0 <= 0.0 || work_size.1 <= 0.0 {
    return None;
  }
  Some(GlideFrame {
    x: (frame.x - work_origin.0) / work_size.0,
    y: (frame.y - work_origin.1) / work_size.1,
    width: frame.width / work_size.0,
    height: frame.height / work_size.1,
  })
}

/// Keeps the proportional horizontal grip and absolute titlebar depth.
pub fn landing_point(anchor: (f64, f64), original: GlideFrame, achieved: GlideFrame) -> (f64, f64) {
  let ratio = if original.width > 0.0 {
    ((anchor.0 - original.x) / original.width).clamp(0.0, 1.0)
  } else {
    0.5
  };
  let offset = (anchor.1 - original.y).clamp(0.0, achieved.height.max(0.0));
  (achieved.x + ratio * achieved.width, achieved.y + offset)
}

fn axis_origin(origin: f64, extent: f64, achieved: f64, gravity: Gravity) -> f64 {
  match gravity {
    Gravity::Start => origin,
    Gravity::End => (origin + extent - achieved).round(),
    Gravity::Center => (origin + (extent - achieved) / 2.0).round(),
  }
}
