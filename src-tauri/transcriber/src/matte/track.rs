// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Objects clicked in one frame of the camera, followed through the
//! recording: SAM 2.1 turns each click into the object's mask in that frame,
//! and Cutie follows the objects forwards to the end, then backwards to the
//! start, on the GPU where it can be. The masks after the click are written
//! and told of first, so what follows it is shown soonest.
//!
//! Most objects worth picking sit still, so Cutie follows one frame in a
//! few (`follower`). Where an object's mask changes between two followed
//! frames, the frames between are followed too, from the earlier one, so a
//! moving object leaves no trail; elsewhere they take a blend of the two.
//!
//! Each way stops once every object has been gone from the frame for a
//! while ([`follower::GONE_MS`]): one that comes back later is clicked again.
//!
//! Each object's masks are written as two small grey videos, one either side
//! of the click, each with a frame for every camera frame it covers.

mod cutie;
mod follower;
mod sam;
mod slots;

use std::path::{Path, PathBuf};

use screenwide_transcriber::MatteTimings;

use self::cutie::{from_logits, Cutie, Object, HEIGHT, WIDTH};
pub(in crate::matte) use self::cutie::{MASK_HEIGHT, MASK_WIDTH};
use self::follower::{quarter, Follower};
use self::sam::Sam;
use self::slots::Slots;
use super::decode::Decoder;

pub(super) const WHAT: &str = "Object tracking";
/// How many frames are decoded at once going backwards, which decoding can
/// only do forwards.
const BACKWARDS_RUN: usize = 48;
/// How far before a frame decoding starts, so a seek that lands a little
/// late still decodes it.
const LEAD_MS: f64 = 200.0;

/// An object clicked: where, 0 to 1 across and down the frame, at one point
/// or several along a painted line, and where its masks go: those from the
/// click to the end, and from the start to it.
pub struct Clicked<'a> {
  pub points: &'a [[f32; 2]],
  pub after: &'a Path,
  pub before: &'a Path,
}

/// What to follow.
pub struct Follow<'a> {
  /// The model's files in the catalogue's order: SAM's image encoder and
  /// mask decoder, then Cutie's encode, read and memorise graphs.
  pub model: &'a [PathBuf],
  pub ffmpeg: &'a Path,
  pub camera: &'a Path,
  pub timings: &'a Path,
  pub frame: u64,
  pub objects: &'a [Clicked<'a>],
}

/// Follows `job`'s objects, telling `progress` how far it has got and
/// `halfway` once the masks after the click are written.
pub fn follow_objects(
  job: &Follow<'_>,
  progress: &mut dyn FnMut(f32),
  halfway: &mut dyn FnMut(),
) -> Result<(), String> {
  let [vision, decoder, encode, read, memorise] = job.model else {
    return Err(format!("{WHAT} needs the five files of its model"));
  };
  let timings: MatteTimings = std::fs::read(job.timings)
    .map_err(|error| error.to_string())
    .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|error| error.to_string()))
    .map_err(|error| format!("Could not read when the camera's frames show: {error}"))?;
  let frames = &timings.frames_ms;
  let seed = job.frame as usize;
  if seed >= frames.len() || job.objects.is_empty() {
    return Err("The frame clicked is not in the camera recording".to_owned());
  }
  if job.objects.iter().any(|object| object.points.is_empty()) {
    return Err("An object was picked at no point".to_owned());
  }
  // SAM reads one frame, quicker on the CPU than loading it onto the GPU.
  let clicked = {
    let mut sam = Sam::load(vision, decoder)?;
    let mut rgb = vec![0_u8; sam::SIDE * sam::SIDE * 3];
    let side = sam::SIDE as u32;
    let mut decoding = start(job, frames[seed], (side, side))?;
    if !read_until(&mut decoding, frames[seed], &mut rgb)? {
      return Err("The frame clicked could not be read".to_owned());
    }
    job
      .objects
      .iter()
      .map(|object| Ok(from_logits(&sam.mask(&rgb, object.points)?, sam::MASK_SIDE)))
      .collect::<Result<Vec<_>, String>>()?
  };
  let mut cutie = Cutie::load([encode, read, memorise], &cache(vision))?;
  let mut done = 0_usize;
  let total = frames.len() + 1;
  let mut tell = |more: usize| {
    done += more;
    progress((done as f32 / total as f32).min(1.0));
  };
  let mut rgb = vec![0_u8; WIDTH * HEIGHT * 3];
  let mut decoding = start(job, frames[seed], (WIDTH as u32, HEIGHT as u32))?;
  if !read_until(&mut decoding, frames[seed], &mut rgb)? {
    return Err("The frame clicked could not be read".to_owned());
  }
  let seen = cutie.see(&rgb)?;
  let objects = clicked
    .into_iter()
    .map(|mask| cutie.start(&seen, mask))
    .collect::<Result<Vec<_>, String>>()?;
  let quarters = objects
    .iter()
    .map(|object| quarter(object.last()))
    .collect::<Vec<_>>();
  let after = job
    .objects
    .iter()
    .map(|object| object.after)
    .collect::<Vec<_>>();
  let before = job
    .objects
    .iter()
    .map(|object| object.before)
    .collect::<Vec<_>>();
  // Forwards from the clicked frame to the end.
  let mut slots = Slots::create(&after, seed)?;
  seeded(&mut slots, &quarters, seed)?;
  tell(1);
  let mut follower = Follower::new(objects.clone(), quarters.clone());
  while let Some(ms) = decoding.next(&mut rgb)? {
    let index = frames.partition_point(|&frame_ms| frame_ms < ms - 2.0);
    if index <= seed || index >= frames.len() {
      continue;
    }
    if follower.gone(frames, ms) {
      // The rest of the way is left empty, and counted as done.
      tell(frames.len() - index);
      break;
    }
    follower.push(&mut cutie, &mut slots, index, rgb.clone(), &mut tell)?;
  }
  drop(decoding);
  follower.finish(&mut cutie, &mut slots, &mut tell)?;
  slots.finish(job.ffmpeg, frames.len() - seed)?;
  halfway();
  // Backwards from the clicked frame to the start.
  let mut slots = Slots::create(&before, 0)?;
  seeded(&mut slots, &quarters, seed)?;
  backwards(
    job,
    frames,
    seed,
    (&mut cutie, &mut slots),
    objects,
    quarters,
    &mut tell,
  )?;
  slots.finish(job.ffmpeg, seed + 1)
}

/// The objects' masks in the clicked frame into `slots`.
fn seeded(slots: &mut Slots, quarters: &[Vec<f32>], seed: usize) -> Result<(), String> {
  for (index, mask) in quarters.iter().enumerate() {
    slots.write(index, seed, mask)?;
  }
  Ok(())
}

/// Follows `objects` back from the clicked frame `seed` to the start, a run
/// of frames at a time, since decoding only goes forwards.
fn backwards(
  job: &Follow<'_>,
  frames: &[f64],
  seed: usize,
  (cutie, slots): (&mut Cutie, &mut Slots),
  objects: Vec<Object>,
  quarters: Vec<Vec<f32>>,
  tell: &mut dyn FnMut(usize),
) -> Result<(), String> {
  let mut follower = Follower::new(objects, quarters);
  let mut rgb = vec![0_u8; WIDTH * HEIGHT * 3];
  let mut end = seed;
  while end > 0 && !follower.gone(frames, frames[end]) {
    let first = end.saturating_sub(BACKWARDS_RUN);
    let mut decoding = start(job, frames[first], (WIDTH as u32, HEIGHT as u32))?;
    let mut run = Vec::with_capacity(end - first);
    for &wanted_ms in &frames[first..end] {
      if !read_until(&mut decoding, wanted_ms, &mut rgb)? {
        return Err("The camera recording ended early".to_owned());
      }
      run.push(rgb.clone());
    }
    drop(decoding);
    for (offset, rgb) in run.into_iter().enumerate().rev() {
      follower.push(cutie, slots, first + offset, rgb, tell)?;
    }
    end = first;
  }
  // Where every object was gone, the rest of the way is left empty.
  tell(end);
  follower.finish(cutie, slots, tell)
}

/// Where CoreML keeps the models it compiles: beside the model, under the
/// ONNX Runtime build that compiled them, since one compiled by another
/// build fails to run rather than being compiled again.
fn cache(model: &Path) -> PathBuf {
  use std::hash::{Hash, Hasher};
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  ort::info().hash(&mut hasher);
  model
    .parent()
    .unwrap_or_else(|| Path::new("."))
    .join(format!("coreml-cache-{:016x}", hasher.finish()))
}

fn start(job: &Follow<'_>, from_ms: f64, size: (u32, u32)) -> Result<Decoder, String> {
  let from_ms = (from_ms - LEAD_MS).max(0.0) as u64;
  Decoder::start(job.ffmpeg, job.camera, from_ms, size)
}

/// Reads frames until the one showing from `wanted_ms`, into `rgb`. False
/// if the camera ends first.
fn read_until(decoder: &mut Decoder, wanted_ms: f64, rgb: &mut [u8]) -> Result<bool, String> {
  // A frame's time read back can differ from the list's by rounding.
  loop {
    let Some(ms) = decoder.next(rgb)? else {
      return Ok(false);
    };
    if ms >= wanted_ms - 2.0 {
      return Ok(true);
    }
  }
}
