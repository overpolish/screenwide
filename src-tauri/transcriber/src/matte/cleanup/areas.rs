// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which parts of the frame are the person and which the room, decided on a
//! copy of the matte and the frame half the size each way: the areas are
//! large and their rules measured in pixels many wide, so the copy loses
//! nothing they turn on, and deciding them costs a quarter as much.

use super::fields::{blur, distance, label, step, Lines};
use super::{
  CLEAR, LIKE_PERSON, NOTCH, PERSON_SPREAD, ROOM_SPREAD, SOLID, TINY, UNLIKE_ROOM, UNSURE,
};

/// How many pixels of the half-size copy a cell of the colour grid is
/// across: four of the frame's.
const CELL: usize = 2;

/// Buffers kept from frame to frame, so a frame allocates nothing.
pub(super) struct Areas {
  pub(super) size: (usize, usize),
  scale: f32,
  pub(super) alpha: Vec<f32>,
  rgb: Vec<u8>,
  /// The person, with every pocket inside them and every bite filled.
  pub(super) decided: Vec<bool>,
  marks: Vec<bool>,
  seen: Vec<bool>,
  order: Vec<usize>,
  inside: Vec<f32>,
  outside: Vec<f32>,
  scratch: Vec<f32>,
  flags: Vec<bool>,
  lines: Lines,
  /// The room's red, green, blue and weight, then the person's, on a grid
  /// `cells` in size.
  colours: [Vec<f32>; 8],
  cells: (usize, usize),
  grid: Lines,
}

impl Areas {
  /// Areas of a matte `(width, height)` in size.
  pub(super) fn new((width, height): (usize, usize)) -> Self {
    let size = (width.div_ceil(2), height.div_ceil(2));
    let pixels = size.0 * size.1;
    let cells = (size.0.div_ceil(CELL), size.1.div_ceil(CELL));
    Self {
      size,
      scale: size.1 as f32 / 1080.0,
      alpha: vec![0.0; pixels],
      rgb: vec![0; pixels * 3],
      decided: vec![false; pixels],
      marks: vec![false; pixels],
      seen: vec![false; pixels],
      order: Vec::with_capacity(pixels),
      inside: vec![0.0; pixels],
      outside: vec![0.0; pixels],
      scratch: Vec::new(),
      flags: Vec::new(),
      lines: Lines::whole(size),
      colours: std::array::from_fn(|_| vec![0.0; cells.0 * cells.1]),
      cells,
      grid: Lines::whole(cells),
    }
  }

  /// Decides the matte `alpha` of the 8-bit RGB frame `rgb`, both
  /// `(width, height)` in size, into `decided`.
  pub(super) fn decide(&mut self, alpha: &[u8], rgb: &[u8], (width, height): (usize, usize)) {
    let small = self.size;
    for y in 0..small.1 {
      let rows = [2 * y, (2 * y + 1).min(height - 1)];
      for x in 0..small.0 {
        let columns = [2 * x, (2 * x + 1).min(width - 1)];
        let at = |row: usize, column: usize| row * width + column;
        let corners = rows.map(|row| columns.map(|column| at(row, column)));
        let corners = corners.as_flattened();
        let index = y * small.0 + x;
        let sum: u32 = corners.iter().map(|&pixel| u32::from(alpha[pixel])).sum();
        self.alpha[index] = sum as f32 / (4.0 * 255.0);
        for channel in 0..3 {
          let sum: u32 = corners
            .iter()
            .map(|&pixel| u32::from(rgb[pixel * 3 + channel]))
            .sum();
          self.rgb[index * 3 + channel] = ((sum + 2) / 4) as u8;
        }
      }
    }
    self.areas();
    self.notches();
  }

  /// Fills `decided`: the person, with every pocket inside them.
  fn areas(&mut self) {
    let size = self.size;
    let tiny = (TINY * self.alpha.len() as f64) as usize;
    for (person, &alpha) in self.marks.iter_mut().zip(&self.alpha) {
      *person = alpha >= 0.5;
    }
    // Specks of person floating in the room are dropped.
    for area in label(&self.marks, true, size, &mut self.seen, &mut self.order) {
      if area.size <= tiny {
        for &index in &self.order[area.pixels] {
          self.marks[index] = false;
        }
      }
    }
    self.decided.fill(true);
    let width = size.0;
    for area in label(&self.marks, false, size, &mut self.seen, &mut self.order) {
      let pixels = &self.order[area.pixels];
      let clear = pixels
        .iter()
        .filter(|&&index| self.alpha[index] <= CLEAR)
        .count();
      // The camera cuts the person off at the bottom, so only the top and
      // the sides open onto the room.
      let opens = pixels.iter().any(|&index| {
        let x = index % width;
        index < width || x == 0 || x + 1 == width
      });
      if opens || (clear * 2 > area.size && area.size > tiny) {
        for &index in pixels {
          self.decided[index] = false;
        }
      }
    }
  }

  /// Adds to `decided` the bites out of its outline RVM was unsure of that
  /// are coloured like the person rather than the room.
  fn notches(&mut self) {
    let size = self.size;
    let reach = NOTCH * self.scale;
    // What closing the person over `reach` fills, and is not the person.
    let lines = &self.lines;
    distance(
      &self.decided,
      false,
      size,
      lines,
      &mut self.outside,
      &mut self.scratch,
    );
    for ((grown, &decided), &away) in self.marks.iter_mut().zip(&self.decided).zip(&self.outside) {
      *grown = decided || away <= reach;
    }
    distance(
      &self.marks,
      true,
      size,
      lines,
      &mut self.inside,
      &mut self.scratch,
    );
    let mut any = false;
    for index in 0..self.alpha.len() {
      let alpha = self.alpha[index];
      let bite = !self.decided[index] && self.inside[index] > reach;
      self.marks[index] = bite && alpha > UNSURE.0 && alpha < UNSURE.1;
      any |= self.marks[index];
    }
    if !any {
      return;
    }
    // The colour of the room and of the person around each pixel, measured
    // on a coarser grid: the means spread far, so the grid loses nothing of
    // them, and measuring them costs a fraction as much.
    let cells = self.cells;
    for field in &mut self.colours {
      field.fill(0.0);
    }
    for index in 0..self.alpha.len() {
      let alpha = self.alpha[index];
      let room = f32::from(alpha <= CLEAR);
      let person = f32::from(self.decided[index] && alpha >= SOLID);
      if room == 0.0 && person == 0.0 {
        continue;
      }
      let cell = cell_of(index, size.0, cells.0);
      for channel in 0..3 {
        let value = f32::from(self.rgb[index * 3 + channel]) / 255.0;
        self.colours[channel][cell] += value * room;
        self.colours[4 + channel][cell] += value * person;
      }
      self.colours[3][cell] += room;
      self.colours[7][cell] += person;
    }
    let spread = self.scale / CELL as f32;
    for (index, field) in self.colours.iter_mut().enumerate() {
      let sigma = if index < 4 {
        ROOM_SPREAD
      } else {
        PERSON_SPREAD
      } * spread;
      blur(field, &mut self.scratch, cells, sigma, &self.grid);
    }
    for index in 0..self.alpha.len() {
      if !self.marks[index] {
        continue;
      }
      let cell = cell_of(index, size.0, cells.0);
      let colour = [0, 1, 2].map(|channel| f32::from(self.rgb[index * 3 + channel]) / 255.0);
      let apart = |first: usize| {
        let weight = self.colours[first + 3][cell].max(1e-4);
        (0..3)
          .map(|channel| colour[channel] - self.colours[first + channel][cell] / weight)
          .map(|difference| difference * difference)
          .sum::<f32>()
          .sqrt()
      };
      let (to_room, to_person) = (apart(0), apart(4));
      self.marks[index] = to_room > UNLIKE_ROOM && to_person < to_room * LIKE_PERSON;
    }
    // A lone pixel or a hair-thin line is not a bite.
    step(&mut self.marks, &mut self.flags, size, 1, false);
    step(&mut self.marks, &mut self.flags, size, 1, true);
    for (decided, &bite) in self.decided.iter_mut().zip(&self.marks) {
      *decided |= bite;
    }
  }
}

/// The cell of the colour grid, `cells_across` wide, that pixel `index` of
/// a frame `width` wide falls in.
fn cell_of(index: usize, width: usize, cells_across: usize) -> usize {
  (index / width / CELL) * cells_across + (index % width) / CELL
}
