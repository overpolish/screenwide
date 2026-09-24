// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! GDI rasterisation of a text box's lines, the twin of
//! `screenwide_text_box_draw` in `gpu_compositor_macos_annotation_text_box.m`.
//!
//! macOS lays a native text view over the box being typed into, which draws
//! its own caret and selection. Nothing can be laid over a DirectComposition
//! visual that way, so here the caret and the selection are drawn into the
//! box's own cell, as coverage the shader inks like the text.

use super::rasterize::raster_cell;
use super::*;
use crate::editor::annotations::text::metrics::LINE_HEIGHT;
use crate::editor::annotations::text::typing::TypingMarks;
use crate::editor::preview_platform::surface::type_device::TypeDevice;

/// The caret's width, in atlas pixels: one drawn pixel.
const CARET_WIDTH: f64 = SUPERSAMPLE;
/// Room either side of the widest line, in atlas pixels: one pixel keeps a
/// four-tap sample on this cell, and a caret after the widest line needs its
/// own width. Kept on both sides so the cell stays centred on the block.
const MARGIN: f64 = 1.0 + CARET_WIDTH;
/// How much ink a selection lays over what it covers: the share macOS's text
/// view highlights with.
const SELECTION_SHARE: f64 = 0.28;

/// The lines at one size in atlas pixels, and how wide each is.
struct Block<'a> {
  device: TypeDevice,
  lines: Vec<(usize, &'a str)>,
  widths: Vec<f64>,
  width: f64,
  line: f64,
}

fn block(text: &str, size: f32) -> Result<Block<'_>, String> {
  let font = f64::from(size) * SUPERSAMPLE;
  let device = TypeDevice::new(font)?;
  let mut offset = 0;
  let lines: Vec<(usize, &str)> = text
    .split('\n')
    .map(|line| {
      let start = offset;
      offset += line.len() + 1;
      (start, line)
    })
    .collect();
  let widths: Vec<f64> = lines.iter().map(|(_, line)| device.advance(line)).collect();
  let width = widths.iter().copied().fold(0.0, f64::max);
  Ok(Block {
    device,
    lines,
    widths,
    width,
    line: font * LINE_HEIGHT,
  })
}

/// The cell a box's text needs at `size`, or none for a box with nothing to
/// show: an empty box shows its caret while it is typed into, and nothing
/// once it is not.
pub(super) fn measure(
  text: &str,
  size: f32,
  marks: Option<TypingMarks>,
) -> Result<Option<(u32, u32)>, String> {
  if size <= 1.0 {
    return Ok(None);
  }
  let block = block(text, size)?;
  if block.width <= 0.0 && marks.is_none() {
    return Ok(None);
  }
  Ok(Some((
    (block.width + 2.0 * MARGIN).ceil() as u32,
    (block.line * block.lines.len() as f64).ceil() as u32 + 2,
  )))
}

/// The largest character boundary at or before `offset`: the marks follow
/// the typing, which can be a frame ahead of the text being drawn.
fn clamped(text: &str, offset: usize) -> usize {
  let mut offset = offset.min(text.len());
  while !text.is_char_boundary(offset) {
    offset -= 1;
  }
  offset
}

/// Lays `value` over the coverage of every pixel in the rectangle.
fn shade(raster: &mut [u8], cell: (u32, u32), x: (f64, f64), y: (f64, f64), value: u8) {
  let columns = (x.0.round().max(0.0) as u32)..(x.1.round().min(f64::from(cell.0)) as u32);
  let rows = (y.0.round().max(0.0) as u32)..(y.1.round().min(f64::from(cell.1)) as u32);
  for row in rows {
    for column in columns.clone() {
      let at = (row as usize * cell.0 as usize + column as usize) * 4 + 2;
      raster[at] = raster[at].max(value);
    }
  }
}

/// `text` drawn at `size` into a cell of `cell` pixels, each line placed by
/// `alignment`, with the caret and the selection of a box being typed into.
pub(super) fn draw(
  text: &str,
  size: f32,
  alignment: u32,
  marks: Option<TypingMarks>,
  cell: (u32, u32),
) -> Result<Vec<u8>, String> {
  let block = block(text, size)?;
  let (ascent, descent) = block.device.vertical_metrics();
  let share = match alignment {
    1 => 0.5,
    2 => 1.0,
    _ => 0.0,
  };
  let left = |index: usize| MARGIN + (block.width - block.widths[index]) * share;
  // Each line sits centred in its own fixed-height band, as on macOS.
  let top = |index: usize| 1.0 + block.line * index as f64 + (block.line - ascent - descent) * 0.5;
  let x_at = |index: usize, offset: usize| {
    let (start, line) = block.lines[index];
    let within = clamped(line, offset.saturating_sub(start));
    left(index) + block.device.advance(&line[..within])
  };
  raster_cell(&block.device, cell, |dc, raster| {
    let marks = marks.map(|marks| TypingMarks {
      start: clamped(text, marks.start),
      end: clamped(text, marks.end),
      caret: marks.caret,
    });
    // The selection goes under the glyphs, which GDI blends over it.
    if let Some(marks) = marks.filter(|marks| marks.start < marks.end) {
      for (index, (start, line)) in block.lines.iter().enumerate() {
        let (from, to) = (marks.start.max(*start), marks.end.min(start + line.len()));
        if from < to {
          let band = 1.0 + block.line * index as f64;
          shade(
            raster,
            cell,
            (x_at(index, from), x_at(index, to)),
            (band, band + block.line),
            (SELECTION_SHARE * 255.0).round() as u8,
          );
        }
      }
    }
    let mut drawn = true;
    for (index, (_, line)) in block.lines.iter().enumerate() {
      if line.is_empty() {
        continue;
      }
      let wide: Vec<u16> = line.encode_utf16().collect();
      let (x, y) = (left(index).round() as i32, top(index).round() as i32);
      drawn &= unsafe { TextOutW(dc, x, y, &wide) }.as_bool();
    }
    if let Some(marks) = marks.filter(|marks| marks.caret && marks.start == marks.end) {
      let index = block
        .lines
        .iter()
        .position(|(start, line)| marks.start <= start + line.len())
        .unwrap_or(block.lines.len() - 1);
      let x = x_at(index, marks.start);
      shade(
        raster,
        cell,
        (x, x + CARET_WIDTH),
        (top(index), top(index) + ascent + descent),
        255,
      );
    }
    drawn
  })
}
