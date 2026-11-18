// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A canvas, or a picture on it, larger than the device takes, drawn a band
//! at a time. Each band is a target the device takes, and each layer's
//! picture is uploaded as the window of it the band reads: the source rows
//! the band shows, what the lenses and highlights there read for the part of
//! them the band draws, and as far again as a redaction reads past the pixel
//! it paints. A focus blur reads the canvas around each pixel, so bands with
//! one are drawn overlapping and only their middles kept.

use super::band_needs::Read;
use super::layer::CanvasBand;

/// What one layer's picture asks of the bands that draw it.
pub(super) struct LayerNeeds {
  pub(super) source: (u32, u32),
  /// Where the upright picture lies on the canvas, x, y, width and height
  /// in canvas pixels.
  pub(super) image_rect: [f64; 4],
  /// The inverse of the layer's turn in depth, which carries a canvas pixel
  /// back to the upright canvas, or `None` where it stands upright.
  pub(super) inverse_turn: Option<[[f32; 3]; 3]>,
  /// What the lenses and highlights read of the picture, in source pixels.
  pub(super) reads: Vec<Read>,
  /// How far past the pixel it paints a redaction reads, in source pixels.
  pub(super) margin: u32,
  /// How far around a pixel the focus blur reads, in canvas pixels.
  pub(super) overlap: u32,
  /// How far past the part of the canvas it draws a band reads the
  /// annotations, in canvas pixels: the spotlights' blur lays them a little
  /// wider than the band.
  pub(super) reads_past: u32,
}

/// One band: the part of the canvas its target holds, the part of that
/// kept, x0, y0, x1, y1 in canvas pixels, and the window of each layer's
/// picture it reads, x0, y0, x1, y1 in source pixels, `None` where the
/// layer's whole picture is uploaded at once.
#[derive(Debug, PartialEq)]
pub(super) struct Band {
  pub(super) target: CanvasBand,
  pub(super) kept: [u32; 4],
  pub(super) windows: Vec<Option<[u32; 4]>>,
}

/// The smallest band side the plan shrinks to before it gives the windows
/// up to what fits instead.
const SMALLEST_SIDE: u32 = 64;

/// The bands a `canvas` drawn from `layers` takes on a device whose
/// textures are at most `limit` on a side; one band covering everything
/// where all of it fits.
pub(super) fn plan(canvas: (u32, u32), limit: u32, layers: &[LayerNeeds]) -> Vec<Band> {
  let whole = |layer: &LayerNeeds| layer.source.0 <= limit && layer.source.1 <= limit;
  if canvas.0 <= limit && canvas.1 <= limit && layers.iter().all(whole) {
    return vec![Band {
      target: CanvasBand {
        origin: (0, 0),
        size: canvas,
      },
      kept: [0, 0, canvas.0, canvas.1],
      windows: vec![None; layers.len()],
    }];
  }
  // Every band's target must fit with its overlap either side of it.
  let overlap = layers
    .iter()
    .map(|layer| layer.overlap)
    .max()
    .unwrap_or(0)
    .min(limit.saturating_sub(SMALLEST_SIDE) / 2);
  // A side that fits whole needs no overlap along it.
  let side = |length: u32| {
    if length <= limit {
      length
    } else {
      limit.saturating_sub(2 * overlap).max(SMALLEST_SIDE)
    }
  };
  let mut tile = (side(canvas.0), side(canvas.1));
  loop {
    let bands = tiles(canvas, tile, overlap)
      .map(|(target, kept)| {
        let windows = layers
          .iter()
          .map(|layer| (!whole(layer)).then(|| needed(layer, area(&target))))
          .collect::<Vec<_>>();
        Band {
          target,
          kept,
          windows,
        }
      })
      .collect::<Vec<_>>();
    let widest = |axis: usize| {
      bands
        .iter()
        .flat_map(|band| band.windows.iter().flatten())
        .map(|window| window[axis + 2] - window[axis])
        .max()
        .unwrap_or(0)
    };
    let shrink = |length: u32, needed: u32| {
      if needed <= limit {
        return length;
      }
      let scaled = (u64::from(length) * u64::from(limit) * 9 / (10 * u64::from(needed))) as u32;
      scaled.max(SMALLEST_SIDE).min(length)
    };
    let next = (shrink(tile.0, widest(0)), shrink(tile.1, widest(1)));
    if next == tile {
      // Nothing smaller helps: even the smallest band shows more of the
      // picture than the device takes, as one drawn at under a hundredth of
      // its size, or turned until a band reaches past its horizon, does.
      // Its window keeps what fits round the band's own.
      return bands
        .into_iter()
        .map(|band| fitted(band, layers, limit))
        .collect();
    }
    tile = next;
  }
}

/// The bands `tile` in size that cover `canvas`, each drawn `overlap` past
/// its kept part where the canvas goes on: the target, then the kept part.
fn tiles(
  canvas: (u32, u32),
  tile: (u32, u32),
  overlap: u32,
) -> impl Iterator<Item = (CanvasBand, [u32; 4])> {
  let starts = |length: u32, step: u32| (0..length).step_by(step.max(1) as usize);
  starts(canvas.1, tile.1).flat_map(move |y| {
    starts(canvas.0, tile.0).map(move |x| {
      let kept = [x, y, (x + tile.0).min(canvas.0), (y + tile.1).min(canvas.1)];
      let low = (x.saturating_sub(overlap), y.saturating_sub(overlap));
      let high = (
        (kept[2] + overlap).min(canvas.0),
        (kept[3] + overlap).min(canvas.1),
      );
      let target = CanvasBand {
        origin: low,
        size: (high.0 - low.0, high.1 - low.1),
      };
      (target, kept)
    })
  })
}

/// The part of the canvas `band` holds, x0, y0, x1, y1 in canvas pixels.
fn area(band: &CanvasBand) -> [f64; 4] {
  let (x0, y0) = (f64::from(band.origin.0), f64::from(band.origin.1));
  [
    x0,
    y0,
    x0 + f64::from(band.size.0),
    y0 + f64::from(band.size.1),
  ]
}

/// The part of `layer`'s picture a draw of `part` of the canvas reads, x0,
/// y0, x1, y1 in canvas pixels, which may reach past the canvas's edge.
pub(super) fn needed(layer: &LayerNeeds, part: [f64; 4]) -> [u32; 4] {
  let past = f64::from(layer.reads_past);
  let shown = shown(
    layer,
    [
      part[0] - past,
      part[1] - past,
      part[2] + past,
      part[3] + past,
    ],
  );
  let reaches = layer
    .reads
    .iter()
    .filter_map(|read| read.of(shown))
    .fold(shown, union);
  let margin = f64::from(layer.margin);
  clamp(
    [
      reaches[0] - margin,
      reaches[1] - margin,
      reaches[2] + margin,
      reaches[3] + margin,
    ],
    layer.source,
  )
}

/// The source pixels `part` of the canvas shows of `layer`, x0, y0, x1, y1,
/// with one either side that a filtered read or a feathered edge takes in.
fn shown(layer: &LayerNeeds, [x0, y0, x1, y1]: [f64; 4]) -> [f64; 4] {
  let everything = [
    0.0,
    0.0,
    f64::from(layer.source.0),
    f64::from(layer.source.1),
  ];
  let corners = [[x0, y0], [x1, y0], [x0, y1], [x1, y1]];
  let upright = match layer.inverse_turn {
    None => corners,
    Some(rows) => {
      let mut carried = corners;
      for (corner, point) in carried.iter_mut().zip(corners) {
        let dot = |row: [f32; 3]| {
          f64::from(row[0]) * point[0] + f64::from(row[1]) * point[1] + f64::from(row[2])
        };
        let weight = dot(rows[2]);
        // A corner past the horizon shows points as far as the picture goes.
        if weight <= 1e-6 {
          return everything;
        }
        *corner = [dot(rows[0]) / weight, dot(rows[1]) / weight];
      }
      carried
    }
  };
  let [ix, iy, iw, ih] = layer.image_rect;
  if iw <= 0.0 || ih <= 0.0 {
    return everything;
  }
  let to_source = |[x, y]: [f64; 2]| {
    [
      (x - ix) * f64::from(layer.source.0) / iw,
      (y - iy) * f64::from(layer.source.1) / ih,
    ]
  };
  let points = upright.map(to_source);
  let low = points.iter().fold([f64::INFINITY; 2], |low, point| {
    [low[0].min(point[0]), low[1].min(point[1])]
  });
  let high = points.iter().fold([f64::NEG_INFINITY; 2], |high, point| {
    [high[0].max(point[0]), high[1].max(point[1])]
  });
  [low[0] - 2.0, low[1] - 2.0, high[0] + 2.0, high[1] + 2.0]
}

pub(super) fn meets(a: [f64; 4], b: [f64; 4]) -> bool {
  a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

fn union(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
  [
    a[0].min(b[0]),
    a[1].min(b[1]),
    a[2].max(b[2]),
    a[3].max(b[3]),
  ]
}

/// `rect` in whole pixels inside a `source`-sized picture, at least one
/// pixel across so even a band that shows none of it has a window.
fn clamp(rect: [f64; 4], source: (u32, u32)) -> [u32; 4] {
  let axis = |low: f64, high: f64, length: u32| {
    let last = length.max(1);
    let low = (low.floor().max(0.0) as u32).min(last - 1);
    let high = (high.ceil().max(0.0) as u32).clamp(low + 1, last);
    (low, high)
  };
  let (x0, x1) = axis(rect[0], rect[2], source.0);
  let (y0, y1) = axis(rect[1], rect[3], source.1);
  [x0, y0, x1, y1]
}

/// `band` with each window cut to what the device takes, kept round the
/// source pixels the band itself shows.
fn fitted(mut band: Band, layers: &[LayerNeeds], limit: u32) -> Band {
  for (window, layer) in band.windows.iter_mut().zip(layers) {
    let Some(window) = window else {
      continue;
    };
    let core = clamp(shown(layer, area(&band.target)), layer.source);
    for axis in 0..2 {
      let (low, high) = (window[axis], window[axis + 2]);
      if high - low <= limit {
        continue;
      }
      let middle = (core[axis] + core[axis + 2]) / 2;
      let start = middle.saturating_sub(limit / 2).clamp(low, high - limit);
      window[axis] = start;
      window[axis + 2] = start + limit;
    }
  }
  band
}

#[cfg(test)]
mod tests;
