// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading each line found: cut out of the picture, scaled to the
//! recognition model's height, and read in batches of lines of a like shape,
//! so a batch wastes little on padding. Each batch is padded to a width that
//! is a multiple of [`WIDTH_STEP`], so the model meets few distinct shapes.

use screenwide_transcriber::TextLine;

use super::decode::{self, Dictionary, Reading};
use super::detect::TextBox;
use super::picture::Picture;

/// The height the model reads a line at, in pixels.
pub(super) const HEIGHT: usize = 48;
/// The narrowest a batch is: the width the model was made at.
const LEAST_WIDTH: usize = 320;
/// What a batch's width is rounded up to.
const WIDTH_STEP: usize = 64;
/// The most lines read at once: the fastest through DirectML on a full
/// screen of lines.
const BATCH: usize = 16;
/// How sure a reading must be to be kept.
const LEAST_CONFIDENCE: f32 = 0.5;

/// A line cut out of the picture, in whole pixels.
#[derive(Clone, Copy)]
pub(super) struct Crop {
  pub found: TextBox,
  pub region: [usize; 4],
}

impl Crop {
  pub(super) fn of(found: TextBox, picture: &Picture<'_>) -> Self {
    let left = (found.left.round() as usize).min(picture.width - 1);
    let top = (found.top.round() as usize).min(picture.height - 1);
    let right = (found.right.round() as usize).clamp(left + 1, picture.width);
    let bottom = (found.bottom.round() as usize).clamp(top + 1, picture.height);
    Self {
      found,
      region: [left, top, right - left, bottom - top],
    }
  }

  fn aspect(&self) -> f32 {
    self.region[2] as f32 / self.region[3] as f32
  }

  /// The width the line is read at, at the model's height.
  fn scaled_width(&self) -> usize {
    ((HEIGHT as f32 * self.aspect()).ceil() as usize).max(1)
  }
}

/// The crops in batches of up to [`BATCH`] of a like shape, each with the
/// width it is padded to.
pub(super) fn batches(crops: &[Crop]) -> Vec<(Vec<usize>, usize)> {
  let mut order = (0..crops.len()).collect::<Vec<_>>();
  order.sort_by(|&a, &b| crops[a].aspect().total_cmp(&crops[b].aspect()));
  order
    .chunks(BATCH)
    .map(|batch| {
      let widest = batch
        .iter()
        .map(|&index| crops[index].scaled_width())
        .max()
        .unwrap_or(LEAST_WIDTH);
      let width = widest.max(LEAST_WIDTH).div_ceil(WIDTH_STEP) * WIDTH_STEP;
      (batch.to_vec(), width)
    })
    .collect()
}

/// The batch's lines as the model takes them: three planes each, blue,
/// green and red, scaled to -1 to 1, padded on the right with 0.
pub(super) fn model_input(
  picture: &Picture<'_>,
  crops: &[Crop],
  batch: &[usize],
  width: usize,
) -> Vec<f32> {
  let plane = HEIGHT * width;
  let mut input = vec![0.0; batch.len() * 3 * plane];
  for (slot, &index) in batch.iter().enumerate() {
    let crop = &crops[index];
    let scaled = crop.scaled_width().min(width);
    let base = slot * 3 * plane;
    picture.resample(crop.region, (scaled, HEIGHT), |x, y, bgr| {
      for channel in 0..3 {
        input[base + channel * plane + y * width + x] = bgr[channel] / 127.5 - 1.0;
      }
    });
  }
  input
}

/// The line `crop` reads as from the model's `chances` for it, `steps` rows
/// across a batch `width` wide, or nothing where it read nothing sure.
pub(super) fn line(
  crop: &Crop,
  chances: &[f32],
  (steps, width): (usize, usize),
  dictionary: &Dictionary,
) -> Option<TextLine> {
  let Reading {
    symbols,
    confidence,
  } = decode::read(chances, dictionary);
  if confidence < LEAST_CONFIDENCE {
    return None;
  }
  let found = crop.found;
  let line_width = found.right - found.left;
  // A step covers `width / steps` of the padded line, of which the line
  // itself took its scaled width.
  let scaled = crop.scaled_width().min(width) as f32;
  let to_x = |step: f32| {
    let across = step * width as f32 / steps as f32 / scaled;
    found.left + across.clamp(0.0, 1.0) * line_width
  };
  let mut text = String::new();
  let mut characters = Vec::new();
  for (index, symbol) in symbols.iter().enumerate() {
    let start = match index {
      0 => found.left,
      _ => to_x((symbols[index - 1].steps.1 + symbol.steps.0) as f32 / 2.0),
    };
    let end = match symbols.get(index + 1) {
      Some(next) => to_x((symbol.steps.1 + next.steps.0) as f32 / 2.0),
      None => found.right,
    };
    for character in symbol.text.chars() {
      text.push(character);
      characters.push([start, end]);
    }
  }
  trim(&mut text, &mut characters);
  (!text.is_empty()).then_some(TextLine {
    text,
    confidence,
    bounds: [found.left, found.top, line_width, found.bottom - found.top],
    characters,
  })
}

/// Drops the spaces read at either end, with their boxes.
fn trim(text: &mut String, characters: &mut Vec<[f32; 2]>) {
  let leading = text.chars().take_while(|c| c.is_whitespace()).count();
  let kept = text.chars().count() - leading;
  let trailing = text
    .chars()
    .rev()
    .take_while(|c| c.is_whitespace())
    .count()
    .min(kept);
  *text = text.chars().skip(leading).take(kept - trailing).collect();
  characters.drain(..leading);
  characters.truncate(kept - trailing);
}
