// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which pixels are ink, which ink is text, and how bright the two are.
//!
//! Ink is whatever stands far enough from the page. Text is ink drawn in
//! strokes: a letter's runs of ink are short, across and down. An avatar, an
//! icon on its tile, a neighbouring pane, a window's edge or a table's rule
//! runs on one way or the other, and read as text it fuses separate lines into
//! one and carries a line's words across into whatever sits beside them.

use super::lines::Run;
use super::page::Page;
use super::{apart, luma, INK_DISTANCE};
use crate::editor::annotations::highlight::model::{HighlightBand, HighlightTone};

/// What one pixel is to a reader.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Cell {
  Page,
  /// Ink in a stroke a letter could make.
  Glyph,
  /// Ink in a stretch too long for a letter: a picture, a pane, a bar or a
  /// rule.
  Solid,
}

/// Every pixel across the picture over the rows a selection reads, as a
/// reader sees it. Built once a selection, so finding the lines and walking
/// along them read the same cells.
pub(super) struct Mask {
  width: usize,
  rows: Run,
  cells: Vec<Cell>,
}

impl Mask {
  /// `rows` of `page`, ink measured against `surface`. A run of ink longer
  /// than `solid` either way is solid; runs are measured past the rows read,
  /// so a rule the window cuts into is still a rule.
  pub(super) fn read(page: &Page<'_>, rows: Run, surface: [u8; 3], solid: usize) -> Self {
    let width = page.width();
    let scan = Run {
      from: rows.from.saturating_sub(solid),
      to: (rows.to + solid).min(page.height()),
    };
    let ink: Vec<bool> = (scan.from..scan.to)
      .flat_map(|y| (0..width).map(move |x| (x, y)))
      .map(|(x, y)| page.is_ink(x, y, surface))
      .collect();
    let mut long = vec![false; ink.len()];
    for row in 0..scan.len() {
      mark_long_runs(&ink, &mut long, row * width, 1, width, solid);
    }
    for column in 0..width {
      mark_long_runs(&ink, &mut long, column, width, scan.len(), solid);
    }
    let skip = (rows.from - scan.from) * width;
    let cells = ink
      .iter()
      .zip(&long)
      .skip(skip)
      .take(rows.len() * width)
      .map(|(&ink, &long)| match (ink, long) {
        (false, _) => Cell::Page,
        (true, false) => Cell::Glyph,
        (true, true) => Cell::Solid,
      })
      .collect();
    Self { width, rows, cells }
  }

  /// Row `y` across `columns`, the page wherever the mask was not read.
  pub(super) fn row(&self, y: usize, columns: Run) -> impl Iterator<Item = Cell> + '_ {
    let row = (y >= self.rows.from && y < self.rows.to).then(|| (y - self.rows.from) * self.width);
    (columns.from..columns.to).map(move |x| match row {
      Some(row) if x < self.width => self.cells[row + x],
      _ => Cell::Page,
    })
  }
}

/// Marks in `long` every run of `ink` longer than `solid` along the `count`
/// cells from `start`, `stride` apart.
fn mark_long_runs(
  ink: &[bool],
  long: &mut [bool],
  start: usize,
  stride: usize,
  count: usize,
  solid: usize,
) {
  let mut run = 0;
  for step in 0..=count {
    if step < count && ink[start + step * stride] {
      run += 1;
      continue;
    }
    if run > solid {
      for back in step - run..step {
        long[start + back * stride] = true;
      }
    }
    run = 0;
  }
}

impl Page<'_> {
  pub(super) fn is_ink(&self, x: usize, y: usize, surface: [u8; 3]) -> bool {
    let [r, g, b, a] = self.pixel(x, y);
    if a < 128 {
      return false;
    }
    let far = |channel: u8, page: u8| (i32::from(channel) - i32::from(page)).abs() > INK_DISTANCE;
    far(r, surface[0]) || far(g, surface[1]) || far(b, surface[2])
  }

  /// How bright the page and its ink are. The ink is read from the pixels
  /// inside the bands that stand out from the surface, at the strong end of
  /// how far they stand out, so anti-aliased edges do not pull it towards the
  /// page.
  pub(super) fn tone(
    &self,
    surface: [u8; 3],
    bands: &[HighlightBand],
    scale: (f64, f64),
  ) -> HighlightTone {
    let page = luma(surface.map(|channel| f64::from(channel) / 255.0));
    let mut inks: Vec<f64> = Vec::new();
    for band in bands {
      let columns = Self::span(band.left * scale.0, band.right * scale.0, 0.0, self.width());
      let rows = Self::span(
        band.top * scale.1,
        band.bottom * scale.1,
        0.0,
        self.height(),
      );
      let area = (columns.len() * rows.len()) as f64;
      let step = ((area / 20_000.0).sqrt().ceil() as usize).max(1);
      for y in (rows.from..rows.to).step_by(step) {
        for x in (columns.from..columns.to).step_by(step) {
          if self.is_ink(x, y, surface) {
            let [r, g, b, _] = self.pixel(x, y);
            inks.push(luma([r, g, b].map(|channel| f64::from(channel) / 255.0)));
          }
        }
      }
    }
    inks.sort_by(|a, b| (a - page).abs().total_cmp(&(b - page).abs()));
    let ink = inks
      .get(inks.len() * 4 / 5)
      .copied()
      .unwrap_or(if page > 0.5 { 0.0 } else { 1.0 });
    HighlightTone {
      surface: page,
      ink: apart(page, ink),
    }
  }
}
