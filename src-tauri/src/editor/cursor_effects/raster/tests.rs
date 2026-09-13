// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn interpolated_artwork_produces_fractional_edge_coverage() {
  let cursor = CursorRaster::new(CursorStyle::Arrow, 17.0, 28.0, 40.0, 0.0, 0.0, 4.0);
  let has_partial_pixel = (0..200).any(|y| {
    (0..200).any(|x| {
      let alpha = cursor.sample_for_draw(x as f64 + 0.5, y as f64 + 0.5, 40.0, 40.0)[3];
      alpha > 0.0 && alpha < 255.0
    })
  });
  assert!(has_partial_pixel);
}

#[test]
fn fallback_arrow_keeps_its_native_aspect_inside_a_square_cursor_box() {
  let cursor = CursorRaster::new(CursorStyle::Arrow, 0.0, 32.0, 32.0, 0.0, 0.0, 1.0);
  let rightmost = (0..32)
    .flat_map(|y| (0..32).map(move |x| (x, y)))
    .filter(|(x, y)| cursor.sample(*x as f64 + 0.5, *y as f64 + 0.5, 0.0, 0.0)[3] > 0.0)
    .map(|(x, _)| x)
    .max()
    .unwrap();

  assert!(
    rightmost <= 23,
    "the 28:40 arrow was stretched to x={rightmost}"
  );
}

#[test]
fn custom_cursors_never_share_a_system_style_artwork() {
  assert!(!uses_same_artwork(CursorStyle::Custom, CursorStyle::Arrow));
  assert!(uses_same_artwork(CursorStyle::Custom, CursorStyle::Custom));
  assert!(uses_same_artwork(CursorStyle::Arrow, CursorStyle::Arrow));
}

/// The test process never calls `initialize_system_artwork`, so
/// `gpu_artworks` can only observe the fallback branch; the routing is
/// pinned through `custom_gpu_artwork` with a stand-in arrow entry.
#[cfg(target_os = "macos")]
#[test]
fn custom_cursors_index_the_system_arrow_fitted_to_their_box() {
  assert_eq!(
    artwork_index(CursorStyle::Custom),
    GPU_CUSTOM_ARTWORK_INDEX,
    "a custom cursor must not index the stretched system arrow"
  );
  let arrow = platform::StyleArtwork {
    hotspot_x: 5.0,
    hotspot_y: 5.0,
    image: RgbaImage::from_pixel(28, 40, image::Rgba([1, 2, 3, 255])),
  };
  let artwork = custom_gpu_artwork(Some(&arrow));
  assert_eq!(
    artwork.pixels,
    *arrow.image.as_raw(),
    "the custom slot must carry the system arrow's pixels"
  );
  assert!(
    artwork.use_design,
    "the custom slot must fit its design frame into the box, not stretch"
  );
  assert_eq!(artwork.design_width, 28.0);
  assert_eq!(artwork.design_height, 40.0);
  assert_eq!(
    (artwork.origin_x, artwork.origin_y),
    (5.0, 5.0),
    "the arrow is anchored by its own hotspot, not the recorded one"
  );
  assert!(
    !artwork.clip_local_box,
    "the fitted arrow reaches outside the recorded box at its hotspot"
  );
  assert_eq!(
    artwork.pixels.len(),
    artwork.width as usize * artwork.height as usize * 4
  );
}

/// Without system artwork at all the slot keeps the baked vector arrow.
#[cfg(target_os = "macos")]
#[test]
fn custom_cursors_fall_back_to_the_baked_arrow_without_system_artwork() {
  let artwork = custom_gpu_artwork(None);
  assert!(artwork.use_design);
  assert!(!artwork.clip_local_box);
  assert_eq!(artwork.design_width, 28.0);
  assert_eq!(artwork.design_height, 40.0);
  assert_eq!(
    artwork.pixels.len(),
    artwork.width as usize * artwork.height as usize * 4
  );
  let artworks = gpu_artworks();
  let uploaded = artworks
    .get(GPU_CUSTOM_ARTWORK_INDEX as usize)
    .expect("the custom slot is uploaded");
  assert_eq!(uploaded.pixels, artwork.pixels);
}

/// The CPU raster utility follows the GPU slot's aspect-preserving placement
/// and anchors the arrow's own hotspot at the recorded position.
#[test]
fn custom_cursors_fit_the_system_arrow_by_its_own_hotspot() {
  let image: &'static RgbaImage = Box::leak(Box::new(RgbaImage::from_pixel(
    4,
    8,
    image::Rgba([255, 255, 255, 255]),
  )));
  let mut cursor = CursorRaster::new(CursorStyle::Custom, 0.0, 32.0, 32.0, 0.0, 0.0, 1.0);
  cursor.system_artwork = Some(image);
  cursor.system_design = Some(SystemDesign {
    height: 8.0,
    origin_x: 1.0,
    origin_y: 2.0,
    width: 4.0,
  });

  // min(32 / 4, 32 / 8) = 4, so the 4x8 arrow draws 16x32 and never fills
  // the square box's width.
  let lit = |x: f64, y: f64| cursor.sample(x, y, 0.0, 0.0)[3] > 0.0;
  assert!(lit(-3.5, -7.5), "the artwork's top-left corner was clipped");
  assert!(
    lit(11.5, 23.5),
    "the artwork's bottom-right corner is drawn"
  );
  assert!(
    !lit(12.5, 0.0),
    "the artwork was stretched past 4 x 4 units"
  );
  assert!(
    !lit(0.0, 24.5),
    "the artwork was stretched past 8 x 4 units"
  );
  assert!(
    !lit(-4.5, 0.0) && !lit(0.0, -8.5),
    "the artwork drew outside its design frame"
  );
}

/// With no system artwork loaded (as in this process) a custom cursor still
/// draws the baked vector arrow at its own aspect and tip.
#[test]
fn custom_cursors_keep_the_fallback_arrows_aspect_in_a_square_box() {
  let cursor = CursorRaster::new(CursorStyle::Custom, 0.0, 32.0, 32.0, 0.0, 0.0, 1.0);
  let rightmost = (0..32)
    .flat_map(|y| (0..32).map(move |x| (x, y)))
    .filter(|(x, y)| cursor.sample(*x as f64 + 0.5, *y as f64 + 0.5, 0.0, 0.0)[3] > 0.0)
    .map(|(x, _)| x)
    .max()
    .unwrap();

  assert!(
    rightmost <= 23,
    "the custom cursor stretched the arrow to x={rightmost}"
  );
  assert!(
    cursor.sample(0.0, 0.0, 0.0, 0.0)[3] > 0.0,
    "the arrow's tip must sit at the drawn position"
  );
}

#[test]
fn fallback_arrow_places_its_visible_tip_at_the_recorded_hotspot() {
  let cursor = CursorRaster::new(CursorStyle::Arrow, 0.0, 32.0, 32.0, 0.0, 0.0, 1.0);
  assert!(cursor.sample(0.0, 0.0, 0.0, 0.0)[3] > 0.0);
  assert!(
    cursor.sample(-0.5, 0.0, 0.0, 0.0)[3] > 0.0,
    "the rounded tip stroke was clipped at the hotspot"
  );
}
