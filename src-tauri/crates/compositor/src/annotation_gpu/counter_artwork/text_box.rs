// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rasterisation of a text box's lines into the annotation atlas, through the
//! platform's type device.
//!
//! macOS lays a native text view over the box being typed into, which draws
//! its own caret and selection. Nothing can be laid over a DirectComposition
//! visual that way, so here the caret and the selection are drawn into the
//! box's own cell, as coverage the shader inks like the text.

use super::outline::outlined;
use super::rasterize::tinted;
use crate::platform::type_device::TypeDevice;
use screenwide_annotations::text::face::TypeFace;
use screenwide_annotations::text::metrics::LINE_HEIGHT;
use screenwide_annotations::text::typing::TypingMarks;

/// Room either side of the widest line beyond the caret, in atlas pixels: one
/// pixel keeps the shader's filtered taps on this cell. A caret after the
/// widest line needs its own width too, and both are kept on both sides so
/// the cell stays centred on the block.
const MARGIN: f64 = 1.0;
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

fn block<'a>(face: &TypeFace, text: &'a str, size: f32) -> Result<Block<'a>, String> {
  let font = f64::from(size);
  let device = TypeDevice::in_face(face.clone(), font)?;
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

/// Room around a caption's type, in ems: a face chosen from the system may
/// draw past its advance and its line, as a wide T's bar or a tall accent
/// does, and the cell would cut what reaches out. Inter stays inside both.
const FACE_ROOM: f64 = 0.25;

/// How a box's type is set beyond its text: its face, the outline drawn
/// around it, as wide as `outline` atlas pixels, or none, and the shadow a
/// caption's word casts, its edge spread over `shadow` atlas pixels.
#[derive(Clone, Copy)]
pub(super) struct Look<'a> {
  pub(super) face: &'a TypeFace,
  pub(super) outline: f64,
  pub(super) shadow: Option<f64>,
}

/// The room `look` is given on every side at `size`, in atlas pixels: its
/// outline's width and a pixel for its edge, and its shadow's softness
/// beyond that, and more for a face of the system's.
fn room(look: Look, size: f32) -> f64 {
  let face = if *look.face == TypeFace::ANNOTATION {
    0.0
  } else {
    f64::from(size) * FACE_ROOM
  };
  let outline = if look.outline > 0.0 {
    look.outline + 1.0 + look.shadow.unwrap_or(0.0)
  } else {
    0.0
  };
  face.max(outline)
}

/// The cell a box's text needs at `size`, or none for a box with nothing to
/// show: an empty box shows its caret while it is typed into, and nothing
/// once it is not. `size` and the caret's width `caret`, one drawn pixel,
/// are in atlas pixels.
pub(super) fn measure(
  look: Look,
  text: &str,
  size: f32,
  caret: f64,
  marks: Option<TypingMarks>,
) -> Result<Option<(u32, u32)>, String> {
  if size <= 1.0 {
    return Ok(None);
  }
  let block = block(look.face, text, size)?;
  if block.width <= 0.0 && marks.is_none() {
    return Ok(None);
  }
  let room = room(look, size);
  Ok(Some((
    (block.width + 2.0 * (MARGIN + caret + room)).ceil() as u32,
    (block.line * block.lines.len() as f64 + 2.0 * room).ceil() as u32 + 2,
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

/// Blends `ink` into the coverage of every pixel in the rectangle.
fn shade(
  coverage: &mut [u8],
  cell: (u32, u32),
  x: (f64, f64),
  y: (f64, f64),
  ink: impl Fn(u8) -> u8,
) {
  let columns = (x.0.round().max(0.0) as u32)..(x.1.round().min(f64::from(cell.0)) as u32);
  let rows = (y.0.round().max(0.0) as u32)..(y.1.round().min(f64::from(cell.1)) as u32);
  for row in rows {
    for column in columns.clone() {
      let at = row as usize * cell.0 as usize + column as usize;
      coverage[at] = ink(coverage[at]);
    }
  }
}

/// `text` drawn at `size` into a cell of `cell` pixels, each line placed by
/// `alignment`, with the caret, `caret` atlas pixels wide, and the selection
/// of a box being typed into; with its outline, where `look` has one.
pub(super) fn draw(
  look: Look,
  text: &str,
  size: f32,
  caret: f64,
  alignment: u32,
  marks: Option<TypingMarks>,
  cell: (u32, u32),
) -> Result<Vec<u8>, String> {
  let block = block(look.face, text, size)?;
  let (ascent, descent) = block.device.vertical_metrics();
  let share = match alignment {
    1 => 0.5,
    2 => 1.0,
    _ => 0.0,
  };
  let room = room(look, size);
  let left = |index: usize| MARGIN + caret + room + (block.width - block.widths[index]) * share;
  // Each line sits centred in its own fixed-height band, as on macOS.
  let top =
    |index: usize| 1.0 + room + block.line * index as f64 + (block.line - ascent - descent) * 0.5;
  let x_at = |index: usize, offset: usize| {
    let (start, line) = block.lines[index];
    let within = clamped(line, offset.saturating_sub(start));
    left(index) + block.device.advance(&line[..within])
  };
  let lines: Vec<((f64, f64), &str)> = block
    .lines
    .iter()
    .enumerate()
    .map(|(index, (_, line))| ((left(index), top(index)), *line))
    .collect();
  let mut coverage = block.device.draw(cell, &lines)?;
  let marks = marks.map(|marks| TypingMarks {
    start: clamped(text, marks.start),
    end: clamped(text, marks.end),
    caret: marks.caret,
  });
  // The selection goes under the glyphs: they cover it where they are drawn.
  if let Some(marks) = marks.filter(|marks| marks.start < marks.end) {
    let under = |covered: u8| {
      let behind = f64::from(255 - covered) * SELECTION_SHARE;
      covered.saturating_add(behind.round() as u8)
    };
    for (index, (start, line)) in block.lines.iter().enumerate() {
      let (from, to) = (marks.start.max(*start), marks.end.min(start + line.len()));
      if from < to {
        let band = 1.0 + block.line * index as f64;
        shade(
          &mut coverage,
          cell,
          (x_at(index, from), x_at(index, to)),
          (band, band + block.line),
          under,
        );
      }
    }
  }
  if let Some(marks) = marks.filter(|marks| marks.caret && marks.start == marks.end) {
    let index = block
      .lines
      .iter()
      .position(|(start, line)| marks.start <= start + line.len())
      .unwrap_or(block.lines.len() - 1);
    let x = x_at(index, marks.start);
    shade(
      &mut coverage,
      cell,
      (x, x + caret),
      (top(index), top(index) + ascent + descent),
      |_| 255,
    );
  }
  Ok(if look.outline > 0.0 {
    outlined(&coverage, cell, look.outline, look.shadow)
  } else {
    tinted(&coverage)
  })
}
