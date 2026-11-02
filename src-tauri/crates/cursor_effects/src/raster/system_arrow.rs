// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_records::cursor::CursorStyle;

use super::{natural_box, platform};

/// The system's arrow: its pixels, the box it is drawn in, in points, and
/// its hotspot within that box.
pub struct SystemArrow {
  pub image: &'static image::RgbaImage,
  pub points: (f64, f64),
  pub hotspot: (f64, f64),
}

pub(crate) fn system_arrow() -> Option<SystemArrow> {
  let [width, height, hotspot_x, hotspot_y] = natural_box(CursorStyle::Arrow)?;
  Some(SystemArrow {
    image: &platform::style_artwork(CursorStyle::Arrow)?.image,
    points: (width, height),
    hotspot: (hotspot_x, hotspot_y),
  })
}
