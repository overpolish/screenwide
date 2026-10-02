// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A magnifier's prepared geometry, as the macOS chrome reaches it. Its
//! loupe's size and its rounding ride beside its three points rather than
//! among them, so it has an entry point of its own beside
//! `screenwide_annotation_prepare`.

use super::geometry::{magnify_part, prepare_magnify, MagnifyPart};
use crate::editor::annotations::geometry::ArrowGeometry;
use crate::editor::annotations::reveal::AnnotationReveal;

/// A magnifier's draw geometry, in the pixel space its points are given in:
/// its zoom area from `p0` to `p2` and its loupe centred on `p1`, `size`
/// along its longer side in those pixels, rounded by `radius` percent of the
/// shorter side, its rim `width` wide.
///
/// # Safety
/// `out` must point at one writable [`ArrowGeometry`].
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn screenwide_magnify_prepare(
  p0x: f32,
  p0y: f32,
  p1x: f32,
  p1y: f32,
  p2x: f32,
  p2y: f32,
  size: f32,
  radius: f32,
  width: f32,
  reveal: AnnotationReveal,
  out: *mut ArrowGeometry,
) {
  if out.is_null() {
    return;
  }
  *out = prepare_magnify(
    [p0x, p0y],
    [p1x, p1y],
    [p2x, p2y],
    size,
    radius,
    width,
    reveal,
  );
}

/// Which part of a prepared magnifier a point is on: 1 for the loupe, 2 for
/// the zoom area, 0 for neither. The loupe wins where the two meet.
///
/// # Safety
/// `geometry` must point at one readable [`ArrowGeometry`].
#[no_mangle]
pub unsafe extern "C" fn screenwide_magnify_part(
  px: f32,
  py: f32,
  geometry: *const ArrowGeometry,
) -> u32 {
  match geometry
    .as_ref()
    .and_then(|geometry| magnify_part([px, py], geometry))
  {
    Some(MagnifyPart::Loupe) => 1,
    Some(MagnifyPart::Area) => 2,
    None => 0,
  }
}
