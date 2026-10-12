// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{raster, Appearance, CursorStyle, APPEARANCE_STABILITY_US};

pub(super) fn stable_appearances(
  appearances: &[Appearance],
  recording_end_us: u64,
) -> Vec<Appearance> {
  let mut changes = Vec::new();
  for appearance in appearances {
    if changes.last().is_some_and(|previous: &Appearance| {
      raster::uses_same_artwork(previous.style, appearance.style)
    }) {
      continue;
    }
    changes.push(*appearance);
  }
  let Some(first) = changes.first().copied() else {
    return Vec::new();
  };
  let mut stable = vec![first];
  for (index, appearance) in changes.iter().enumerate().skip(1) {
    let end_us = changes
      .get(index + 1)
      .map_or(recording_end_us, |next| next.timestamp_us);
    if end_us.saturating_sub(appearance.timestamp_us) < APPEARANCE_STABILITY_US {
      continue;
    }
    if stable
      .last()
      .is_none_or(|previous| !raster::uses_same_artwork(previous.style, appearance.style))
    {
      stable.push(*appearance);
    }
  }
  stable
}

/// Custom cursor pixels are not recorded yet, so they render with the arrow
/// artwork. Give that stand-in the recording's arrow dimensions too: using a
/// custom cursor's unrelated bitmap box makes the same arrow visibly jump in
/// size even though the user's percentage setting has not changed.
pub(super) fn normalize_custom_fallback_size(appearances: &mut [Appearance]) {
  let Some((width, height)) = appearances
    .iter()
    .find(|appearance| appearance.style == CursorStyle::Arrow)
    .map(|appearance| (appearance.width, appearance.height))
  else {
    return;
  };
  for appearance in appearances
    .iter_mut()
    .filter(|appearance| appearance.style == CursorStyle::Custom)
  {
    appearance.width = width;
    appearance.height = height;
  }
}

/// A cursor recognised by its shape may have been drawn in another design
/// than the system artwork it is drawn with here - an application's classic
/// 9x18 I-beam, say, drawn with the system's 23x22 one - and its recorded box
/// is that other design's. Stretched over it, the artwork takes the other
/// design's proportions and comes out squashed. Where the recorded box is not
/// the artwork's own at any scale, the artwork is drawn at its own size about
/// its own hotspot instead; a box that is the artwork's at some scale is kept,
/// scale and all. `natural` is each style's own box: width, height and
/// hotspot, in points.
pub(super) fn fit_to_artwork(
  appearances: &mut [Appearance],
  natural: impl Fn(CursorStyle) -> Option<[f64; 4]>,
) {
  for appearance in appearances.iter_mut() {
    let Some([width, height, hotspot_x, hotspot_y]) = natural(appearance.style) else {
      continue;
    };
    let proportions = |wide: f64, high: f64| wide / high;
    let same_design = appearance.height > 0.0
      && (proportions(appearance.width, appearance.height) / proportions(width, height) - 1.0)
        .abs()
        <= 0.05;
    if !same_design {
      (
        appearance.width,
        appearance.height,
        appearance.hotspot_x,
        appearance.hotspot_y,
      ) = (width, height, hotspot_x, hotspot_y);
    }
  }
}
