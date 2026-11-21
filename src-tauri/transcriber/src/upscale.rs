// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Small pictures drawn four times as large with Real-ESRGAN's x4plus anime
//! 6B model (Xintao Wang, BSD-3-Clause), made for flat drawn art: cursors,
//! icons, stickers. Stretched, they blur; drawn by the model, their edges
//! stay sharp and their colours true.
//!
//! The model draws colour alone. A picture with transparency is drawn twice,
//! over white and over black: where the two agree the picture is opaque, and
//! how far apart they are says how see-through it is, so its edges come out
//! as sharp as its colours, with no fringe on any background. The picture is
//! drawn a tile at a time, each read with a margin round it so the tiles meet
//! without seams, which keeps a large picture within the GPU's memory and
//! gives progress to tell.

use std::path::Path;

use ort::session::Session;
use ort::value::TensorRef;

/// The feature's name in what the helper reports.
const WHAT: &str = "Upscaling";
/// How much larger the model draws a picture, across and down.
const SCALE: usize = 4;
/// Each tile's side in the picture's pixels, before its margin.
const TILE: usize = 128;
/// The margin read round each tile and dropped from what it draws, wide
/// enough that the model sees past the tile's edge.
const MARGIN: usize = 16;

/// Draws `image`, `size` pixels of red, green, blue and alpha bytes, four
/// times as large with the model at `model`, into `output` in the same form,
/// telling `progress` how far it has got.
pub fn upscale(
  model: &Path,
  image: &Path,
  size: (u32, u32),
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let (width, height) = (size.0 as usize, size.1 as usize);
  let rgba =
    std::fs::read(image).map_err(|error| format!("Could not read the picture: {error}"))?;
  if width == 0 || height == 0 || rgba.len() != width * height * 4 {
    return Err("The picture's size does not match its pixels".to_owned());
  }
  let mut session = crate::session::session(model, WHAT)?;
  let opaque = rgba.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 255);
  // An opaque picture looks the same over any background, so is drawn once.
  let backgrounds: &[f32] = if opaque { &[0.0] } else { &[1.0, 0.0] };
  let tiles = width.div_ceil(TILE) * height.div_ceil(TILE);
  let runs = (tiles * backgrounds.len()) as f32;
  let mut done = 0.0;
  let mut drawn = Vec::with_capacity(backgrounds.len());
  for &background in backgrounds {
    let over = composite(&rgba, background);
    drawn.push(draw(&mut session, &over, (width, height), &mut || {
      done += 1.0;
      progress(done / runs);
    })?);
  }
  let pixels = if let [white, black] = drawn.as_slice() {
    unmatte(white, black)
  } else {
    drawn[0]
      .as_chunks::<3>()
      .0
      .iter()
      .flat_map(|rgb| [byte(rgb[0]), byte(rgb[1]), byte(rgb[2]), 255])
      .collect()
  };
  std::fs::write(output, pixels).map_err(|error| format!("Could not write the picture: {error}"))
}

/// The picture laid over a flat `background`, 0 black to 1 white, as red,
/// green and blue from 0 to 1, row by row.
fn composite(rgba: &[u8], background: f32) -> Vec<f32> {
  rgba
    .as_chunks::<4>()
    .0
    .iter()
    .flat_map(|pixel| {
      let alpha = f32::from(pixel[3]) / 255.0;
      [0, 1, 2]
        .map(|channel| f32::from(pixel[channel]) / 255.0 * alpha + background * (1.0 - alpha))
    })
    .collect()
}

/// `rgb`, `size` pixels, four times as large, as red, green and blue from
/// 0 to 1, row by row, telling `tile_done` after each tile.
fn draw(
  session: &mut Session,
  rgb: &[f32],
  (width, height): (usize, usize),
  tile_done: &mut dyn FnMut(),
) -> Result<Vec<f32>, String> {
  let out_width = width * SCALE;
  let mut out = vec![0.0_f32; out_width * height * SCALE * 3];
  for top in (0..height).step_by(TILE) {
    for left in (0..width).step_by(TILE) {
      let inner = (
        left,
        top,
        (left + TILE).min(width),
        (top + TILE).min(height),
      );
      let read = (
        inner.0.saturating_sub(MARGIN),
        inner.1.saturating_sub(MARGIN),
        (inner.2 + MARGIN).min(width),
        (inner.3 + MARGIN).min(height),
      );
      let (read_width, read_height) = (read.2 - read.0, read.3 - read.1);
      // The model reads planes: all the red, then the green, then the blue.
      let mut planes = vec![0.0_f32; 3 * read_width * read_height];
      for y in 0..read_height {
        for x in 0..read_width {
          let from = ((read.1 + y) * width + read.0 + x) * 3;
          for channel in 0..3 {
            planes[(channel * read_height + y) * read_width + x] = rgb[from + channel];
          }
        }
      }
      let input = TensorRef::from_array_view(([1, 3, read_height, read_width], planes.as_slice()))
        .map_err(failed)?;
      let outputs = session
        .run(ort::inputs!["image" => input])
        .map_err(failed)?;
      let (shape, values) = outputs[0].try_extract_tensor::<f32>().map_err(failed)?;
      let (drawn_height, drawn_width) = (read_height * SCALE, read_width * SCALE);
      if shape.len() != 4 || shape[2] as usize != drawn_height || shape[3] as usize != drawn_width {
        return Err("The upscaling model drew a picture of another size".to_owned());
      }
      // Only the tile itself is kept; its margin was there for the model.
      for y in (inner.1 - read.1) * SCALE..(inner.3 - read.1) * SCALE {
        for x in (inner.0 - read.0) * SCALE..(inner.2 - read.0) * SCALE {
          let to = ((read.1 * SCALE + y) * out_width + read.0 * SCALE + x) * 3;
          for channel in 0..3 {
            out[to + channel] = values[(channel * drawn_height + y) * drawn_width + x];
          }
        }
      }
      tile_done();
    }
  }
  Ok(out)
}

/// The picture drawn over white and over black, as red, green, blue and
/// alpha bytes: how see-through each pixel is from how far apart the two are,
/// and its colour from the one over black, where the background added none.
fn unmatte(white: &[f32], black: &[f32]) -> Vec<u8> {
  white
    .as_chunks::<3>()
    .0
    .iter()
    .zip(black.as_chunks::<3>().0)
    .flat_map(|(over_white, over_black)| {
      let apart = (0..3)
        .map(|channel| over_white[channel] - over_black[channel])
        .sum::<f32>()
        / 3.0;
      let alpha = (1.0 - apart).clamp(0.0, 1.0);
      if alpha < 1.0 / 255.0 {
        return [0; 4];
      }
      [
        byte(over_black[0] / alpha),
        byte(over_black[1] / alpha),
        byte(over_black[2] / alpha),
        byte(alpha),
      ]
    })
    .collect()
}

fn byte(value: f32) -> u8 {
  (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn failed(error: ort::Error) -> String {
  format!("{WHAT} could not run: {error}")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_pixel_drawn_alike_over_white_and_black_is_opaque() {
    let red = [0.8, 0.1, 0.1];
    assert_eq!(unmatte(&red, &red), vec![204, 26, 26, 255]);
  }

  #[test]
  fn half_see_through_white_keeps_its_colour_and_half_its_alpha() {
    // White at half alpha: over white it is white, over black mid grey.
    assert_eq!(
      unmatte(&[1.0, 1.0, 1.0], &[0.5, 0.5, 0.5]),
      vec![255, 255, 255, 128]
    );
  }

  #[test]
  fn nothing_over_either_background_is_clear() {
    assert_eq!(
      unmatte(&[1.0, 1.0, 1.0], &[0.0, 0.0, 0.0]),
      vec![0, 0, 0, 0]
    );
  }
}
