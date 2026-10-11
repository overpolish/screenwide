// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects drawn on the CPU, for the pictures made of a camera
//! frame outside the live preview: the timeline's thumbnails and the stills.
//! The same effects as `camera_effects.wgsl`, on a decoded frame.

use ::image::imageops::{self, FilterType};
use ::image::RgbaImage;

use super::live::LiveCameraEffects;
use super::CameraBackground;
use crate::editor::annotations::pin::LumaFrame;
use crate::screenshots::CapturedImage;

/// How much smaller than the frame the room is blurred at: a still's blur is
/// wide and smooth, so a quarter size loses nothing of it, and a smooth
/// enlargement back keeps it free of the blocks a bilinear one leaves.
const SHRINK: u32 = 4;

/// The camera's effects for a run of stills, and the matte they read.
pub(crate) struct StillEffects {
  live: std::sync::Arc<LiveCameraEffects>,
  matte: Option<super::matte_frames::MatteFrames>,
}

impl StillEffects {
  /// The effects `live`, their matte read at most `longest_side` pixels on
  /// its longer side, or nothing where there are none.
  pub(crate) fn new(
    live: Option<std::sync::Arc<LiveCameraEffects>>,
    longest_side: u32,
  ) -> Option<Self> {
    let live = live?;
    let matte = live
      .matte
      .as_ref()
      .and_then(|file| super::matte_frames::MatteFrames::open(file, longest_side).ok());
    Some(Self { live, matte })
  }

  /// Draws the effects on `frame`, the camera at `ms`.
  pub(crate) fn apply(&mut self, frame: &mut CapturedImage, ms: u64) {
    let matte = self
      .matte
      .as_mut()
      .and_then(|matte| matte.at(ms).ok().flatten());
    apply(frame, matte.as_deref(), &self.live);
  }

  /// Whether the frames it draws keep a transparent background, which a
  /// JPEG cannot carry.
  pub(crate) fn is_transparent(&self) -> bool {
    self.live.background == CameraBackground::Transparent && self.matte.is_some()
  }
}

/// Draws `effects` on `frame`, with its matte `matte`. A background taken
/// away with no matte is left as recorded. Where there is no background the
/// frame keeps its transparency in its alpha.
pub(crate) fn apply(
  frame: &mut CapturedImage,
  matte: Option<&LumaFrame>,
  effects: &LiveCameraEffects,
) {
  let (width, height) = (frame.width, frame.height);
  if width == 0 || height == 0 {
    return;
  }
  let background = if matte.is_some() {
    effects.background
  } else {
    CameraBackground::Kept
  };
  let behind = match background {
    CameraBackground::Blurred => {
      RgbaImage::from_raw(width, height, frame.rgba.clone()).map(|room| {
        let small = imageops::resize(
          &room,
          (width / SHRINK).max(1),
          (height / SHRINK).max(1),
          FilterType::Triangle,
        );
        let deviation = super::gpu::room_blur_deviation((width, height)) / SHRINK as f32;
        let blurred = imageops::blur(&small, deviation);
        (
          imageops::resize(&blurred, width, height, FilterType::CatmullRom),
          true,
        )
      })
    }
    CameraBackground::Image => effects
      .picture
      .as_deref()
      .map(|picture| (filled(picture, (width, height)), false)),
    CameraBackground::Kept | CameraBackground::Transparent => None,
  };
  // The frame as recorded, which soft edge pixels take the person's colour
  // from, as the GPU's effects do.
  let recorded = (background != CameraBackground::Kept).then(|| frame.rgba.clone());
  // Sharpened before the rest, as `camera_effects.wgsl` does.
  let sharp = effects
    .enhance
    .then(|| super::sharpen::sharpened(&frame.rgba, (width, height)));
  for (index, pixel) in frame.rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
    let (x, y) = (index as u32 % width, index as u32 / width);
    let mut person = sharp
      .as_ref()
      .map_or([pixel[0], pixel[1], pixel[2]], |sharp| sharp[index]);
    if let (Some(recorded), Some(matte)) = (recorded.as_deref(), matte) {
      let alpha = f32::from(sample(matte, (x, y), (width, height))) / 255.0;
      if alpha > 0.0 && alpha < 0.97 {
        if let Some(inner) = inner_colour(recorded, matte, (x, y), (width, height)) {
          let share = ((0.97 - alpha) / 0.5).clamp(0.0, 1.0);
          for (channel, value) in person.iter_mut().enumerate() {
            *value = (f32::from(*value) * (1.0 - share) + inner[channel] * share).round() as u8;
          }
        }
      }
    }
    if let Some(grade) = effects.grade {
      person = grade.apply(person);
    }
    let alpha = match (background, matte) {
      (CameraBackground::Kept, _) | (_, None) => 255,
      (_, Some(matte)) => sample(matte, (x, y), (width, height)),
    };
    let mixed = match (&behind, background) {
      (Some((behind, graded)), _) => {
        let at = behind.get_pixel(x, y).0;
        let mut room = [at[0], at[1], at[2]];
        if let (true, Some(grade)) = (*graded, effects.grade) {
          room = grade.apply(room);
        }
        let mix = |front: u8, back: u8| {
          ((u32::from(front) * u32::from(alpha) + u32::from(back) * (255 - u32::from(alpha))) / 255)
            as u8
        };
        [
          mix(person[0], room[0]),
          mix(person[1], room[1]),
          mix(person[2], room[2]),
          255,
        ]
      }
      (None, CameraBackground::Transparent) => [person[0], person[1], person[2], alpha],
      (None, _) => [person[0], person[1], person[2], pixel[3]],
    };
    pixel.copy_from_slice(&mixed);
  }
}

/// The colour of the person around frame pixel `(x, y)`, from the pixels the
/// matte holds solid within about ten pixels of a 1080-line frame, as
/// `camera_effects.wgsl`'s `inner_colour` takes it.
fn inner_colour(
  recorded: &[u8],
  matte: &LumaFrame,
  (x, y): (u32, u32),
  (width, height): (u32, u32),
) -> Option<[f32; 3]> {
  let reach = height as f32 / 1080.0;
  let (mut sum, mut weights) = ([0.0_f32; 3], 0.0_f32);
  for ring in 1..=3 {
    let radius = ring as f32 * 3.5;
    let fall = (-radius * radius / 50.0).exp();
    for turn in 0..8 {
      let angle = (turn as f32 + 0.5 * (ring % 2) as f32) * std::f32::consts::FRAC_PI_4;
      let at = |base: u32, offset: f32, limit: u32| {
        (base as f32 + offset * radius * reach)
          .round()
          .clamp(0.0, (limit - 1) as f32) as u32
      };
      let (sx, sy) = (at(x, angle.cos(), width), at(y, angle.sin(), height));
      if sample(matte, (sx, sy), (width, height)) < 247 {
        continue;
      }
      let index = (sy * width + sx) as usize * 4;
      for (channel, total) in sum.iter_mut().enumerate() {
        *total += f32::from(recorded[index + channel]) * fall;
      }
      weights += fall;
    }
  }
  (weights > 0.0).then(|| sum.map(|total| total / weights))
}

/// The matte at frame pixel `at`, the matte stretched over the frame.
fn sample(matte: &LumaFrame, (x, y): (u32, u32), (width, height): (u32, u32)) -> u8 {
  let mx = (u64::from(x) * u64::from(matte.width) / u64::from(width)) as u32;
  let my = (u64::from(y) * u64::from(matte.height) / u64::from(height)) as u32;
  let index = (my.min(matte.height.saturating_sub(1)) * matte.width
    + mx.min(matte.width.saturating_sub(1))) as usize;
  matte.pixels.get(index).copied().unwrap_or(255)
}

/// `picture` filling `size`, cut at its middle where its shape differs.
fn filled(picture: &RgbaImage, (width, height): (u32, u32)) -> RgbaImage {
  let scale = (f64::from(width) / f64::from(picture.width().max(1)))
    .max(f64::from(height) / f64::from(picture.height().max(1)));
  let scaled = (
    ((f64::from(picture.width()) * scale).ceil() as u32).max(width),
    ((f64::from(picture.height()) * scale).ceil() as u32).max(height),
  );
  let resized = imageops::resize(picture, scaled.0, scaled.1, FilterType::Triangle);
  imageops::crop_imm(
    &resized,
    (scaled.0 - width) / 2,
    (scaled.1 - height) / 2,
    width,
    height,
  )
  .to_image()
}
