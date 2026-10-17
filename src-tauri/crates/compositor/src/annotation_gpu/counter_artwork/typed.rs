// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The box being typed into. Its atlas key leaves out its caret and selection,
//! so its cell keeps its place while they move; that one cell is drawn again
//! when they do, over its type as last rasterised. A drag or a blink then
//! costs no new layout, no other cell and no new rasterisation of the type,
//! which with an outline and a shadow is the slow part.

use screenwide_annotations::counter::atlas::{AtlasDraw, AtlasRect};
use screenwide_text::engine::typing::TypingMarks;

use super::{look, text_box, PreparedType};

/// The typed box's type without its marks, for one key, size and cell.
struct Base {
  key: Vec<u8>,
  size: f32,
  cell: (u32, u32),
  pixels: Vec<u8>,
}

#[derive(Default)]
pub(super) struct TypedBox {
  base: Option<Base>,
  /// The cell the marks were last drawn into: its key and place, and them.
  shown: Option<(Vec<u8>, AtlasRect, TypingMarks)>,
}

impl TypedBox {
  /// Adds a draw to `draws` for a box being typed into whose marks are not
  /// the ones its cell shows, where the frame does not draw it already.
  /// `scale` is in atlas pixels per canvas pixel.
  pub(super) fn redraws(
    &self,
    types: &[PreparedType],
    keys: &[Vec<u8>],
    rects: &[AtlasRect],
    scale: f32,
    draws: &mut Vec<AtlasDraw>,
  ) {
    for (index, entry) in types.iter().enumerate() {
      let Some(marks) = entry.marks else {
        continue;
      };
      let rect = rects[index];
      if entry.style == 0 || rect.width == 0.0 || draws.iter().any(|draw| draw.index == index) {
        continue;
      }
      let shown = self
        .shown
        .as_ref()
        .is_some_and(|(key, at, shown)| *key == keys[index] && *at == rect && *shown == marks);
      if !shown {
        draws.push(AtlasDraw {
          index,
          // Quantised as the atlas keys a size, so it matches the first draw.
          radius: (entry.size * scale * 4.0).round() / 4.0,
        });
      }
    }
  }

  /// `entry`'s cell, `size` atlas pixels and drawn at `rect`, with `marks`.
  pub(super) fn draw(
    &mut self,
    entry: &PreparedType,
    key: &[u8],
    size: f32,
    drawn: f64,
    rect: AtlasRect,
    marks: TypingMarks,
  ) -> Result<Vec<u8>, String> {
    let cell = (rect.width as u32, rect.height as u32);
    let look = look(entry, size);
    let alignment = entry.style - 1;
    let base = match self.base.take() {
      Some(base) if base.key == key && base.size == size && base.cell == cell => base,
      _ => Base {
        key: key.to_vec(),
        size,
        cell,
        pixels: text_box::draw(look, &entry.text, size, drawn, alignment, cell)?,
      },
    };
    let mut pixels = base.pixels.clone();
    self.base = Some(base);
    text_box::mark(
      &mut pixels,
      look,
      &entry.text,
      size,
      drawn,
      alignment,
      marks,
      cell,
    )?;
    self.shown = Some((key.to_vec(), rect, marks));
    Ok(pixels)
  }
}
