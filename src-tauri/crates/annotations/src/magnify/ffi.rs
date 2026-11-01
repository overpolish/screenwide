// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A lens's prepared geometry, as the macOS chrome reaches it. Its zoom and
//! its rounding ride beside its corners rather than among them, so it has an
//! entry point of its own beside `screenwide_annotation_prepare`.

use super::geometry::prepare_magnify;
use crate::geometry::ArrowGeometry;
use crate::reveal::AnnotationReveal;

/// A lens's draw geometry, in the pixel space its corners are given in: its
/// box from `p0` to `p2`, enlarging `zoom` times, rounded by `radius` percent
/// of its shorter side, `width` wide.
///
/// # Safety
/// `out` must point at one writable [`ArrowGeometry`].
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn screenwide_magnify_prepare(
  p0x: f32,
  p0y: f32,
  p2x: f32,
  p2y: f32,
  zoom: f32,
  radius: f32,
  width: f32,
  reveal: AnnotationReveal,
  out: *mut ArrowGeometry,
) {
  if out.is_null() {
    return;
  }
  *out = prepare_magnify([p0x, p0y], [p2x, p2y], zoom, radius, width, reveal);
}
