// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;

/// Match the surface touching the inset, rather than averaging the interior
/// controls into it. Several shallow rings tolerate a thin border or shadow;
/// bounded samples on each side also give short edges an equal vote.
pub(super) fn detect(rgba: &[u8], width: u32, height: u32) -> Option<[u8; 3]> {
  let expected = usize::try_from(width.checked_mul(height)?.checked_mul(4)?).ok()?;
  if width == 0 || height == 0 || rgba.len() < expected {
    return None;
  }
  let mut colours = BTreeMap::<[u8; 3], u32>::new();
  let depth = (width.min(height) / 2).saturating_sub(1).min(4);
  let mut depths = vec![0, depth / 2, depth];
  depths.dedup();
  for inset in depths {
    let left = inset;
    let top = inset;
    let right = width - 1 - inset;
    let bottom = height - 1 - inset;
    for (start, end) in [
      ((left, top), (right, top)),
      ((left, bottom), (right, bottom)),
      ((left, top), (left, bottom)),
      ((right, top), (right, bottom)),
    ] {
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
#[path = "recenter_background_tests.rs"]
mod tests;
