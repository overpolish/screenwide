// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The colour of the surface a region sits on, voted from shallow rings of
//! pixels. Recenter reads the rings just inside a picture's edge to match the
//! inset it pads with; a redaction reads the rings just outside its box so an
//! erased region takes the colour of what surrounds it.

use std::collections::BTreeMap;

/// A straight run of pixels, both ends included.
type Segment = ((u32, u32), (u32, u32));

/// How far outside a box its rings sit, in pixels. Several shallow rings
/// tolerate a hairline border or a shadow hugging the box.
const SURROUNDING_RINGS: [u32; 3] = [1, 2, 4];

/// Match the surface touching the inset, rather than averaging the interior
/// controls into it. Several shallow rings tolerate a thin border or shadow;
/// bounded samples on each side also give short edges an equal vote.
pub(crate) fn inset_colour(rgba: &[u8], width: u32, height: u32) -> Option<[u8; 3]> {
  if !covers(rgba, width, height) {
    return None;
  }
  let depth = (width.min(height) / 2).saturating_sub(1).min(4);
  let mut depths = vec![0, depth / 2, depth];
  depths.dedup();
  let mut segments = Vec::with_capacity(depths.len() * 4);
  for inset in depths {
    let (left, top) = (inset, inset);
    let (right, bottom) = (width - 1 - inset, height - 1 - inset);
    segments.extend([
      ((left, top), (right, top)),
      ((left, bottom), (right, bottom)),
      ((left, top), (left, bottom)),
      ((right, top), (right, bottom)),
    ]);
  }
  dominant(rgba, width, &segments)
}

/// The surface around the box `[x0, y0, x1, y1)`, read only from pixels
/// outside it. A side that would fall off the picture is left out rather than
/// clamped back in: clamping would read the covered pixels along that edge.
/// `None` where nothing opaque surrounds the box.
pub(crate) fn surrounding_colour(
  rgba: &[u8],
  width: u32,
  height: u32,
  bounds: [u32; 4],
) -> Option<[u8; 3]> {
  if !covers(rgba, width, height) {
    return None;
  }
  let [x0, y0, x1, y1] = bounds.map(i64::from);
  let (last_x, last_y) = (i64::from(width) - 1, i64::from(height) - 1);
  let mut segments = Vec::with_capacity(SURROUNDING_RINGS.len() * 4);
  for distance in SURROUNDING_RINGS.map(i64::from) {
    let (left, top) = (x0 - distance, y0 - distance);
    let (right, bottom) = (x1 - 1 + distance, y1 - 1 + distance);
    let (from_x, to_x) = (left.max(0), right.min(last_x));
    let (from_y, to_y) = (top.max(0), bottom.min(last_y));
    let point = |x: i64, y: i64| (x as u32, y as u32);
    if from_x <= to_x {
      if top >= 0 {
        segments.push((point(from_x, top), point(to_x, top)));
      }
      if bottom <= last_y {
        segments.push((point(from_x, bottom), point(to_x, bottom)));
      }
    }
    if from_y <= to_y {
      if left >= 0 {
        segments.push((point(left, from_y), point(left, to_y)));
      }
      if right <= last_x {
        segments.push((point(right, from_y), point(right, to_y)));
      }
    }
  }
  dominant(rgba, width, &segments)
}

fn covers(rgba: &[u8], width: u32, height: u32) -> bool {
  let expected = width
    .checked_mul(height)
    .and_then(|pixels| pixels.checked_mul(4))
    .and_then(|bytes| usize::try_from(bytes).ok());
  width > 0 && height > 0 && expected.is_some_and(|expected| rgba.len() >= expected)
}

/// The most supported opaque colour along `segments`, each sampled at no more
/// than 256 evenly spaced points.
fn dominant(rgba: &[u8], width: u32, segments: &[Segment]) -> Option<[u8; 3]> {
  let mut colours = BTreeMap::<[u8; 3], u32>::new();
  for &(start, end) in segments {
    let length = (end.0 - start.0).max(end.1 - start.1);
    let steps = length.clamp(1, 255);
    for step in 0..=steps {
      let x = start.0 + (u64::from(end.0 - start.0) * u64::from(step) / u64::from(steps)) as u32;
      let y = start.1 + (u64::from(end.1 - start.1) * u64::from(step) / u64::from(steps)) as u32;
      let offset = ((y * width + x) * 4) as usize;
      if rgba[offset + 3] != 255 {
        continue;
      }
      let colour = [rgba[offset], rgba[offset + 1], rgba[offset + 2]];
      *colours.entry(colour).or_default() += 1;
    }
  }
  // Count nearby colours across arbitrary bucket boundaries, but return an
  // actual sampled colour. Never blend neighbouring dark UI surfaces together.
  colours.keys().copied().max_by_key(|colour| {
    let mut support = 0;
    for r in colour[0].saturating_sub(2)..=colour[0].saturating_add(2) {
      for g in colour[1].saturating_sub(2)..=colour[1].saturating_add(2) {
        for b in colour[2].saturating_sub(2)..=colour[2].saturating_add(2) {
          support += colours.get(&[r, g, b]).copied().unwrap_or(0);
        }
      }
    }
    (support, colours[colour], std::cmp::Reverse(*colour))
  })
}

#[cfg(test)]
#[path = "surface_colour_tests.rs"]
mod tests;
