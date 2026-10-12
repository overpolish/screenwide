// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The page a selection's text is printed on.

use super::page::Page;
use super::{INK_DISTANCE, PAGE_SHARE, SURFACE_SAMPLES};

impl Page<'_> {
  /// The page a selection from `press` to `release` lies on. Each end reads
  /// the page around it; where they read different ones - a drag pressed on a
  /// label or a cell's colour and let go on the page it sits on - the page is
  /// the one most of the stretch the drag spans is. Where only one end finds a
  /// page, that end's; `None` where neither does.
  pub(super) fn surface_of(
    &self,
    press: (f64, f64),
    release: (f64, f64),
    reach: f64,
  ) -> Option<[u8; 3]> {
    let (pressed, released) = match (
      self.surface_at(press, reach),
      self.surface_at(release, reach),
    ) {
      (Some(pressed), Some(released)) => (pressed, released),
      (pressed, released) => return pressed.or(released),
    };
    let alike = |a: [u8; 3], b: [u8; 3]| {
      a.iter()
        .zip(b)
        .all(|(a, b)| (i32::from(*a) - i32::from(b)).abs() <= INK_DISTANCE)
    };
    if alike(pressed, released) {
      return Some(pressed);
    }
    let columns = Self::span(press.0, release.0, reach * 4.0, self.width());
    let rows = Self::span(press.1, release.1, reach, self.height());
    let area = (columns.len() * rows.len()) as f64;
    let step = ((area / SURFACE_SAMPLES).sqrt().ceil() as usize).max(1);
    let (mut on_pressed, mut on_released) = (0_usize, 0_usize);
    for y in (rows.from..rows.to).step_by(step) {
      for x in (columns.from..columns.to).step_by(step) {
        let [r, g, b, a] = self.pixel(x, y);
        if a < 128 {
          continue;
        }
        on_pressed += usize::from(alike([r, g, b], pressed));
        on_released += usize::from(alike([r, g, b], released));
      }
    }
    Some(if on_released > on_pressed {
      released
    } else {
      pressed
    })
  }

  /// The page the selection was pressed on: the colour most of the
  /// neighbourhood of the press is. Colours are counted at five bits a
  /// channel, and the page is the mean of the pixels in the fullest bucket.
  ///
  /// Read at the press rather than across the whole drag, so a pane or a
  /// picture the drag runs into cannot outvote the page it started on. `None`
  /// where no colour is most of it - a photo, a gradient - and there is no
  /// page for text to sit on.
  pub(super) fn surface_at(&self, press: (f64, f64), reach: f64) -> Option<[u8; 3]> {
    let columns = Self::span(press.0, press.0, reach * 4.0, self.width());
    let rows = Self::span(press.1, press.1, reach, self.height());
    let area = (columns.len() * rows.len()) as f64;
    let step = ((area / SURFACE_SAMPLES).sqrt().ceil() as usize).max(1);
    let mut counts = vec![0_u32; 1 << 15];
    let mut sums = vec![[0_u32; 3]; 1 << 15];
    for y in (rows.from..rows.to).step_by(step) {
      for x in (columns.from..columns.to).step_by(step) {
        let [r, g, b, a] = self.pixel(x, y);
        if a < 128 {
          continue;
        }
        let bucket = (usize::from(r >> 3) << 10) | (usize::from(g >> 3) << 5) | usize::from(b >> 3);
        counts[bucket] += 1;
        let sum = &mut sums[bucket];
        sum[0] += u32::from(r);
        sum[1] += u32::from(g);
        sum[2] += u32::from(b);
      }
    }
    let (bucket, &count) = counts
      .iter()
      .enumerate()
      .max_by_key(|(_, count)| **count)
      .filter(|(_, count)| **count > 0)?;
    let sum = sums[bucket];
    let surface = [
      (sum[0] / count) as u8,
      (sum[1] / count) as u8,
      (sum[2] / count) as u8,
    ];
    let (mut page, mut seen) = (0_usize, 0_usize);
    for y in (rows.from..rows.to).step_by(step) {
      for x in (columns.from..columns.to).step_by(step) {
        if self.pixel(x, y)[3] < 128 {
          continue;
        }
        seen += 1;
        if !self.is_ink(x, y, surface) {
          page += 1;
        }
      }
    }
    (page as f64 >= seen as f64 * PAGE_SHARE).then_some(surface)
  }
}
