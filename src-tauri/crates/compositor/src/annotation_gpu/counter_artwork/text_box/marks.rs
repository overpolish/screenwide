// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The caret and selection of a box being typed into, drawn into its cell
//! after its type, its outline and its shadow, which are the type's alone.

use super::{clamped, Look, Placed};
use screenwide_text::engine::typing::TypingMarks;

/// How much ink a selection lays over what it covers: the share macOS's text
/// view highlights with.
const SELECTION_SHARE: f64 = 0.28;

/// Blends `ink` into the type's coverage of every pixel in the rectangle of
/// `pixels`, the cell's BGRA. The outline's coverage, in the red, carries the
/// type's too, so it is raised to match.
fn shade(
  pixels: &mut [u8],
  cell: (u32, u32),
  x: (f64, f64),
  y: (f64, f64),
  ink: impl Fn(u8) -> u8,
) {
  let columns = (x.0.round().max(0.0) as u32)..(x.1.round().min(f64::from(cell.0)) as u32);
  let rows = (y.0.round().max(0.0) as u32)..(y.1.round().min(f64::from(cell.1)) as u32);
  for row in rows {
    for column in columns.clone() {
      let at = (row as usize * cell.0 as usize + column as usize) * 4;
      let covered = ink(pixels[at + 3]);
      pixels[at + 3] = covered;
      pixels[at + 2] = pixels[at + 2].max(covered);
    }
  }
}

/// Draws the caret and selection `marks` into `pixels`, the cell
/// [`draw`](super::draw) drew the same text into with the same arguments.
/// Neither is outlined nor casts a shadow: the caret is a plain bar in the
/// type's colour.
#[allow(clippy::too_many_arguments)]
pub(in super::super) fn mark(
  pixels: &mut [u8],
  look: Look,
  text: &str,
  size: f32,
  caret: f64,
  alignment: u32,
  marks: TypingMarks,
  cell: (u32, u32),
) -> Result<(), String> {
  let placed = Placed::new(look, text, size, caret, alignment)?;
  let lines = &placed.block.lines;
  let (start, end) = (clamped(text, marks.start), clamped(text, marks.end));
  // The selection goes under the glyphs: they cover it where they are drawn.
  if start < end {
    let under = |covered: u8| {
      let behind = f64::from(255 - covered) * SELECTION_SHARE;
      covered.saturating_add(behind.round() as u8)
    };
    for (index, (line_start, line)) in lines.iter().enumerate() {
      let (from, to) = (start.max(*line_start), end.min(line_start + line.len()));
      if from < to {
        let band = placed.band(index);
        shade(
          pixels,
          cell,
          (placed.x_at(index, from), placed.x_at(index, to)),
          (band, band + placed.block.line),
          under,
        );
      }
    }
  }
  if marks.caret && start == end {
    let index = lines
      .iter()
      .position(|(line_start, line)| start <= line_start + line.len())
      .unwrap_or(lines.len() - 1);
    let x = placed.x_at(index, start);
    let top = placed.top(index);
    shade(
      pixels,
      cell,
      (x, x + caret),
      (top, top + placed.ascent + placed.descent),
      |_| 255,
    );
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::super::{draw, measure};
  use super::*;
  use screenwide_text::engine::face::TypeFace;

  /// The caret on outlined, shadowed type is a bare bar: it changes no pixel
  /// beyond its own, so it has no outline and casts no shadow.
  #[test]
  fn a_caret_on_outlined_type_is_not_outlined() {
    let face = TypeFace::ANNOTATION;
    let look = Look {
      face: &face,
      outline: 6.0,
      shadow: Some(4.0),
    };
    let marks = TypingMarks {
      start: 2,
      end: 2,
      caret: true,
    };
    let cell = measure(look, "Hi", 40.0, 2.0, Some(marks))
      .unwrap()
      .unwrap();
    let base = draw(look, "Hi", 40.0, 2.0, 0, cell).unwrap();
    let mut marked = base.clone();
    mark(&mut marked, look, "Hi", 40.0, 2.0, 0, marks, cell).unwrap();
    let changed: Vec<usize> = (0..base.len() / 4)
      .filter(|&pixel| base[pixel * 4..pixel * 4 + 4] != marked[pixel * 4..pixel * 4 + 4])
      .collect();
    assert!(!changed.is_empty());
    let width = cell.0 as usize;
    let columns: Vec<usize> = changed.iter().map(|pixel| pixel % width).collect();
    let span = columns.iter().max().unwrap() - columns.iter().min().unwrap();
    assert!(span < 2, "the caret is two pixels wide, not {}", span + 1);
    assert!(changed.iter().all(|&pixel| marked[pixel * 4 + 3] == 255));
  }
}
