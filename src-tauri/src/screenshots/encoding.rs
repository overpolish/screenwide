// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[path = "encoding/tests.rs"]
mod tests;

use std::collections::HashMap;
use std::io::Cursor;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use quantette::deps::palette::{cast::from_component_slice, Srgb};
use quantette::{ImageRef, Pipeline, QuantizeMethod};

use super::CapturedImage;

/// The largest palette an 8-bit indexed PNG can carry.
const MAX_PALETTE: usize = 256;
/// Applies an antialiased alpha mask without changing the captured dimensions.
pub fn rounded_corners(image: &CapturedImage, radius_percent: f64) -> CapturedImage {
  let radius = f64::from(image.width.min(image.height)) * radius_percent.clamp(0.0, 50.0) / 100.0;
  let mut rgba = image.rgba.clone();
  if radius <= 0.0 {
    return CapturedImage {
      height: image.height,
      rgba,
      width: image.width,
    };
  }

  for y in 0..image.height {
    for x in 0..image.width {
      let pixel_x = f64::from(x) + 0.5;
      let pixel_y = f64::from(y) + 0.5;
      let center_x = if pixel_x < radius {
        Some(radius)
      } else if pixel_x > f64::from(image.width) - radius {
        Some(f64::from(image.width) - radius)
      } else {
        None
      };
      let center_y = if pixel_y < radius {
        Some(radius)
      } else if pixel_y > f64::from(image.height) - radius {
        Some(f64::from(image.height) - radius)
      } else {
        None
      };
      let (Some(center_x), Some(center_y)) = (center_x, center_y) else {
        continue;
      };
      let distance = (pixel_x - center_x).hypot(pixel_y - center_y);
      let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
      let alpha = &mut rgba[((y * image.width + x) * 4 + 3) as usize];
      *alpha = (f64::from(*alpha) * coverage).round() as u8;
    }
  }

  CapturedImage {
    height: image.height,
    rgba,
    width: image.width,
  }
}

fn is_opaque(rgba: &[u8]) -> bool {
  rgba.chunks_exact(4).all(|pixel| pixel[3] == u8::MAX)
}

fn rgb_from_rgba(rgba: &[u8]) -> Vec<u8> {
  let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
  for pixel in rgba.chunks_exact(4) {
    rgb.extend_from_slice(&pixel[..3]);
  }
  rgb
}

/// An exact palette, for a still that already fits inside 256 colours.
///
/// A terminal, a solid window or a flat UI is bit-exact this way, rather than
/// being handed to a quantizer that can only approximate it. Anything richer
/// bails on the 257th colour, which for a real screenshot happens within the
/// first handful of pixels, so this costs nothing in the common case.
fn exact_palette(rgb: &[u8]) -> Option<(Vec<[u8; 3]>, Vec<u8>)> {
  let mut palette: Vec<[u8; 3]> = Vec::new();
  let mut index_of: HashMap<[u8; 3], u8> = HashMap::new();
  let mut indices = Vec::with_capacity(rgb.len() / 3);

  for pixel in rgb.chunks_exact(3) {
    let colour = [pixel[0], pixel[1], pixel[2]];
    let index = match index_of.get(&colour) {
      Some(index) => *index,
      None => {
        if palette.len() >= MAX_PALETTE {
          return None;
        }
        let index = u8::try_from(palette.len()).ok()?;
        palette.push(colour);
        index_of.insert(colour, index);
        index
      }
    };
    indices.push(index);
  }

  Some((palette, indices))
}

/// Reduces the still to a 256-colour palette.
///
/// k-means with Floyd-Steinberg dithering: the dithering is what keeps
/// wallpapers and photos from banding, and k-means is both markedly more
/// faithful than Wu and, run in parallel, faster than encoding the image
/// losslessly would have been.
fn quantized_palette(rgb: &[u8], width: u32, height: u32) -> Option<(Vec<[u8; 3]>, Vec<u8>)> {
  let colours = from_component_slice::<Srgb<u8>>(rgb);
  let image = ImageRef::new(width, height, colours).ok()?;
  let (palette, indices) = Pipeline::new()
    .quantize_method(QuantizeMethod::kmeans())
    .parallel(true)
    .input_image(image)
    .output_srgb8_indexed_image()
    .into_parts();

  Some((
    palette
      .into_iter()
      .map(|colour| [colour.red, colour.green, colour.blue])
      .collect(),
    indices,
  ))
}

fn encode_indexed_png(
  palette: &[[u8; 3]],
  indices: &[u8],
  width: u32,
  height: u32,
) -> Result<Vec<u8>, String> {
  let mut png = Vec::new();
  {
    let mut encoder = png::Encoder::new(Cursor::new(&mut png), width, height);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    encoder.set_palette(palette.iter().flatten().copied().collect::<Vec<u8>>());
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer
      .write_image_data(indices)
      .map_err(|error| error.to_string())?;
  }

  Ok(png)
}

pub(crate) fn encode_truecolor_png(image: &CapturedImage) -> Result<Vec<u8>, String> {
  let mut png = Vec::new();
  PngEncoder::new_with_quality(
    Cursor::new(&mut png),
    CompressionType::Default,
    FilterType::Sub,
  )
  .write_image(
    &image.rgba,
    image.width,
    image.height,
    ExtendedColorType::Rgba8,
  )
  .map_err(|error| error.to_string())?;

  Ok(png)
}

/// Encodes a still as PNG bytes.
///
/// Deliberately one function, so the compression backend can be swapped
/// without capture or saving noticing. A screen capture is almost always fully
/// opaque, so the alpha channel is dropped and the image is written as an
/// 8-bit indexed PNG - that indexing is where the size comes from, not the
/// quantization on its own. A capture that is not opaque keeps every channel
/// and is written losslessly instead, because the palette path has no alpha.
///
/// Measured on a synthetic 3456x2234 desktop: 4.57 MB originally, 2.78 MB
/// losslessly, 0.57 MB here, in ~390ms.
pub fn encode_png(image: &CapturedImage) -> Result<Vec<u8>, String> {
  if !is_opaque(&image.rgba) {
    return encode_truecolor_png(image);
  }

  let rgb = rgb_from_rgba(&image.rgba);
  let (palette, indices) = exact_palette(&rgb)
    .or_else(|| quantized_palette(&rgb, image.width, image.height))
    .ok_or_else(|| "The capture could not be reduced to a palette".to_owned())?;

  encode_indexed_png(&palette, &indices, image.width, image.height)
}
