// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the annotations' type - counters' numbers and text boxes' text - sits
//! in the atlas both backends rasterise it into. Type is drawn by each
//! platform's own text engine; where it goes is decided here, once. The atlas
//! outlives a composition and is keyed by the text, its size and how it is
//! set, so a frame rasterises only what the atlas does not hold yet. It holds
//! what recent frames drew, not what the document holds: when a frame's new
//! type no longer fits, every cell that frame does not use is dropped and the
//! atlas is laid out again at a size for that frame's own type.

use std::collections::HashMap;

/// Where one number sits, in atlas pixels. A number that is not drawn gets a
/// zero rectangle, which the kernels skip. The twin of
/// `ScreenwideAnnotationTextRect`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct AtlasRect {
  pub(crate) x: f32,
  pub(crate) y: f32,
  pub(crate) width: f32,
  pub(crate) height: f32,
}

/// The largest side the atlas takes: the biggest texture every D3D11 feature
/// level 11 device accepts. Metal holds the same ceiling so both backends
/// agree on how many numbers one frame can show.
const MAX_SIDE: u32 = 16_384;
const BASE_WIDTH: u32 = 1_024;
const BASE_HEIGHT: u32 = 256;

/// One piece of type a composition draws: its text, its size in drawn pixels
/// (a counter's disc radius or a text box's type size), and how it is set:
/// zero for a counter's number, one more than its alignment for a text box.
pub(crate) struct CounterNumber<'a> {
  pub(crate) text: &'a [u8],
  pub(crate) radius: f32,
  pub(crate) style: u32,
}

/// One number to rasterise into its rectangle, at the radius it is keyed by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AtlasDraw {
  pub(crate) index: usize,
  pub(crate) radius: f32,
}

/// The storage a frame reads from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AtlasLayout {
  pub(crate) size: (u32, u32),
  /// The atlas was laid out again: the platform starts from new storage of
  /// `size`, and the frame's draws cover every number it shows. Without it,
  /// cells already drawn stay where they are and the draws land in space no
  /// cell has used.
  pub(crate) fresh: bool,
}

/// A piece of type, its size and how it is set, compared without allocating
/// where it can be: a counter's number always fits the short form.
#[derive(Clone, PartialEq, Eq, Hash)]
enum CellKey {
  Short {
    len: u8,
    bytes: [u8; 11],
    quarters: u32,
    style: u32,
  },
  Long {
    text: Box<[u8]>,
    quarters: u32,
    style: u32,
  },
}

impl CellKey {
  fn new(text: &[u8], quarters: u32, style: u32) -> Self {
    if text.len() <= 11 {
      let mut bytes = [0; 11];
      bytes[..text.len()].copy_from_slice(text);
      Self::Short {
        len: text.len() as u8,
        bytes,
        quarters,
        style,
      }
    } else {
      Self::Long {
        text: text.into(),
        quarters,
        style,
      }
    }
  }
}

#[derive(Clone, Copy)]
struct Cell {
  x: u32,
  y: u32,
  width: u32,
  height: u32,
  frame: u64,
}

impl Cell {
  fn rect(&self) -> AtlasRect {
    AtlasRect {
      x: self.x as f32,
      y: self.y as f32,
      width: self.width as f32,
      height: self.height as f32,
    }
  }
}

#[path = "atlas_shelves.rs"]
mod shelves;
use shelves::Shelves;

/// One distinct number in a frame, and every place in the frame's list that
/// shows it.
struct Unique {
  key: CellKey,
  quarters: u32,
  size: Option<(u32, u32)>,
  cell: Option<Cell>,
  indices: Vec<usize>,
}

#[derive(Default)]
pub(crate) struct CounterAtlas {
  space: Shelves,
  cells: HashMap<CellKey, Cell>,
  frame: u64,
}

impl CounterAtlas {
  /// Lays out one composition's numbers. `measure` gives the cell a number
  /// needs at a radius, and is asked only for numbers the atlas does not hold;
  /// `None` leaves the number undrawn. `rects` receives one rectangle per
  /// number and `draws` the numbers the platform rasterises now.
  pub(crate) fn frame(
    &mut self,
    numbers: &[CounterNumber<'_>],
    mut measure: impl FnMut(usize, f32) -> Option<(u32, u32)>,
    rects: &mut [AtlasRect],
    draws: &mut Vec<AtlasDraw>,
  ) -> AtlasLayout {
    self.frame += 1;
    draws.clear();
    rects.fill(AtlasRect::default());
    let mut uniques: Vec<Unique> = Vec::new();
    let mut seen: HashMap<CellKey, usize> = HashMap::new();
    for (index, number) in numbers.iter().enumerate() {
      // A disc under a pixel across is mid-arrival and has nothing to show.
      if number.text.is_empty() || !number.radius.is_finite() || number.radius <= 1.0 {
        continue;
      }
      let quarters = (number.radius * 4.0).round() as u32;
      let key = CellKey::new(number.text, quarters, number.style);
      if let Some(&unique) = seen.get(&key) {
        uniques[unique].indices.push(index);
        continue;
      }
      let cell = self.cells.get_mut(&key).map(|cell| {
        cell.frame = self.frame;
        *cell
      });
      seen.insert(key.clone(), uniques.len());
      uniques.push(Unique {
        key,
        quarters,
        size: cell.map(|cell| (cell.width, cell.height)),
        cell,
        indices: vec![index],
      });
    }
    for unique in uniques.iter_mut().filter(|unique| unique.cell.is_none()) {
      unique.size = measure(unique.indices[0], unique.quarters as f32 / 4.0)
        .filter(|&(width, height)| width > 0 && height > 0);
    }

    let fresh = !self.place_new(&mut uniques);
    if fresh {
      self.lay_out(&mut uniques);
    }
    for unique in &uniques {
      let Some(cell) = unique.cell else {
        continue;
      };
      for &index in &unique.indices {
        rects[index] = cell.rect();
      }
      if fresh || cell.frame == self.frame && !self.cells.contains_key(&unique.key) {
        draws.push(AtlasDraw {
          index: unique.indices[0],
          radius: unique.quarters as f32 / 4.0,
        });
      }
      self.cells.insert(unique.key.clone(), cell);
    }
    AtlasLayout {
      size: self.space.size(),
      fresh,
    }
  }

  /// Fits the numbers the atlas does not hold into the space it has left,
  /// reporting whether every one found room. On `false` the layout is laid
  /// out again, so what was placed before the miss does not matter.
  fn place_new(&mut self, uniques: &mut [Unique]) -> bool {
    if self.space.size() == (0, 0) {
      return uniques.iter().all(|unique| unique.size.is_none());
    }
    for unique in uniques.iter_mut().filter(|unique| unique.cell.is_none()) {
      let Some((width, height)) = unique.size else {
        continue;
      };
      match self.space.allocate(width, height) {
        Some((x, y)) => {
          unique.cell = Some(Cell {
            x,
            y,
            width,
            height,
            frame: self.frame,
          });
        }
        None => return false,
      }
    }
    true
  }

  /// Drops every cell and lays the frame's own numbers out again, at the
  /// smallest size that holds them with room to spare. At the ceiling, what
  /// does not fit is left undrawn.
  fn lay_out(&mut self, uniques: &mut [Unique]) {
    self.cells.clear();
    let widest = uniques
      .iter()
      .filter_map(|unique| unique.size.map(|(width, _)| width))
      .max()
      .unwrap_or(1);
    let mut order: Vec<usize> = (0..uniques.len())
      .filter(|&index| uniques[index].size.is_some())
      .collect();
    // Tallest first, so a shelf is opened by the cell that sets its height.
    order.sort_by_key(|&index| std::cmp::Reverse(uniques[index].size.map_or(0, |size| size.1)));
    let mut size = (
      BASE_WIDTH.max(widest.next_power_of_two()).min(MAX_SIDE),
      BASE_HEIGHT,
    );
    loop {
      self.space.reset(size);
      let mut placed = Vec::with_capacity(order.len());
      let mut fits = true;
      for &index in &order {
        let (width, height) = uniques[index].size.unwrap_or_default();
        match self.space.allocate(width, height) {
          Some(at) => placed.push((index, at)),
          None => fits = false,
        }
      }
      let at_ceiling = size == (MAX_SIDE, MAX_SIDE);
      // Half the height spare, so the frames after this one add their new
      // sizes without laying everything out again straight away.
      if fits && self.space.used_height() <= size.1 / 2 || at_ceiling {
        for unique in uniques.iter_mut() {
          unique.cell = None;
        }
        for (index, (x, y)) in placed {
          let (width, height) = uniques[index].size.unwrap_or_default();
          uniques[index].cell = Some(Cell {
            x,
            y,
            width,
            height,
            frame: self.frame,
          });
        }
        return;
      }
      size = if size.1 < size.0 {
        (size.0, size.1 * 2)
      } else {
        ((size.0 * 2).min(MAX_SIDE), size.1)
      };
      size.1 = size.1.min(MAX_SIDE);
    }
  }
}

#[cfg(test)]
#[path = "atlas_tests.rs"]
mod tests;
