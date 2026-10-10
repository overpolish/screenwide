// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! RVM's matte decided. RVM is right about where the person is far more
//! often than about how see-through each pixel is, so every part of the
//! frame is decided as the person or the room, and only the line between
//! them is soft, drawn again at one steady softness:
//!
//! - Anything RVM gives half or more is the person, apart from specks of it
//!   floating in the room.
//! - Every other area is the room if it reaches the top or either side of
//!   the frame, or if RVM is sure of most of it and it is bigger than a
//!   pinhole. Anything else is a pocket inside the person, and the person.
//! - A bite out of the person's outline that RVM was unsure of, coloured
//!   like the person beside it and not like the room around it, is the
//!   person: where RVM lost a patch against something like a sofa.
//! - The outline is evened out, and the edge drawn along it fading over a
//!   steady width, handed over smoothly to the decided value away from it.

mod fields;

use fields::{blur, distance, label, step};

/// Sizes in pixels of a 1080-line matte, scaled to the matte's own.
const BAND: f32 = 8.0;
const EDGE: f32 = 8.0;
const OUTLINE: f32 = 5.0;
const SOFTEN: f32 = 2.0;
const FARTHEST: f32 = 24.0;
const NOTCH: f32 = 10.0;
const ROOM_SPREAD: f32 = 20.0;
const PERSON_SPREAD: f32 = 8.0;
/// The largest share of the frame a pinhole or a speck takes.
const TINY: f64 = 0.0005;
/// Alpha RVM is sure of, either way.
const CLEAR: f32 = 0.1;
const SOLID: f32 = 0.9;
const UNSURE: (f32, f32) = (0.03, 0.97);
/// How far a bite's colour must be from the room's, and how much nearer the
/// person's, to be taken as the person, in RGB from 0 to 1.
const UNLIKE_ROOM: f32 = 0.12;
const LIKE_PERSON: f32 = 0.5;
/// How many pixels across a cell of the colour grid is.
const CELL: usize = 4;

/// Buffers kept from frame to frame, so a frame allocates nothing.
pub(super) struct Cleanup {
  size: (usize, usize),
  scale: f32,
  alpha: Vec<f32>,
  decided: Vec<bool>,
  marks: Vec<bool>,
  seen: Vec<bool>,
  order: Vec<usize>,
  inside: Vec<f32>,
  outside: Vec<f32>,
  field: Vec<f32>,
  scratch: Vec<f32>,
  flags: Vec<bool>,
  /// The room's red, green, blue and weight, then the person's, on a grid
  /// `cells` in size.
  colours: [Vec<f32>; 8],
  cells: (usize, usize),
}

impl Cleanup {
  pub(super) fn new((width, height): (usize, usize)) -> Self {
    let pixels = width * height;
    Self {
      size: (width, height),
      scale: height as f32 / 1080.0,
      alpha: vec![0.0; pixels],
      decided: vec![false; pixels],
      marks: vec![false; pixels],
      seen: vec![false; pixels],
      order: Vec::with_capacity(pixels),
      inside: vec![0.0; pixels],
      outside: vec![0.0; pixels],
      field: vec![0.0; pixels],
      scratch: Vec::new(),
      flags: Vec::new(),
      colours: std::array::from_fn(|_| vec![0.0; width.div_ceil(CELL) * height.div_ceil(CELL)]),
      cells: (width.div_ceil(CELL), height.div_ceil(CELL)),
    }
  }

  /// `alpha`, RVM's matte of the 8-bit RGB frame `rgb`, decided in place.
  pub(super) fn apply(&mut self, alpha: &mut [u8], rgb: &[u8]) {
    for (value, &byte) in self.alpha.iter_mut().zip(alpha.iter()) {
      *value = f32::from(byte) / 255.0;
    }
    self.decide();
    self.notches(rgb);
    self.draw_edge();
    for (byte, &value) in alpha.iter_mut().zip(&self.field) {
      *byte = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
  }

  /// Fills `decided`: the person, with every pocket inside them.
  fn decide(&mut self) {
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
  fn notches(&mut self, rgb: &[u8]) {
    let size = self.size;
    let reach = NOTCH * self.scale;
    // What closing the person over `reach` fills, and is not the person.
    distance(
      &self.decided,
      false,
      size,
      &mut self.outside,
      &mut self.scratch,
    );
    for ((grown, &decided), &away) in self.marks.iter_mut().zip(&self.decided).zip(&self.outside) {
      *grown = decided || away <= reach;
    }
    distance(&self.marks, true, size, &mut self.inside, &mut self.scratch);
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
    // on a grid a quarter the size: the means spread far, so the grid loses
    // nothing of them, and measuring them is sixteen times cheaper.
    let (small, cells) = (self.cells, self.cells.0 * self.cells.1);
    for field in &mut self.colours {
      field[..cells].fill(0.0);
    }
    for index in 0..self.alpha.len() {
      let alpha = self.alpha[index];
      let room = f32::from(alpha <= CLEAR);
      let person = f32::from(self.decided[index] && alpha >= SOLID);
      if room == 0.0 && person == 0.0 {
        continue;
      }
      let cell = cell_of(index, size.0, small.0);
      for channel in 0..3 {
        let value = f32::from(rgb[index * 3 + channel]) / 255.0;
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
      blur(&mut field[..cells], &mut self.scratch, small, sigma);
    }
    for index in 0..self.alpha.len() {
      if !self.marks[index] {
        continue;
      }
      let cell = cell_of(index, size.0, small.0);
      let colour = [0, 1, 2].map(|channel| f32::from(rgb[index * 3 + channel]) / 255.0);
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

  /// Fills `field` with the matte drawn around `decided`.
  fn draw_edge(&mut self) {
    let size = self.size;
    let scale = self.scale;
    distance(
      &self.decided,
      true,
      size,
      &mut self.inside,
      &mut self.scratch,
    );
    distance(
      &self.decided,
      false,
      size,
      &mut self.outside,
      &mut self.scratch,
    );
    let farthest = FARTHEST * scale;
    for ((field, &inside), &outside) in self.field.iter_mut().zip(&self.inside).zip(&self.outside) {
      *field = (inside - outside).clamp(-farthest, farthest);
    }
    // The outline evened out: ripples a few pixels across smoothed away.
    blur(&mut self.field, &mut self.scratch, size, OUTLINE * scale);
    let (edge, band) = ((EDGE * scale).max(1.0), (BAND * scale).max(1.0));
    for index in 0..self.field.len() {
      let drawn = (0.5 + self.field[index] / edge).clamp(0.0, 1.0);
      let t = ((self.inside[index] + self.outside[index]) / band).clamp(0.0, 1.0);
      let weight = t * t * (3.0 - 2.0 * t);
      let decided = f32::from(self.decided[index]);
      self.field[index] = drawn * (1.0 - weight) + decided * weight;
    }
    blur(&mut self.field, &mut self.scratch, size, SOFTEN * scale);
  }
}

/// The cell of the quarter-size grid, `cells_across` wide, that pixel `index`
/// of a frame `width` wide falls in.
fn cell_of(index: usize, width: usize, cells_across: usize) -> usize {
  (index / width / CELL) * cells_across + (index % width) / CELL
}

#[cfg(test)]
mod tests;
