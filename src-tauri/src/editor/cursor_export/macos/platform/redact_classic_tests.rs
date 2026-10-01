// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Classic pixelation through the real compositor: the ordinary kind, each
//! block the average of the pixels under it, and so open to the same
//! depixelation attack the secure pixelation withstands.

use super::redact_attack_tests::{composed, Text, INK, LEFT, SECRETS, SURFACE, TEXTS, TOP};
use crate::editor::annotations::AnnotationRedaction;

/// The compositor's classic blocks over a text's box: `block` pixels square
/// and centred on the box, measured in floats as the GPU measures them.
struct Blocks {
  size: f32,
  across: u32,
  down: u32,
  origin: [f32; 2],
  box_size: [u32; 2],
}

impl Blocks {
  fn new(text: Text, block: u32) -> Self {
    let size = block as f32;
    let box_size = [text.box_width(), text.box_height()];
    let count = |extent: u32| (extent as f32 / size).ceil().max(1.0) as u32;
    let (across, down) = (count(box_size[0]), count(box_size[1]));
    let origin = [
      (box_size[0] as f32 - across as f32 * size) * 0.5,
      (box_size[1] as f32 - down as f32 * size) * 0.5,
    ];
    Self {
      size,
      across,
      down,
      origin,
      box_size,
    }
  }

  fn axis(&self, local: u32, axis: usize, last: u32) -> u32 {
    (((local as f32 - self.origin[axis]) / self.size)
      .floor()
      .max(0.0) as u32)
      .min(last)
  }

  /// The block a pixel `(x, y)` into the box falls in.
  fn of(&self, x: u32, y: u32) -> usize {
    (self.axis(y, 1, self.down - 1) * self.across + self.axis(x, 0, self.across - 1)) as usize
  }

  /// How many pixels each block holds, and how many of those `mask` inks.
  fn counts(&self, mask: &[bool]) -> (Vec<u32>, Vec<u32>) {
    let blocks = (self.across * self.down) as usize;
    let (mut pixels, mut inked) = (vec![0; blocks], vec![0; blocks]);
    for (index, ink) in mask.iter().enumerate() {
      let (x, y) = (
        index as u32 % self.box_size[0],
        index as u32 / self.box_size[0],
      );
      let block = self.of(x, y);
      pixels[block] += 1;
      inked[block] += u32::from(*ink);
    }
    (pixels, inked)
  }

  /// A pixel of the box inside block `index`: its middle, held to the box.
  fn inside(&self, index: usize) -> (u32, u32) {
    let (column, row) = (index as u32 % self.across, index as u32 / self.across);
    let middle = |cell: u32, axis: usize| {
      let at = self.origin[axis] + (cell as f32 + 0.5) * self.size;
      (at.max(0.0) as u32).min(self.box_size[axis] - 1)
    };
    (middle(column, 0), middle(row, 1))
  }
}

/// The green each block shows: the channel ink and surface differ most in.
fn shown(text: Text, rgba: &[u8], blocks: &Blocks) -> Vec<f64> {
  let width = text.width();
  (0..(blocks.across * blocks.down) as usize)
    .map(|index| {
      let (x, y) = blocks.inside(index);
      assert_eq!(blocks.of(x, y), index);
      f64::from(rgba[(((TOP + y) * width + LEFT + x) * 4 + 1) as usize])
    })
    .collect()
}

/// The green a block of `pixels` pixels shows with `inked` of them inked.
fn expected(inked: u32, pixels: u32) -> f64 {
  let [surface, ink] = [SURFACE[1], INK[1]].map(f64::from);
  surface + (ink - surface) * f64::from(inked) / f64::from(pixels.max(1))
}

#[test]
fn classic_pixelation_paints_each_block_its_average() {
  let text = TEXTS[0];
  let blocks = Blocks::new(text, 16);
  let rgba = composed(text, SECRETS[0], AnnotationRedaction::PixelateClassic, 16);
  let (pixels, inked) = blocks.counts(&text.ink(SECRETS[0]));
  for (index, green) in shown(text, &rgba, &blocks).into_iter().enumerate() {
    let want = expected(inked[index], pixels[index]);
    assert!(
      (green - want).abs() <= 1.5,
      "block {index}: {green} for {want}"
    );
  }
}

/// The attack the secure pixelation leaves nothing to narrow, run on the
/// classic one's real output: each block's shade gives away how many of its
/// pixels are inked, and matching those against every candidate narrows ten
/// thousand PINs to the PIN itself at the smallest block, and to a sliver of
/// them at the next. This is why the editor calls it a look.
#[test]
fn classic_pixelation_gives_a_pin_away() {
  for text in TEXTS {
    for block in [16, 24] {
      let blocks = Blocks::new(text, block);
      for secret in SECRETS {
        let observed = shown(
          text,
          &composed(text, secret, AnnotationRedaction::PixelateClassic, block),
          &blocks,
        );
        let fits = |pin: u32| {
          let (pixels, inked) = blocks.counts(&text.ink(pin));
          observed
            .iter()
            .enumerate()
            .all(|(index, green)| (expected(inked[index], pixels[index]) - green).abs() <= 1.5)
        };
        assert!(
          fits(secret),
          "{text:?} block {block}: {secret:04} does not fit itself"
        );
        let left = (0..10_000).filter(|pin| fits(*pin)).count();
        // Blocks of 16 give the PIN itself; blocks of 24 over small thin type
        // leave a couple of hundred at worst, from ten thousand.
        assert!(
          left <= 250,
          "{text:?} block {block}: {left} candidates left for {secret:04}"
        );
      }
    }
  }
}
