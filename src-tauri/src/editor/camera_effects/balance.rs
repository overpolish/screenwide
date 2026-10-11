// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Enhance's white balance and exposure, worked out once for the
//! whole recording from frames sampled across it, so the camera never
//! flickers between corrections.
//!
//! Skin and the room each judge what they can. A cast of green or magenta
//! turns skin off the skin line, so skin judges that; warm and cool mostly
//! move skin along the line, so the room behind the person judges those,
//! held to a bound unless much of the room is plainly grey. Every estimate,
//! the room's included, is then judged by the skin it leaves, and one that
//! drains skin of its colour is turned away. Only hue is read from skin,
//! never how dark it is, so every skin tone is treated alike.
//!
//! The exposure lift is the same for everyone in the frame and never
//! darkens: it brings an underexposed frame's usual highlights towards white
//! rather than aiming a face at a brightness, which would push darker skin
//! towards lighter.

mod filters;
mod skin;

use self::filters::{edges, eroded};
use super::look::{to_display, to_linear, LUMA};

/// The matte's share of a pixel below which it is the room, and above which
/// the person's inside.
const ROOM_BELOW: u8 = 26;
const INSIDE_ABOVE: u8 = 229;
/// Where the person is not, eroded so their soft edge is left out.
const BEHIND: u8 = 1;
/// Where the person surely is, eroded so the room at their edge is left out.
const INNER: u8 = 2;
/// The frame the pixel counts below were set for; a frame of another size
/// takes the same shares of its pixels.
const TUNED_PIXELS: f64 = 960.0 * 540.0;

/// The gains, in linear light, that correct a recording sampled as `rgb`
/// frames of `size` back to back, judged with the person where their
/// `matte` frames are given: worked out on each frame alone, then the median
/// across them, so a few odd frames, a hand over the face or nobody in
/// shot, do not sway it. Without a matte each frame is both room and person.
pub(super) fn correction(
  rgb: &[u8],
  matte: Option<&[u8]>,
  (width, height): (usize, usize),
) -> [f64; 3] {
  let pixels = (width * height).max(1);
  let table: Vec<f32> = (0..=255u8)
    .map(|value| to_linear(f32::from(value) / 255.0))
    .collect();
  let frames: Vec<[f64; 3]> = rgb
    .chunks_exact(pixels * 3)
    .enumerate()
    .map_while(|(index, frame)| {
      let matte = match matte {
        Some(matte) => Some(matte.get(index * pixels..(index + 1) * pixels)?),
        None => None,
      };
      Some(Sample::new(frame, matte, (width, height), &table).correction())
    })
    .collect();
  if frames.is_empty() {
    return [1.0; 3];
  }
  [0, 1, 2].map(|channel| {
    let mut logs: Vec<f64> = frames.iter().map(|gains| gains[channel].ln()).collect();
    let middle = logs.len() / 2;
    logs.select_nth_unstable_by(middle, f64::total_cmp).1.exp()
  })
}

/// One frame sampled from the recording, with where each pixel is the room
/// and where it is the person.
struct Sample {
  linear: Vec<[f32; 3]>,
  /// How fast each channel changes at each pixel, for gray-edge.
  edges: Vec<[f32; 3]>,
  places: Vec<u8>,
}

impl Sample {
  /// `rgb` of `size`, made linear by `table`, and where given its `matte`.
  fn new(rgb: &[u8], matte: Option<&[u8]>, (width, height): (usize, usize), table: &[f32]) -> Self {
    let linear: Vec<[f32; 3]> = rgb
      .as_chunks::<3>()
      .0
      .iter()
      .map(|pixel| pixel.map(|value| table[usize::from(value)]))
      .collect();
    let places = match matte {
      Some(matte) => {
        let behind = eroded(
          matte.iter().map(|value| *value < ROOM_BELOW).collect(),
          (width, height),
          2,
        );
        let inner = eroded(
          matte.iter().map(|value| *value > INSIDE_ABOVE).collect(),
          (width, height),
          1,
        );
        behind
          .into_iter()
          .zip(inner)
          .map(|(behind, inner)| (u8::from(behind) * BEHIND) | (u8::from(inner) * INNER))
          .collect()
      }
      None => vec![BEHIND | INNER; width * height],
    };
    Self {
      edges: edges(&linear, (width, height)),
      linear,
      places,
    }
  }

  /// The gains, in linear light with green at 1 before the exposure lift,
  /// that correct this frame.
  fn correction(&self) -> [f64; 3] {
    let room = self.room();
    let candidates = [
      self.gray_edge(&|lin, _| usable(lin)),
      Some(room),
      Some(self.combined(room)),
    ];
    let exposure = self.exposure();
    self.judged(&candidates).map(|gain| gain * exposure)
  }

  /// Whether `count` pixels are as large a share of this frame as `tuned`
  /// were of the frame they were set for.
  fn enough(&self, count: usize, tuned: f64) -> bool {
    count as f64 >= tuned / TUNED_PIXELS * self.linear.len() as f64
  }

  fn count(&self, keep: &dyn Fn([f32; 3], u8) -> bool) -> usize {
    self
      .linear
      .iter()
      .zip(&self.places)
      .filter(|(lin, place)| keep(**lin, **place))
      .count()
  }

  /// Shades-of-gray: the light's colour as the `p`-mean of the kept pixels.
  fn shades_of_gray(&self, keep: &dyn Fn([f32; 3], u8) -> bool, p: i32) -> Option<[f64; 3]> {
    let mut sum = [0.0; 3];
    let mut count = 0usize;
    for (lin, place) in self.linear.iter().zip(&self.places) {
      if keep(*lin, *place) && usable(*lin) {
        for (sum, value) in sum.iter_mut().zip(lin) {
          *sum += f64::from(*value).powi(p);
        }
        count += 1;
      }
    }
    (count > 0).then(|| normalised(sum.map(|sum| (sum / count as f64).powf(1.0 / f64::from(p)))))
  }

  /// Gray-edge: the light's colour as the 6-mean of how fast each channel
  /// changes, so a large single-coloured wall weighs little.
  fn gray_edge(&self, keep: &dyn Fn([f32; 3], u8) -> bool) -> Option<[f64; 3]> {
    let mut sum = [0.0; 3];
    let mut count = 0usize;
    for ((lin, place), edge) in self.linear.iter().zip(&self.places).zip(&self.edges) {
      if keep(*lin, *place) && usable(*lin) {
        for (sum, value) in sum.iter_mut().zip(edge) {
          *sum += f64::from(*value).powi(6);
        }
        count += 1;
      }
    }
    (count > 0).then(|| normalised(sum.map(|sum| (sum / count as f64).powf(1.0 / 6.0))))
  }

  /// The room past the person, its strongly coloured parts left out, as the
  /// mean of shades-of-gray and gray-edge, which fail in different ways: one
  /// on a large coloured wall, the other on a plain backdrop.
  fn room(&self) -> [f64; 3] {
    let behind = |_: [f32; 3], place: u8| place & BEHIND != 0;
    let first = self.shades_of_gray(&behind, 4).unwrap_or([1.0; 3]);
    let calm = move |lin: [f32; 3], place: u8| behind(lin, place) && saturation(lin, first) < 0.35;
    let calm: &dyn Fn([f32; 3], u8) -> bool = if self.enough(self.count(&calm), 500.0) {
      &calm
    } else {
      &behind
    };
    match (self.shades_of_gray(calm, 4), self.gray_edge(calm)) {
      (Some(gray), Some(edge)) => [0, 1, 2].map(|index| (gray[index] * edge[index]).sqrt()),
      (gray, edge) => gray.or(edge).unwrap_or([1.0; 3]),
    }
  }

  /// The brightest usual highlights brought up towards white, by at most a
  /// third again.
  fn exposure(&self) -> f64 {
    let mut luma: Vec<f32> = self
      .linear
      .iter()
      .map(|lin| {
        lin
          .iter()
          .zip(LUMA)
          .map(|(value, weight)| value * weight)
          .sum()
      })
      .collect();
    if luma.is_empty() {
      return 1.0;
    }
    let at = (luma.len() - 1) * 995 / 1000;
    let top = *luma.select_nth_unstable_by(at, f32::total_cmp).1;
    (0.85 / f64::from(top).max(1e-4)).clamp(1.0, 1.33)
  }
}

/// Neither clipped nor nearly black.
fn usable(lin: [f32; 3]) -> bool {
  let top = lin[0].max(lin[1]).max(lin[2]);
  top < 0.95 && top > 0.02
}

/// How far from grey `lin` is once `gains` are applied, 0 to 1.
fn saturation(lin: [f32; 3], gains: [f64; 3]) -> f64 {
  let values = [0, 1, 2].map(|index| f64::from(lin[index]) * gains[index]);
  let top = values[0].max(values[1]).max(values[2]);
  let bottom = values[0].min(values[1]).min(values[2]);
  (top - bottom) / top.max(1e-6)
}

/// The gains that make the light `illuminant` grey, green kept.
fn normalised(illuminant: [f64; 3]) -> [f64; 3] {
  let illuminant = illuminant.map(|value| value.max(1e-6));
  illuminant.map(|value| illuminant[1] / value)
}

/// Gains as a move along warm to cool (blue against red) and along green
/// to magenta (green against both).
fn axes(gains: [f64; 3]) -> (f64, f64) {
  let [r, g, b] = gains.map(f64::ln);
  ((b - r) / 2.0, g - (r + b) / 2.0)
}

fn from_axes(temperature: f64, tint: f64) -> [f64; 3] {
  let gains = [
    (-temperature - tint / 3.0).exp(),
    (2.0 * tint / 3.0).exp(),
    (temperature - tint / 3.0).exp(),
  ];
  gains.map(|gain| gain / gains[1])
}

/// The display colour of linear `lin` once `gains` are applied.
fn displayed(lin: [f64; 3], gains: [f64; 3]) -> [f32; 3] {
  [0, 1, 2].map(|index| to_display((lin[index] * gains[index]) as f32))
}
