// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How finely the annotations' type is rasterised into the atlas, decided
//! once for both backends.
//!
//! The atlas holds type at a fixed number of pixels per canvas pixel, and the
//! kernels draw it wherever the canvas lands on screen. A fixed density would
//! be too coarse for a canvas shown larger than its resolution - a Retina
//! display, or a zoomed-in editor - where one atlas pixel spreads over
//! several drawn ones and the type comes out in blocks, and wastefully fine
//! for one shown smaller. So the density follows how large one canvas pixel
//! is drawn.

/// Atlas pixels per drawn pixel, at the least: two, so the four-tap read
/// over each drawn pixel lands on distinct atlas pixels.
const SUPERSAMPLE: f32 = 2.0;
/// The largest type size the atlas rasterises, in atlas pixels. Past it a
/// zoomed-in view magnifies the raster instead: a long line of type set any
/// larger would outgrow the texture side both backends accept.
const MAX_RASTER_SIZE: f32 = 512.0;
/// The range of drawn-pixel densities the raster follows, as powers of two:
/// from a canvas shown at a sixteenth of its size to one canvas pixel drawn
/// over thirty-two, a Retina display at the editor's largest zoom.
const DENSITY_STEPS: (f32, f32) = (-4.0, 5.0);

/// Atlas pixels per canvas pixel for type drawn where one drawn pixel covers
/// `pixel_scale` canvas pixels, the largest of it `largest` canvas pixels in
/// size. Zero or less for `pixel_scale` means one canvas pixel per drawn one.
///
/// The drawn density is rounded up to a power of two, so a zoom drag
/// rasterises afresh at each doubling rather than on every frame, and one
/// drawn pixel always spans between two and four atlas pixels. Large type
/// zoomed in stops at [`MAX_RASTER_SIZE`], but never below the density a
/// canvas shown at its own size is rasterised at.
pub(crate) fn raster_scale(pixel_scale: f32, largest: f32) -> f32 {
  let density = if pixel_scale > 0.0 && pixel_scale.is_finite() {
    pixel_scale.recip()
  } else {
    1.0
  };
  let step = density
    .log2()
    .ceil()
    .clamp(DENSITY_STEPS.0, DENSITY_STEPS.1)
    .exp2();
  let wanted = SUPERSAMPLE * step;
  if wanted <= SUPERSAMPLE || !(largest > 0.0) {
    return wanted;
  }
  let ceiling = (MAX_RASTER_SIZE / largest).log2().floor().exp2();
  wanted.min(ceiling.max(SUPERSAMPLE))
}

#[cfg(test)]
mod tests {
  use super::raster_scale;

  #[test]
  fn keeps_two_atlas_pixels_per_canvas_pixel_at_its_own_size() {
    assert_eq!(raster_scale(1.0, 24.0), 2.0);
    assert_eq!(raster_scale(0.0, 24.0), 2.0);
  }

  #[test]
  fn follows_a_magnified_canvas_in_doublings() {
    // Retina at 100%: one canvas pixel over two drawn ones.
    assert_eq!(raster_scale(0.5, 24.0), 4.0);
    // Between doublings, the next one up, so a drawn pixel never gets fewer
    // than two atlas pixels.
    assert_eq!(raster_scale(0.3, 24.0), 8.0);
    assert_eq!(raster_scale(0.25, 24.0), 8.0);
  }

  #[test]
  fn thins_out_for_a_canvas_shown_smaller() {
    assert_eq!(raster_scale(2.0, 24.0), 1.0);
    assert_eq!(raster_scale(3.0, 24.0), 1.0);
    assert_eq!(raster_scale(1024.0, 24.0), 2.0 / 16.0);
  }

  #[test]
  fn large_type_stops_growing_but_never_below_its_own_size() {
    // 100px type at 16x would set a 3200px face; it stops at 400px.
    assert_eq!(raster_scale(1.0 / 16.0, 100.0), 4.0);
    // Type already past the ceiling at its own size keeps today's raster.
    assert_eq!(raster_scale(1.0 / 16.0, 400.0), 2.0);
    assert_eq!(raster_scale(1.0, 400.0), 2.0);
  }
}
