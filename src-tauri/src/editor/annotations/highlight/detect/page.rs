// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading one picture: its surface, its lines and its ink.

use super::ink::{Cell, Mask};
use super::lines::snap::Snap;
use super::lines::{line_at, merge_lines, Columns, Run};
use super::{
  HighlightPixels, EVEN_HEIGHT, LINE_REACH, PAD_X, PAD_Y, PAGE_SHARE, RUN_GAP, SOLID_RUN,
  SURFACE_SAMPLES, TALLEST_LINE, WORD_GAP,
};

/// One picture, as a selection reads it.
pub(super) struct Page<'a> {
  pixels: HighlightPixels<'a>,
}

impl<'a> Page<'a> {
  pub(super) fn new(pixels: HighlightPixels<'a>) -> Self {
    Self { pixels }
  }

  pub(super) fn width(&self) -> usize {
    self.pixels.width as usize
  }

  pub(super) fn height(&self) -> usize {
    self.pixels.height as usize
  }

  pub(super) fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
    let at = (y * self.width() + x) * 4;
    let rgba = &self.pixels.rgba[at..at + 4];
    [rgba[0], rgba[1], rgba[2], rgba[3]]
  }

  /// A span along one axis, grown by `grow` either side and held to `limit`.
  pub(super) fn span(a: f64, b: f64, grow: f64, limit: usize) -> Run {
    let low = (a.min(b) - grow).floor().max(0.0) as usize;
    let high = ((a.max(b) + grow).ceil().max(0.0) as usize).min(limit);
    Run {
      from: low.min(high),
      to: high,
    }
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

  /// The bands a selection from `from` to `to` covers, in picture pixels as
  /// `[left, top, right, bottom]`, or `None` where there is no text to fit.
  pub(super) fn read(
    &self,
    from: (f64, f64),
    to: (f64, f64),
    reach: f64,
    surface: [u8; 3],
  ) -> Option<Vec<[f64; 4]>> {
    let solid = (reach * SOLID_RUN).ceil() as usize;
    let mask = Mask::read(
      self,
      Self::span(from.1, to.1, reach * 2.0, self.height()),
      surface,
      solid,
    );
    // A line taller than any text the selection could be over is ink the page
    // was misread around, not a line to fit.
    let lines: Vec<Run> = self
      .lines(&mask, from, to, reach)
      .into_iter()
      .filter(|line| (line.len() as f64) <= reach * TALLEST_LINE)
      .collect();
    if lines.is_empty() {
      return None;
    }
    let near = |at: f64| line_at(&lines, at, reach);
    let (first, last, head, tail) = match (near(from.1), near(to.1)) {
      (Some(a), Some(b)) if a < b => (a, b, from, to),
      (Some(a), Some(b)) if a > b => (b, a, to, from),
      (Some(a), Some(_)) | (Some(a), None) | (None, Some(a)) => {
        let (head, tail) = if from.0 <= to.0 {
          (from, to)
        } else {
          (to, from)
        };
        (a, a, head, tail)
      }
      (None, None) => return None,
    };
    let tallest = lines[first..=last]
      .iter()
      .map(|line| line.len())
      .max()
      .unwrap_or(1) as f64;
    let word_gap = (tallest * WORD_GAP).max(2.0);
    let run_gap = (tallest * RUN_GAP).max(word_gap * 3.0);
    let along = Self::span(from.0, to.0, tallest * LINE_REACH, self.width());
    let columns = |line: Run| Columns::read(&mask, line, along);
    let least = tallest * EVEN_HEIGHT;
    let pad_x = (tallest * PAD_X).max(1.0);
    let pad_y = (tallest * PAD_Y).max(1.0);
    // A band reaches past the ink at a word's edge, but a cut through a word
    // stops where it was let go, short of the letters beyond.
    let left_of = |snap: Snap| {
      if snap.cut {
        snap.at as f64
      } else {
        snap.at as f64 - pad_x
      }
    };
    let right_of = |snap: Snap| {
      if snap.cut {
        snap.at as f64
      } else {
        (snap.at + 1) as f64 + pad_x
      }
    };

    let mut spans: Vec<(Run, f64, f64)> = Vec::new();
    if first == last {
      let row = columns(lines[first]);
      let left = left_of(row.snap_start(head.0, tail.0, run_gap));
      let right = right_of(row.snap_end(tail.0, head.0, run_gap));
      let (left, right) = if left < right {
        (left, right)
      } else {
        (
          left_of(Snap::edge(row.clamp(head.0))),
          right_of(Snap::edge(row.clamp(tail.0))),
        )
      };
      spans.push((lines[first], left, right));
    } else {
      // The stretch the drag covered across: a gap inside it - between the
      // cells of a table's row, say - is crossed, and one beyond it still ends
      // a line, so a selection down one column of text keeps to it.
      let (span_left, span_right) = (from.0.min(to.0), from.0.max(to.0));
      let head_row = columns(lines[first]);
      let head_left = head_row.snap_start(head.0, f64::MAX, run_gap);
      let right = head_row
        .run_right(head_left.ink, run_gap, head_row.clamp(span_right))
        .unwrap_or(head_left.at);
      spans.push((
        lines[first],
        left_of(head_left),
        right_of(Snap::edge(right)),
      ));
      let tail_row = columns(lines[last]);
      let tail_right = tail_row.snap_end(tail.0, 0.0, run_gap);
      let tail_left = tail_row
        .run_left(tail_right.ink, run_gap, tail_row.clamp(span_left))
        .unwrap_or(tail_right.at);
      // Every line between starts where its paragraph does, which is where
      // the last line starts.
      for &line in &lines[first + 1..last] {
        if (line.len() as f64) < tallest * 0.35 {
          // A rule or an underline between the lines rather than text.
          continue;
        }
        let row = columns(line);
        let Some(seed) = row.nearest(tail_left.min(head_left.at) as f64, run_gap * 2.0) else {
          continue;
        };
        let middle_left = row
          .run_left(seed, run_gap, row.clamp(span_left))
          .unwrap_or(seed);
        let middle_right = row
          .run_right(seed, run_gap, row.clamp(span_right))
          .unwrap_or(seed);
        spans.push((
          line,
          left_of(Snap::edge(middle_left)),
          right_of(Snap::edge(middle_right)),
        ));
      }
      spans.push((
        lines[last],
        left_of(Snap::edge(tail_left)),
        right_of(tail_right),
      ));
    }

    Some(
      spans
        .into_iter()
        .map(|(line, left, right)| {
          let mut top = line.from as f64;
          let mut bottom = line.to as f64;
          let short = least - (bottom - top);
          if short > 0.0 {
            // A line without ascenders is missing its top more often than a
            // line without descenders is missing its bottom.
            top -= short * 0.7;
            bottom += short * 0.3;
          }
          [left, top - pad_y, right, bottom + pad_y]
        })
        .collect(),
    )
  }

  /// The runs of rows that carry text across the drag, top to bottom. Only
  /// the columns the drag covers are read, and half a band either side of
  /// them: a picture beside the text it started on is not part of its lines.
  fn lines(&self, mask: &Mask, from: (f64, f64), to: (f64, f64), reach: f64) -> Vec<Run> {
    let columns = Self::span(from.0, to.0, reach * 0.5, self.width());
    let rows = Self::span(from.1, to.1, reach * 2.0, self.height());
    if columns.len() == 0 || rows.len() == 0 {
      return Vec::new();
    }
    let inked: Vec<bool> = (rows.from..rows.to)
      .map(|y| mask.row(y, columns).any(|cell| cell == Cell::Glyph))
      .collect();
    let mut runs = Vec::new();
    let mut open: Option<usize> = None;
    for (index, &ink) in inked.iter().enumerate() {
      match (ink, open) {
        (true, None) => open = Some(index),
        (false, Some(begin)) => {
          runs.push(Run {
            from: rows.from + begin,
            to: rows.from + index,
          });
          open = None;
        }
        _ => {}
      }
    }
    if let Some(begin) = open {
      runs.push(Run {
        from: rows.from + begin,
        to: rows.to,
      });
    }
    merge_lines(runs)
  }
}
