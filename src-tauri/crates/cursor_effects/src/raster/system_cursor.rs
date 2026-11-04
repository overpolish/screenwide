// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_records::cursor::CursorStyle;

use super::{natural_box, platform};

/// One of the system's pointers: its pixels, the box it is drawn in, in
/// points, and its hotspot within that box.
pub struct SystemCursorArtwork {
  pub image: &'static image::RgbaImage,
  pub points: (f64, f64),
  pub hotspot: (f64, f64),
}

pub(crate) fn system_cursor(style: CursorStyle) -> Option<SystemCursorArtwork> {
  let [width, height, hotspot_x, hotspot_y] = natural_box(style)?;
  Some(SystemCursorArtwork {
    image: &platform::style_artwork(style)?.image,
    points: (width, height),
    hotspot: (hotspot_x, hotspot_y),
  })
}
