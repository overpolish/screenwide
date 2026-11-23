// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
mod tests;
mod webp;

pub use webp::write_webp;

use std::collections::HashMap;
use std::io::{Cursor, Write};

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use quantette::deps::palette::{cast::from_component_slice, Srgb};
use quantette::{ImageRef, Pipeline, QuantizeMethod};

use super::CapturedImage;

/// An empty vector with room for `length` values, or an error where there is
/// not the memory for them. A screenshot is as large as a scrolling capture
/// made it, so a buffer the size of one is asked for, not assumed.
pub fn reserved<T>(length: usize) -> Result<Vec<T>, String> {
  let mut values = Vec::new();
  values
    .try_reserve_exact(length)
    .map_err(|_| "There is not enough memory to hold the screenshot".to_owned())?;
  Ok(values)
}

/// The largest palette an 8-bit indexed PNG can carry.
const MAX_PALETTE: usize = 256;
/// Applies an antialiased alpha mask without changing the captured dimensions,
/// its corners `radius` pixels round, at most half the shorter side.
pub fn rounded_corners(image: &CapturedImage, radius: f64) -> CapturedImage {
  let radius = radius.clamp(0.0, f64::from(image.width.min(image.height)) / 2.0);
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

/// Whether every pixel is fully opaque, so nothing shows through it.
pub fn is_opaque(rgba: &[u8]) -> bool {
  rgba
    .as_chunks::<4>()
    .0
    .iter()
    .all(|pixel| pixel[3] == u8::MAX)
}

fn rgb_from_rgba(rgba: &[u8]) -> Result<Vec<u8>, String> {
  let mut rgb = reserved(rgba.len() / 4 * 3)?;
  for pixel in rgba.as_chunks::<4>().0.iter() {
    rgb.extend_from_slice(&pixel[..3]);
  }
  Ok(rgb)
}

/// A palette of up to 256 colours, and each pixel's place in it.
type Indexed = (Vec<[u8; 3]>, Vec<u8>);

/// An exact palette, for a still that already fits inside 256 colours.
///
/// A terminal, a solid window or a flat UI is bit-exact this way, rather than
/// being handed to a quantizer that can only approximate it. Anything richer
/// bails on the 257th colour, which for a real screenshot happens within the
/// first handful of pixels, so this costs nothing in the common case. The
/// alpha `rgba` carries is passed over: only an opaque still is indexed.
fn exact_palette(rgba: &[u8]) -> Result<Option<Indexed>, String> {
  let mut palette: Vec<[u8; 3]> = Vec::new();
  let mut index_of: HashMap<[u8; 3], u8> = HashMap::new();
  let mut indices = reserved(rgba.len() / 4)?;

  for pixel in rgba.as_chunks::<4>().0.iter() {
    let colour = [pixel[0], pixel[1], pixel[2]];
    let index = match index_of.get(&colour) {
      Some(index) => *index,
      None => {
        let Some(index) = u8::try_from(palette.len())
          .ok()
          .filter(|_| palette.len() < MAX_PALETTE)
        else {
          return Ok(None);
        };
        palette.push(colour);
        index_of.insert(colour, index);
        index
      }
    };
    indices.push(index);
  }

  Ok(Some((palette, indices)))
}

/// Reduces the still to a 256-colour palette.
///
/// k-means with Floyd-Steinberg dithering: the dithering is what keeps
/// wallpapers and photos from banding, and k-means is both markedly more
/// faithful than Wu and, run in parallel, faster than encoding the image
/// losslessly would have been.
fn quantized_palette(rgb: &[u8], width: u32, height: u32) -> Option<Indexed> {
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

fn write_indexed_png(
  palette: &[[u8; 3]],
  indices: &[u8],
  (width, height): (u32, u32),
  out: impl Write,
) -> Result<(), String> {
  let mut encoder = png::Encoder::new(out, width, height);
  encoder.set_color(png::ColorType::Indexed);
  encoder.set_depth(png::BitDepth::Eight);
  encoder.set_compression(png::Compression::High);
  encoder.set_palette(palette.iter().flatten().copied().collect::<Vec<u8>>());
  let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
  writer
    .write_image_data(indices)
    .map_err(|error| error.to_string())?;
  writer.finish().map_err(|error| error.to_string())
}

/// The image as a lossless PNG, every channel kept, written to `out`.
pub fn write_truecolor_png(image: &CapturedImage, out: impl Write) -> Result<(), String> {
  PngEncoder::new_with_quality(out, CompressionType::Default, FilterType::Sub)
    .write_image(
      &image.rgba,
      image.width,
      image.height,
      ExtendedColorType::Rgba8,
    )
    .map_err(|error| error.to_string())
}

pub fn encode_truecolor_png(image: &CapturedImage) -> Result<Vec<u8>, String> {
  let mut png = Vec::new();
  write_truecolor_png(image, &mut png)?;
  Ok(png)
}

/// Writes a still as a PNG to `out` with every colour kept: an opaque still
/// drops only its alpha, which says nothing; any other keeps every channel.
pub fn write_lossless_png(image: &CapturedImage, out: impl Write) -> Result<(), String> {
  if !is_opaque(&image.rgba) {
    return write_truecolor_png(image, out);
  }
  let rgb = rgb_from_rgba(&image.rgba)?;
  PngEncoder::new_with_quality(out, CompressionType::Default, FilterType::Sub)
    .write_image(&rgb, image.width, image.height, ExtendedColorType::Rgb8)
    .map_err(|error| error.to_string())
}

/// Encodes `frames`, each `width` by `height` RGBA, as an animated PNG that
/// shows each for `frame_ms` and plays them over and over. A viewer decodes
/// it a frame at a time, where frames stacked in one picture are decoded
/// whole.
pub fn encode_animated_png(
  frames: &[Vec<u8>],
  width: u32,
  height: u32,
  frame_ms: u16,
) -> Result<Vec<u8>, String> {
  let count = u32::try_from(frames.len()).map_err(|error| error.to_string())?;
  let mut png = Vec::new();
  {
    let mut encoder = png::Encoder::new(Cursor::new(&mut png), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    // No plays counted means it loops for good.
    encoder
      .set_animated(count, 0)
      .map_err(|error| error.to_string())?;
    encoder
      .set_frame_delay(frame_ms, 1_000)
      .map_err(|error| error.to_string())?;
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    for frame in frames {
      writer
        .write_image_data(frame)
        .map_err(|error| error.to_string())?;
    }
    writer.finish().map_err(|error| error.to_string())?;
  }
  Ok(png)
}

/// Writes a still as a PNG to `out`.
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
pub fn write_png(image: &CapturedImage, out: impl Write) -> Result<(), String> {
  if !is_opaque(&image.rgba) {
    return write_truecolor_png(image, out);
  }
  let size = (image.width, image.height);
  if let Some((palette, indices)) = exact_palette(&image.rgba)? {
    return write_indexed_png(&palette, &indices, size, out);
  }
  let rgb = rgb_from_rgba(&image.rgba)?;
  let (palette, indices) = quantized_palette(&rgb, image.width, image.height)
    .ok_or_else(|| "The capture could not be reduced to a palette".to_owned())?;
  drop(rgb);
  write_indexed_png(&palette, &indices, size, out)
}

/// A still as PNG bytes; see [`write_png`].
pub fn encode_png(image: &CapturedImage) -> Result<Vec<u8>, String> {
  let mut png = Vec::new();
  write_png(image, &mut png)?;
  Ok(png)
}

/// How finely a screenshot saved as a JPEG keeps its detail: high enough
/// that text keeps clean edges, where the size still falls well below a PNG.
const JPEG_QUALITY: u8 = 90;

/// Writes the image as a JPEG to `out`. JPEG has no transparency, so
/// whatever the image leaves see-through, such as rounded corners, is laid
/// over white.
pub fn write_jpeg(image: &CapturedImage, out: impl Write) -> Result<(), String> {
  let over_white = |channel: u8, alpha: u8| {
    let (channel, alpha) = (u32::from(channel), u32::from(alpha));
    ((channel * alpha + 255 * (255 - alpha) + 127) / 255) as u8
  };
  let mut rgb = reserved(image.rgba.len() / 4 * 3)?;
  for &[red, green, blue, alpha] in image.rgba.as_chunks::<4>().0 {
    rgb.extend_from_slice(&[
      over_white(red, alpha),
      over_white(green, alpha),
      over_white(blue, alpha),
    ]);
  }
  image::codecs::jpeg::JpegEncoder::new_with_quality(out, JPEG_QUALITY)
    .write_image(&rgb, image.width, image.height, ExtendedColorType::Rgb8)
    .map_err(|error| error.to_string())
}
