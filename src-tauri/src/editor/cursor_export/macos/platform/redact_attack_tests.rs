// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Trying to recover a redacted secret the way a depixelation attack does.
//!
//! The secret is a four-digit PIN drawn in a known font at a known place, the
//! attacker's best case. The attacker renders all ten thousand candidates
//! the same way, works out what each would show once redacted, and counts
//! how many candidates are at least as likely as the secret: how many
//! guesses the attack needs. Ordinary pixelation, the control, narrows the
//! PIN to a handful. The redaction's pixelation shows which zones held ink,
//! and the attacker is given every zone that shows colour as certain.

use crate::editor::annotations::redact::new_redact;
use crate::editor::annotations::{AnnotationPoint, AnnotationRedaction, AnnotationShape};
use crate::screenshots::CapturedImage;

/// A 5 by 7 digit font, one string a row.
const FONT: [[&str; 7]; 10] = [
  [
    "01110", "10001", "10011", "10101", "11001", "10001", "01110",
  ],
  [
    "00100", "01100", "00100", "00100", "00100", "00100", "01110",
  ],
  [
    "01110", "10001", "00001", "00010", "00100", "01000", "11111",
  ],
  [
    "11111", "00010", "00100", "00010", "00001", "10001", "01110",
  ],
  [
    "00010", "00110", "01010", "10010", "11111", "00010", "00010",
  ],
  [
    "11111", "10000", "11110", "00001", "00001", "10001", "01110",
  ],
  [
    "00110", "01000", "10000", "11110", "10001", "10001", "01110",
  ],
  [
    "11111", "00001", "00010", "00100", "01000", "01000", "01000",
  ],
  [
    "01110", "10001", "10001", "01110", "10001", "10001", "01110",
  ],
  [
    "01110", "10001", "10001", "01111", "00001", "00010", "01100",
  ],
];
const DIGITS: u32 = 4;
pub(super) const LEFT: u32 = 32;
pub(super) const TOP: u32 = 20;
pub(super) const SURFACE: [u8; 3] = [30, 30, 30];
pub(super) const INK: [u8; 3] = [0, 230, 120];
pub(super) const SECRETS: [u32; 4] = [1_984, 2_604, 5_517, 9_031];

/// How the PIN is drawn: each font pixel `scale` source pixels square, and
/// either filled or `thin`, a one-pixel cross through its middle that joins
/// its neighbours' into strokes one pixel wide.
#[derive(Clone, Copy, Debug)]
pub(super) struct Text {
  scale: u32,
  thin: bool,
}

/// Bold type about 13pt on a Retina screen, the same thin, and thin type
/// twice the size: the finer the strokes and the larger the type against the
/// blocks, the more a block pattern can give away.
pub(super) const TEXTS: [Text; 3] = [
  Text {
    scale: 3,
    thin: false,
  },
  Text {
    scale: 3,
    thin: true,
  },
  Text {
    scale: 6,
    thin: true,
  },
];

impl Text {
  pub(super) fn box_width(self) -> u32 {
    DIGITS * 6 * self.scale
  }

  pub(super) fn box_height(self) -> u32 {
    7 * self.scale
  }

  /// The canvas round the box, even and at least 64 pixels each way, which
  /// the compositor asks of a canvas.
  pub(super) fn width(self) -> u32 {
    (LEFT * 2 + self.box_width()).max(64).next_multiple_of(2)
  }

  fn height(self) -> u32 {
    (TOP * 2 + self.box_height()).max(64).next_multiple_of(2)
  }

  /// Which pixels of the box a PIN inks, row by row.
  pub(super) fn ink(self, pin: u32) -> Vec<bool> {
    let digits = [pin / 1_000, pin / 100 % 10, pin / 10 % 10, pin % 10];
    let width = self.box_width();
    let middle = self.scale / 2;
    let mut mask = vec![false; (width * self.box_height()) as usize];
    for (place, digit) in digits.into_iter().enumerate() {
      for (row, bits) in FONT[digit as usize].iter().enumerate() {
        for (column, bit) in bits.bytes().enumerate() {
          if bit != b'1' {
            continue;
          }
          for y in 0..self.scale {
            for x in 0..self.scale {
              if self.thin && x != middle && y != middle {
                continue;
              }
              let px = place as u32 * 6 * self.scale + column as u32 * self.scale + x;
              let py = row as u32 * self.scale + y;
              mask[(py * width + px) as usize] = true;
            }
          }
        }
      }
    }
    mask
  }

  fn picture(self, pin: u32) -> CapturedImage {
    let (width, box_width) = (self.width(), self.box_width());
    let mut rgba =
      [SURFACE[0], SURFACE[1], SURFACE[2], 255].repeat((width * self.height()) as usize);
    for (index, inked) in self.ink(pin).into_iter().enumerate() {
      if inked {
        let (x, y) = (
          LEFT + index as u32 % box_width,
          TOP + index as u32 / box_width,
        );
        rgba[((y * width + x) * 4) as usize..][..3].copy_from_slice(&INK);
      }
    }
    CapturedImage {
      rgba,
      width,
      height: self.height(),
    }
  }

  /// How many inked pixels fall in each `size`-pixel cell of the box, counted
  /// from its top-left corner, as the redaction lays out its blocks and zones.
  pub(super) fn cells(self, mask: &[bool], size: u32) -> Vec<u32> {
    let width = self.box_width();
    let (across, down) = (width.div_ceil(size), self.box_height().div_ceil(size));
    let mut counts = vec![0; (across * down) as usize];
    for (index, inked) in mask.iter().enumerate() {
      if *inked {
        let (x, y) = (index as u32 % width, index as u32 / width);
        counts[((y / size) * across + x / size) as usize] += 1;
      }
    }
    counts
  }

  /// A zone's side in source pixels for blocks `block` pixels square: at
  /// least the box's height and two blocks, the palette's own rule.
  fn zone(self, block: u32) -> u32 {
    block * self.box_height().div_ceil(block).max(2)
  }

  /// Whether a zone `zone` pixels square holding `count` inked pixels takes
  /// the ink as a colour of its own: the palette's rule, from 2% of it.
  fn inked(self, zone: u32, cell: usize, count: u32) -> bool {
    let across = self.box_width().div_ceil(zone);
    let (column, row) = (cell as u32 % across, cell as u32 / across);
    let pixels =
      zone.min(self.box_width() - column * zone) * zone.min(self.box_height() - row * zone);
    f64::from(count) >= (f64::from(pixels) * 0.02).max(1.0)
  }
}

/// The control: ordinary pixelation shows each block's average, which with
/// two colours is how many of its pixels are inked. Matching those counts
/// narrows ten thousand PINs to a handful, or to the PIN itself.
#[test]
fn ordinary_pixelation_gives_a_pin_away() {
  for text in TEXTS {
    for block in [8, 12, 16] {
      for secret in SECRETS {
        let shown = text.cells(&text.ink(secret), block);
        let left = (0..10_000)
          .filter(|pin| text.cells(&text.ink(*pin), block) == shown)
          .count();
        assert!(
          left <= 10,
          "{text:?} block {block}: {left} candidates left for {secret:04}"
        );
      }
    }
  }
}

/// The secret's picture through the real compositor, one pixel to one, under
/// a box over the PIN redacted in `mode` with blocks `block` pixels square.
pub(super) fn composed(text: Text, secret: u32, mode: AnnotationRedaction, block: u32) -> Vec<u8> {
  let point = |x: u32, y: u32| AnnotationPoint {
    x: f64::from(x),
    y: f64::from(y),
  };
  let mut annotation = new_redact("pin".to_owned(), point(LEFT, TOP), None);
  if let AnnotationShape::Redact { end, seed, .. } = &mut annotation.shape {
    *end = point(LEFT + text.box_width(), TOP + text.box_height());
    *seed = 0x2545_f491;
  }
  annotation.style.redaction = mode;
  annotation.style.width = f64::from(block);
  let (width, height) = (text.width(), text.height());
  let mut output = crate::screenshots::test_output_settings(width, height);
  output.background_type = "solid".to_owned();
  output.crop_height = f64::from(height);
  output.crop_width = f64::from(width);
  output.crop_x = 0.0;
  output.crop_y = 0.0;
  output.image_width = f64::from(width);
  output.image_x = 0.0;
  output.image_y = 0.0;
  output.annotations = vec![annotation];
  crate::screenshots::compose_output_layers(
    &text.picture(secret),
    &output,
    0.0,
    false,
    None,
    None,
    None,
    None,
    false,
    false,
  )
  .unwrap()
  .rgba
}

/// What the redaction shows, zone by zone: whether any of its blocks differs
/// at all from the surface beside the box.
fn redacted_zones(text: Text, secret: u32, block: u32) -> Vec<bool> {
  let rgba = composed(text, secret, AnnotationRedaction::Pixelate, block);
  let width = text.width();
  let at = |x: u32, y: u32| {
    let offset = ((y * width + x) * 4) as usize;
    [rgba[offset], rgba[offset + 1], rgba[offset + 2]]
  };
  let surface = at(LEFT - 4, TOP);
  let zone = text.zone(block);
  let across = text.box_width().div_ceil(zone);
  let mut zones = vec![false; (across * text.box_height().div_ceil(zone)) as usize];
  for y in (0..text.box_height()).step_by(block as usize) {
    for x in (0..text.box_width()).step_by(block as usize) {
      if at(LEFT + x, TOP + y) != surface {
        zones[((y / zone) * across + x / zone) as usize] = true;
      }
    }
  }
  zones
}

/// How many guesses the attack needs: the candidates at least as likely as
/// the secret. A zone that shows colour held ink for certain, so a candidate
/// without ink there is out. A zone that shows none most likely held none,
/// though an inked zone can come out plain when its blocks are left blank or
/// blended too lightly to show, so each such zone a candidate needs inked
/// makes it less likely rather than impossible.
fn guesses(text: Text, zone: u32, observed: &[bool], secret: u32) -> usize {
  // Zones the candidate inks that showed plain, or `None` where it cannot
  // have been the secret at all.
  let misses = |pin: u32| {
    let mut misses = 0;
    for (cell, count) in text.cells(&text.ink(pin), zone).into_iter().enumerate() {
      match (text.inked(zone, cell, count), observed[cell]) {
        (false, true) => return None,
        (true, false) => misses += 1,
        _ => {}
      }
    }
    Some(misses)
  };
  let truth = misses(secret).expect("the secret contradicts its own redaction");
  (0..10_000)
    .filter(|pin| misses(*pin).is_some_and(|misses| misses <= truth))
    .count()
}

/// The redaction's pixelation, attacked through the real compositor. Its
/// zones are as tall as the box, so each spans more than a digit however
/// large the type is against the blocks, and the attack learns nothing: it
/// still needs every one of the ten thousand guesses, for bold type, for
/// type a pixel thin, and for thin type twice the size, at every block size.
/// Ordinary pixelation of the same PINs needed ten at most.
#[test]
fn pixelate_leaves_the_attack_nothing_to_narrow() {
  for text in TEXTS {
    for block in [8, 12, 16] {
      for secret in SECRETS {
        let observed = redacted_zones(text, secret, block);
        let needed = guesses(text, text.zone(block), &observed, secret);
        assert_eq!(
          needed, 10_000,
          "{text:?} block {block}: {secret:04} found within {needed} guesses"
        );
      }
    }
  }
}
