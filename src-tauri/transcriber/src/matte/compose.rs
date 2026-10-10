// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The matte with the objects picked in it: each kept object added to the
//! person and each removed object taken away. Where two cross, the one
//! clicked nearest the frame wins: a mask is surest where it was clicked,
//! and one followed far from there may have wandered onto another object,
//! so a removal lying mostly on an object kept from nearer is left out.
//! An object's masks have a frame for each camera frame from the one they
//! start at, at Cutie's mask size, stretched over the matte; the same object
//! may have two, one either side of where it was clicked.

mod edge;

use std::io::{BufReader, Read};
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};

use screenwide_transcriber::PickedMask as Picked;

use self::edge::Edge;
use super::decode::Decoder;
use super::encode::Encoder;
use super::track::{MASK_HEIGHT, MASK_WIDTH, WHAT};

/// The matte at `matte`, `frames` long at `size`, written with `picks` in
/// it to `output` at `frames_per_second`, the objects' edges guided by the
/// frames of `camera`. Tells `progress` how far it has got.
pub fn compose_matte(
  (ffmpeg, camera): (&Path, &Path),
  matte: &Path,
  (size, frames, frames_per_second): ((u32, u32), u64, u32),
  picks: &[Picked],
  output: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let (width, height) = (size.0 as usize, size.1 as usize);
  let mut base = Grey::start(ffmpeg, matte, size)?;
  // The camera has a frame for each of the matte's, in turn.
  let mut decoder = Decoder::start(ffmpeg, camera, 0, size)?;
  let mask_size = (MASK_WIDTH as u32, MASK_HEIGHT as u32);
  let mut masks = picks
    .iter()
    .map(|pick| Grey::start(ffmpeg, &pick.path, mask_size))
    .collect::<Result<Vec<_>, String>>()?;
  let mut order = (0..picks.len()).collect::<Vec<_>>();
  let mut encoder = Encoder::start(ffmpeg, size, frames_per_second, output)?;
  let mut alpha = vec![0_u8; width * height];
  let mut rgb = vec![0_u8; width * height * 3];
  let mut guide = vec![0.0_f32; width * height];
  // Each object's mask this frame, where it has one.
  let mut shown = vec![vec![0_u8; MASK_WIDTH * MASK_HEIGHT]; picks.len()];
  let mut present = vec![false; picks.len()];
  let mut edge = Edge::new(width, height);
  for frame in 0..frames {
    if !base.next(&mut alpha)? || decoder.next(&mut rgb)?.is_none() {
      break;
    }
    for (value, pixel) in guide.iter_mut().zip(rgb.as_chunks::<3>().0) {
      let [r, g, b] = pixel.map(f32::from);
      *value = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
    }
    for (index, pick) in picks.iter().enumerate() {
      // Masks leave the frames before they start and after they end alone.
      present[index] = frame >= pick.first && masks[index].next(&mut shown[index])?;
    }
    // The nearest click last, so it wins; as near, a removal after a keep.
    order.sort_by_key(|&index| {
      let pick = &picks[index];
      (std::cmp::Reverse(frame.abs_diff(pick.clicked)), !pick.keep)
    });
    for (place, &index) in order.iter().enumerate() {
      if !present[index] {
        continue;
      }
      // A removal lying on an object kept from a nearer click has wandered
      // onto it: the same object, which the nearer click says to keep.
      let claimed = !picks[index].keep
        && order[place + 1..].iter().any(|&other| {
          present[other] && picks[other].keep && covers(&shown[other], &shown[index])
        });
      if !claimed {
        edge.apply(&mut alpha, &shown[index], &guide, picks[index].keep);
      }
    }
    encoder.write(&alpha)?;
    if frame % 30 == 0 {
      progress((frame as f32 / frames.max(1) as f32).min(1.0));
    }
  }
  encoder.finish()
}

/// Whether the object in mask `over` covers most of the one in `under`:
/// over half of the cells `under` holds the object in.
fn covers(over: &[u8], under: &[u8]) -> bool {
  let (mut held, mut covered) = (0_usize, 0_usize);
  for (&over, &under) in over.iter().zip(under) {
    if under > 127 {
      held += 1;
      covered += usize::from(over > 127);
    }
  }
  held > 0 && covered * 2 > held
}

/// A grey video's frames in turn.
struct Grey {
  child: Child,
  frames: BufReader<ChildStdout>,
}

impl Grey {
  fn start(ffmpeg: &Path, path: &Path, (width, height): (u32, u32)) -> Result<Self, String> {
    let mut child = Command::new(ffmpeg)
      .args(["-hide_banner", "-nostdin", "-loglevel", "error", "-i"])
      .arg(path)
      .args(["-map", "0:v:0", "-vf"])
      .arg(format!("scale={width}:{height}"))
      .args([
        "-fps_mode",
        "passthrough",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "gray",
        "-",
      ])
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|error| format!("{WHAT} could not read a matte: {error}"))?;
    let frames = child
      .stdout
      .take()
      .map(BufReader::new)
      .ok_or_else(|| format!("{WHAT} could not read a matte"))?;
    Ok(Self { child, frames })
  }

  fn next(&mut self, frame: &mut [u8]) -> Result<bool, String> {
    let mut filled = 0;
    while filled < frame.len() {
      match self.frames.read(&mut frame[filled..]) {
        Ok(0) => return Ok(false),
        Ok(read) => filled += read,
        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
        Err(error) => return Err(format!("{WHAT} could not read a matte: {error}")),
      }
    }
    Ok(true)
  }
}

impl Drop for Grey {
  fn drop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}
