// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use super::{super::fixed_regions, reconstruct};
use crate::screenshots::CapturedImage;

const MAX_OUTPUT_PIXELS: u64 = 64_000_000;

#[derive(Default)]
pub(super) struct Crop {
  pub(super) bottom: u32,
  pub(super) left: u32,
  pub(super) right: u32,
  pub(super) top: u32,
}

pub(super) struct Tile {
  pub(super) crop: Crop,
  pub(super) fixed_bands: Option<fixed_regions::FixedBands>,
  pub(super) image: Arc<CapturedImage>,
  pub(super) x: i32,
  pub(super) y: i32,
}

impl Tile {
  /// The half-open source bounds the placement loop copies from, as
  /// (row_start, row_end, column_start, column_end).
  fn bounds(&self) -> (u32, u32, u32, u32) {
    let row_end = self.image.height.saturating_sub(self.crop.bottom);
    let column_end = self.image.width.saturating_sub(self.crop.right);
    (
      self.crop.top.min(row_end),
      row_end,
      self.crop.left.min(column_end),
      column_end,
    )
  }
}

pub(super) fn compose(tiles: Vec<Tile>) -> Result<CapturedImage, String> {
  let first = tiles
    .first()
    .ok_or_else(|| "The scrolling capture produced no frames".to_owned())?;
  let min_x = tiles.iter().map(|tile| tile.x).min().unwrap_or(0);
  let min_y = tiles.iter().map(|tile| tile.y).min().unwrap_or(0);
  let max_x = tiles
    .iter()
    .map(|tile| i64::from(tile.x) + i64::from(tile.image.width))
    .max()
    .unwrap_or(i64::from(first.image.width));
  let max_y = tiles
    .iter()
    .map(|tile| i64::from(tile.y) + i64::from(tile.image.height))
    .max()
    .unwrap_or(i64::from(first.image.height));
  let width = u32::try_from(max_x - i64::from(min_x))
    .map_err(|_| "The scrolling capture width is invalid".to_owned())?;
  let height = u32::try_from(max_y - i64::from(min_y))
    .map_err(|_| "The scrolling capture height is invalid".to_owned())?;
  if u64::from(width) * u64::from(height) > MAX_OUTPUT_PIXELS {
    return Err("The scrolling capture is too large".to_owned());
  }

  let has_vertical_movement = tiles.windows(2).any(|pair| pair[0].y != pair[1].y);
  let right_edge_width = if has_vertical_movement {
    tiles
      .iter()
      .filter_map(|tile| {
        tile
          .fixed_bands
          .as_ref()
          .map(|bands| bands.right_edge_width(&tile.image))
      })
      .max()
      .unwrap_or(0)
  } else {
    0
  };
  let tile_count = tiles.len();
  let mut rgba = vec![0_u8; width as usize * height as usize * 4];
  // Mask fills are only a stand-in for a rail's background: where a later tile
  // carries real pixels for the same document row — the footer that replaces a
  // sticky sidebar at the page bottom — those must win. Real pixels still take
  // the first writer, so ordinary overlap seams are unaffected.
  let mut is_placeholder = vec![false; width as usize * height as usize];
  for (tile_index, tile) in tiles.iter().enumerate() {
    let destination_x = (tile.x - min_x) as u32;
    let destination_y = (tile.y - min_y) as u32;
    let (row_start, row_end, column_start, column_end) = tile.bounds();
    for row in row_start..row_end {
      for column in column_start..column_end {
        let source = ((row * tile.image.width + column) * 4) as usize;
        let pixel = ((destination_y + row) * width + destination_x + column) as usize;
        let destination = pixel * 4;
        // The terminal tile contains authoritative end-of-page content, and
        // rows it alone covers have no later tile to replace a placeholder, so
        // its exclusion mask is never applied.
        let masked = (tile_index + 1 < tile_count)
          .then_some(tile.fixed_bands.as_ref())
          .flatten()
          .and_then(|bands| bands.background(column, row, &tile.image));
        if rgba[destination + 3] == 0 {
          match masked {
            Some(background) => {
              rgba[destination..destination + 4].copy_from_slice(&background);
              is_placeholder[pixel] = true;
            }
            None => rgba[destination..destination + 4]
              .copy_from_slice(&tile.image.rgba[source..source + 4]),
          }
        } else if is_placeholder[pixel] && masked.is_none() {
          rgba[destination..destination + 4].copy_from_slice(&tile.image.rgba[source..source + 4]);
          is_placeholder[pixel] = false;
        }
      }
    }
  }

  // Overlay scrollbars and minimaps move independently from the document, so
  // the strip they occupy is rebuilt from what the overlapping tiles agree on
  // rather than painted over: the chrome moves between frames, the content
  // beneath it does not.
  if right_edge_width > 0 && right_edge_width < width {
    // Chrome can begin anywhere inside the last band the detector left
    // unmasked, so the strip reaches one band further left. Widening is free of
    // risk because reconstruction only ever rewrites a pixel two tiles already
    // agree on, which reproduces the document verbatim.
    let strip_start = width
      .saturating_sub(right_edge_width)
      .saturating_sub(fixed_regions::detection_band_width(width));
    let coverage: Vec<reconstruct::Coverage<'_>> = tiles
      .iter()
      .map(|tile| {
        let (row_start, row_end, column_start, column_end) = tile.bounds();
        reconstruct::Coverage {
          column_end,
          column_start,
          image: tile.image.as_ref(),
          origin_x: (tile.x - min_x) as u32,
          origin_y: (tile.y - min_y) as u32,
          row_end,
          row_start,
        }
      })
      .collect();
    reconstruct::reconstruct_right_edge(&mut rgba, width, height, strip_start, &coverage);
  }

  Ok(CapturedImage {
    height,
    rgba,
    width,
  })
}
