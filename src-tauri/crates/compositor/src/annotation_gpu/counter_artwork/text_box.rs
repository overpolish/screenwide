// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rasterisation of a text box's lines into the annotation atlas, through the
//! platform's type device. A box being typed into has its caret and selection
//! drawn into the same cell, as coverage the shader inks like the text, so
//! they turn and tilt with the box on both platforms.

use super::outline::outlined;
use super::rasterize::tinted;
use crate::platform::type_device::TypeDevice;
use screenwide_text::engine::face::TypeFace;
use screenwide_text::engine::metrics::LINE_HEIGHT;
use screenwide_text::engine::typing::TypingMarks;

mod marks;
pub(super) use marks::mark;

/// Room either side of the widest line beyond the caret, in atlas pixels: one
/// pixel keeps the shader's filtered taps on this cell. A caret after the
/// widest line needs its own width too, and both are kept on both sides so
/// the cell stays centred on the block.
const MARGIN: f64 = 1.0;

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

/// Where a box's lines sit in its cell, each placed by its alignment, with
/// room for the caret, `caret` atlas pixels wide, beside the widest.
struct Placed<'a> {
  block: Block<'a>,
  ascent: f64,
  descent: f64,
  share: f64,
  room: f64,
  caret: f64,
}

impl<'a> Placed<'a> {
  fn new(look: Look, text: &'a str, size: f32, caret: f64, alignment: u32) -> Result<Self, String> {
    let block = block(look.face, text, size)?;
    let (ascent, descent) = block.device.vertical_metrics();
    let share = match alignment {
      1 => 0.5,
      2 => 1.0,
      _ => 0.0,
    };
    Ok(Self {
      block,
      ascent,
      descent,
      share,
      room: room(look, size),
      caret,
    })
  }

  fn left(&self, index: usize) -> f64 {
    MARGIN + self.caret + self.room + (self.block.width - self.block.widths[index]) * self.share
  }

  /// The top of line `index`'s fixed-height band.
  fn band(&self, index: usize) -> f64 {
    1.0 + self.room + self.block.line * index as f64
  }

  /// The top of line `index`'s type, centred in its band, as on macOS.
  fn top(&self, index: usize) -> f64 {
    self.band(index) + (self.block.line - self.ascent - self.descent) * 0.5
  }

  fn x_at(&self, index: usize, offset: usize) -> f64 {
    let (start, line) = self.block.lines[index];
    let within = clamped(line, offset.saturating_sub(start));
    self.left(index) + self.block.device.advance(&line[..within])
  }
}

/// `text` drawn at `size` into a cell of `cell` pixels as the atlas's BGRA,
/// each line placed by `alignment`, with room for a caret `caret` atlas
/// pixels wide; with its outline, where `look` has one.
pub(super) fn draw(
  look: Look,
  text: &str,
  size: f32,
  caret: f64,
  alignment: u32,
  cell: (u32, u32),
) -> Result<Vec<u8>, String> {
  let placed = Placed::new(look, text, size, caret, alignment)?;
  let lines: Vec<((f64, f64), &str)> = placed
    .block
    .lines
    .iter()
    .enumerate()
    .map(|(index, (_, line))| ((placed.left(index), placed.top(index)), *line))
    .collect();
  let coverage = placed.block.device.draw(cell, &lines)?;
  Ok(if look.outline > 0.0 {
    outlined(&coverage, cell, look.outline, look.shadow)
  } else {
    tinted(&coverage)
  })
}
