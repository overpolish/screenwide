// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The edge drawn at the matte's full size, only near the outline. The
//! areas are decided on the half-size copy; the outline is placed again at
//! full size from RVM's own matte wherever the half-size decision follows
//! it, and the edge drawn along it. Away from the outline every pixel is
//! simply the person or the room, so the measuring and blurring run only on
//! the tiles of the frame near it, which give the same matte as running on
//! the whole frame.

use super::areas::Areas;
use super::fields::{blur, distance, reach, Lines};
use super::{BAND, EDGE, FARTHEST, OUTLINE, SOFTEN};

/// How many pixels across a tile of the frame is: the outline is worked on
/// in whole tiles.
const TILE: usize = 8;

/// Buffers kept from frame to frame, so a frame allocates nothing.
pub(super) struct Edge {
  size: (usize, usize),
  scale: f32,
  decided: Vec<bool>,
  /// The tiles the outline runs through, then those near enough to it to
  /// be worked on, `tiles` in size.
  outline: Vec<bool>,
  near: Vec<bool>,
  tiles: (usize, usize),
  lines: Lines,
  inside: Vec<f32>,
  outside: Vec<f32>,
  field: Vec<f32>,
  scratch: Vec<f32>,
}

impl Edge {
  pub(super) fn new((width, height): (usize, usize)) -> Self {
    let pixels = width * height;
    let tiles = (width.div_ceil(TILE), height.div_ceil(TILE));
    Self {
      size: (width, height),
      scale: height as f32 / 1080.0,
      decided: vec![false; pixels],
      outline: vec![false; tiles.0 * tiles.1],
      near: vec![false; tiles.0 * tiles.1],
      tiles,
      lines: Lines::default(),
      inside: vec![0.0; pixels],
      outside: vec![0.0; pixels],
      field: vec![0.0; pixels],
      scratch: Vec::new(),
    }
  }

  /// Draws into `alpha`, RVM's matte at full size, the edge around what
  /// `areas` decided of it.
  pub(super) fn draw(&mut self, areas: &Areas, alpha: &mut [u8]) {
    self.place(areas, alpha);
    self.find_near();
    let (size, scale) = (self.size, self.scale);
    let lines = &self.lines;
    distance(
      &self.decided,
      true,
      size,
      lines,
      &mut self.inside,
      &mut self.scratch,
    );
    distance(
      &self.decided,
      false,
      size,
      lines,
      &mut self.outside,
      &mut self.scratch,
    );
    let farthest = FARTHEST * scale;
    for index in lines.pixels() {
      self.field[index] = (self.inside[index] - self.outside[index]).clamp(-farthest, farthest);
    }
    // The outline evened out: ripples a few pixels across smoothed away.
    blur(
      &mut self.field,
      &mut self.scratch,
      size,
      OUTLINE * scale,
      lines,
    );
    let (edge, band) = ((EDGE * scale).max(1.0), (BAND * scale).max(1.0));
    for index in lines.pixels() {
      let drawn = (0.5 + self.field[index] / edge).clamp(0.0, 1.0);
      let t = ((self.inside[index] + self.outside[index]) / band).clamp(0.0, 1.0);
      let weight = t * t * (3.0 - 2.0 * t);
      let decided = f32::from(self.decided[index]);
      self.field[index] = drawn * (1.0 - weight) + decided * weight;
    }
    blur(
      &mut self.field,
      &mut self.scratch,
      size,
      SOFTEN * scale,
      lines,
    );
    for (byte, &decided) in alpha.iter_mut().zip(&self.decided) {
      *byte = if decided { 255 } else { 0 };
    }
    for index in lines.pixels() {
      alpha[index] = (self.field[index].clamp(0.0, 1.0) * 255.0).round() as u8;
    }
  }

  /// Fills `decided` at full size from the half-size decision, the outline
  /// placed again from RVM's matte `alpha` where the decision follows it,
  /// and notes the tiles the outline runs through.
  fn place(&mut self, areas: &Areas, alpha: &[u8]) {
    let (width, height) = self.size;
    let (small_width, small_height) = areas.size;
    let small_of = |x: usize, y: usize| (y / 2) * small_width + x / 2;
    for y in 0..height {
      for x in 0..width {
        self.decided[y * width + x] = areas.decided[small_of(x, y)];
      }
    }
    self.outline.fill(false);
    for y in 0..small_height {
      for x in 0..small_width {
        let index = y * small_width + x;
        let here = areas.decided[index];
        let differs = (x + 1 < small_width && areas.decided[index + 1] != here)
          || (y + 1 < small_height && areas.decided[index + small_width] != here);
        if !differs {
          continue;
        }
        // The half-size pixels either side of the outline, at full size.
        for full_y in (2 * y).saturating_sub(2)..(2 * y + 4).min(height) {
          for full_x in (2 * x).saturating_sub(2)..(2 * x + 4).min(width) {
            let small = small_of(full_x, full_y);
            // Where the decision differs from RVM's (a pocket filled, a bite
            // taken in, a speck dropped), it stands.
            if areas.decided[small] == (areas.alpha[small] >= 0.5) {
              self.decided[full_y * width + full_x] = alpha[full_y * width + full_x] >= 128;
            }
          }
        }
        self.outline[(2 * y / TILE) * self.tiles.0 + 2 * x / TILE] = true;
      }
    }
  }

  /// Fills `lines` with the tiles near enough to the outline that the edge
  /// drawn there can differ from the decided value, or a measure of it read
  /// there can differ from what it would be on the whole frame: out to the
  /// farthest distance measured, past the reach of both blurs and the band
  /// the edge is handed over in.
  fn find_near(&mut self) {
    let scale = self.scale;
    let margin = (FARTHEST * scale).ceil() as usize
      + (BAND * scale).ceil() as usize
      + reach(OUTLINE * scale)
      + reach(SOFTEN * scale)
      // The outline placed again lies a few pixels from the half-size one.
      + 4;
    let around = margin.div_ceil(TILE);
    let (across, down) = self.tiles;
    self.near.fill(false);
    for tile_y in 0..down {
      for tile_x in 0..across {
        if !self.outline[tile_y * across + tile_x] {
          continue;
        }
        for y in tile_y.saturating_sub(around)..(tile_y + around + 1).min(down) {
          let row = y * across;
          self.near[row + tile_x.saturating_sub(around)..row + (tile_x + around + 1).min(across)]
            .fill(true);
        }
      }
    }
    let (width, height) = self.size;
    let (near, lines) = (&self.near, &mut self.lines);
    lines.across.clear();
    lines.down.clear();
    let tile_span = |tile: usize, length: usize| tile * TILE..((tile + 1) * TILE).min(length);
    for tile_y in 0..down {
      let flags = (0..across).map(|tile_x| near[tile_y * across + tile_x]);
      each_run(flags, |from, to| {
        let (x, end) = (from * TILE, (to * TILE).min(width));
        for y in tile_span(tile_y, height) {
          lines.across.push((y * width + x, end - x));
        }
      });
    }
    for tile_x in 0..across {
      let flags = (0..down).map(|tile_y| near[tile_y * across + tile_x]);
      each_run(flags, |from, to| {
        let (y, end) = (from * TILE, (to * TILE).min(height));
        for x in tile_span(tile_x, width) {
          lines.down.push((y * width + x, end - y));
        }
      });
    }
  }
}

/// Calls `run` with where each run of `true` in `flags` starts and ends.
fn each_run(flags: impl Iterator<Item = bool>, mut run: impl FnMut(usize, usize)) {
  let mut start = None;
  let mut length = 0;
  for (index, flag) in flags.enumerate() {
    match (flag, start) {
      (true, None) => start = Some(index),
      (false, Some(from)) => {
        run(from, index);
        start = None;
      }
      _ => {}
    }
    length = index + 1;
  }
  if let Some(from) = start {
    run(from, length);
  }
}
