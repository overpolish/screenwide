// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The atlas's free space, handed out a row at a time. A number's cell goes on
//! the first row of about its height with the width left, else on a new row
//! under the last; nothing is ever given back short of starting over.

/// A row of cells: every cell on it starts at its top.
struct Shelf {
  y: u32,
  height: u32,
  used: u32,
}

#[derive(Default)]
pub(super) struct Shelves {
  size: (u32, u32),
  shelves: Vec<Shelf>,
  bottom: u32,
}

impl Shelves {
  pub(super) fn size(&self) -> (u32, u32) {
    self.size
  }

  /// How far down the rows reach.
  pub(super) fn used_height(&self) -> u32 {
    self.bottom
  }

  /// Empties the space and makes it `size`.
  pub(super) fn reset(&mut self, size: (u32, u32)) {
    self.size = size;
    self.shelves.clear();
    self.bottom = 0;
  }

  /// Where a `width` by `height` cell goes, if it fits anywhere.
  pub(super) fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
    if width > self.size.0 || height > self.size.1 {
      return None;
    }
    // A shelf much taller than the cell would waste the gap above it.
    let suits = |shelf: &Shelf| shelf.height >= height && shelf.height <= height + height / 4 + 1;
    let row_width = self.size.0;
    if let Some(shelf) = self
      .shelves
      .iter_mut()
      .find(|shelf| suits(shelf) && row_width - shelf.used >= width)
    {
      let at = (shelf.used, shelf.y);
      shelf.used += width;
      return Some(at);
    }
    if self.size.1 - self.bottom < height {
      return None;
    }
    let y = self.bottom;
    self.shelves.push(Shelf {
      y,
      height,
      used: width,
    });
    self.bottom += height;
    Some((0, y))
  }
}
