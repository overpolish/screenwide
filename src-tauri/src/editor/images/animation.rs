// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An animated picture - a GIF, an animated PNG or an animated WebP - as a
//! image plays it: its frames, each composed over the ones before as the
//! format says, and when each one starts.
//!
//! The frames are held decoded, so playing one never waits on a decoder.
//! That is what bounds them: each is shrunk to at most [`MAX_SIDE`], which is
//! more than an image that moves is usually shown at, and a picture whose
//! frames would take more than [`FRAME_BYTES`] keeps the run that fits.

use std::io::Cursor;
use std::sync::Arc;

use image::codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder};
use image::{AnimationDecoder, Frames, ImageFormat, RgbaImage};

use super::store::fitted;
use crate::editor::annotations::image::play::frame_at;

/// The longest side a moving picture's frames are held at.
const MAX_SIDE: u32 = 512;
/// The most bytes one picture's decoded frames may take.
const FRAME_BYTES: usize = 96 << 20;
/// Delays this short are a GIF that asks for no delay at all, which every
/// browser shows at a tenth of a second instead.
const SHORTEST_DELAY_MS: u32 = 10;
const STAND_IN_DELAY_MS: u32 = 100;

/// A moving picture's frames, premultiplied, and when each starts in one run.
pub(crate) struct ImageAnimation {
  pub(crate) frames: Vec<Arc<RgbaImage>>,
  /// When each frame starts, in milliseconds from the start of a run.
  pub(crate) starts: Vec<u32>,
  /// How long one run lasts, in milliseconds.
  pub(crate) cycle_ms: u32,
  /// The picture's own size, before its frames were shrunk to be held.
  pub(crate) canvas: (u32, u32),
}

impl ImageAnimation {
  /// The bytes its frames take.
  pub(crate) fn bytes(&self) -> usize {
    self.frames.iter().map(|frame| frame.len()).sum()
  }

  /// Which frame shows: see [`frame_at`].
  pub(crate) fn frame_index(&self, start: u32, clock_ms: Option<f32>, once: bool) -> usize {
    frame_at(&self.starts, self.cycle_ms, start, clock_ms, once)
  }
}

/// The file extension a moving picture in `format` is kept under.
pub(crate) fn extension(format: ImageFormat) -> Option<&'static str> {
  match format {
    ImageFormat::Gif => Some("gif"),
    ImageFormat::Png => Some("png"),
    ImageFormat::WebP => Some("webp"),
    _ => None,
  }
}

/// The frames of the picture in `bytes`, where it is one that moves: a GIF,
/// PNG or WebP of more than one frame. `None` for anything else, which is
/// read as a still.
pub(crate) fn decode(bytes: &[u8]) -> Option<ImageAnimation> {
  let frames: Frames<'_> = match image::guess_format(bytes).ok()? {
    ImageFormat::Gif => GifDecoder::new(Cursor::new(bytes)).ok()?.into_frames(),
    ImageFormat::Png => {
      let decoder = PngDecoder::new(Cursor::new(bytes)).ok()?;
      if !decoder.is_apng().ok()? {
        return None;
      }
      decoder.apng().ok()?.into_frames()
    }
    ImageFormat::WebP => {
      let decoder = WebPDecoder::new(Cursor::new(bytes)).ok()?;
      if !decoder.has_animation() {
        return None;
      }
      decoder.into_frames()
    }
    _ => return None,
  };
  let mut animation = ImageAnimation {
    frames: Vec::new(),
    starts: Vec::new(),
    cycle_ms: 0,
    canvas: (0, 0),
  };
  let mut held = (0, 0);
  let mut bytes_held = 0;
  // A frame that cannot be read ends the run there rather than losing it.
  for frame in frames.map_while(Result::ok) {
    let (numerator, denominator) = frame.delay().numer_denom_ms();
    let delay = numerator / denominator.max(1);
    let delay = if delay <= SHORTEST_DELAY_MS {
      STAND_IN_DELAY_MS
    } else {
      delay
    };
    let buffer = frame.into_buffer();
    if animation.frames.is_empty() {
      animation.canvas = buffer.dimensions();
      let (width, height) = animation.canvas;
      let scale = (f64::from(MAX_SIDE) / f64::from(width.max(height).max(1))).min(1.0);
      held = (
        ((f64::from(width) * scale).round() as u32).max(1),
        ((f64::from(height) * scale).round() as u32).max(1),
      );
    }
    let kept = fitted(buffer, held.0, held.1);
    bytes_held += kept.len();
    if bytes_held > FRAME_BYTES && !animation.frames.is_empty() {
      break;
    }
    animation.starts.push(animation.cycle_ms);
    animation.cycle_ms = animation.cycle_ms.saturating_add(delay);
    animation.frames.push(Arc::new(kept));
  }
  (animation.frames.len() > 1).then_some(animation)
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use image::codecs::gif::GifEncoder;
  use image::{Delay, Frame, Rgba};

  /// A GIF of `colours`, each frame `delay_ms` long.
  pub(crate) fn gif(colours: &[[u8; 4]], delay_ms: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
      let mut encoder = GifEncoder::new(&mut bytes);
      for colour in colours {
        let frame = Frame::from_parts(
          RgbaImage::from_pixel(4, 2, Rgba(*colour)),
          0,
          0,
          Delay::from_numer_denom_ms(delay_ms, 1),
        );
        encoder.encode_frame(frame).expect("a frame");
      }
    }
    bytes
  }

  #[test]
  fn a_gif_plays_its_frames_for_as_long_as_each_asks() {
    let animation = decode(&gif(&[[255, 0, 0, 255], [0, 0, 255, 255]], 120)).expect("frames");
    assert_eq!(animation.canvas, (4, 2));
    assert_eq!(animation.starts, vec![0, 120]);
    assert_eq!(animation.cycle_ms, 240);
    assert_eq!(animation.frames[1].get_pixel(0, 0).0, [0, 0, 255, 255]);
    assert_eq!(animation.frame_index(0, Some(130.0), false), 1);
  }

  #[test]
  fn a_gif_asking_for_no_delay_plays_at_a_tenth_of_a_second() {
    let animation = decode(&gif(&[[255, 0, 0, 255], [0, 0, 255, 255]], 0)).expect("frames");
    assert_eq!(animation.cycle_ms, 2 * STAND_IN_DELAY_MS);
  }

  #[test]
  fn a_picture_of_one_frame_is_a_still() {
    assert!(decode(&gif(&[[255, 0, 0, 255]], 100)).is_none());
  }
}
