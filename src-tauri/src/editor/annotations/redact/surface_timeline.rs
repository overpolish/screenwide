// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The surface an erased box on a recording takes over its clip, worked out
//! ahead of time.
//!
//! The ring round the box is read every [`SAMPLE_MS`] across the clip, and
//! the box only changes colour where a new surface holds for [`HOLD_MS`]: a
//! background that turns from blue to white is followed, and something busy
//! passing beside the box, whose vote never settles, is not. Because the
//! whole clip is read first, the change lands where the new surface
//! appeared rather than once it had held, and fades in over [`FADE_MS`].
//! The ring lies outside the box, so following it reveals nothing the box
//! covers.
//!
//! A timeline is a list of entries, `[ms into the clip, packed colour]`, in
//! the side buffer's own form, so the preview and the export read it
//! through the one [`surface_at`].

use super::palette::packed;

/// How far apart the ring is read, in source milliseconds.
pub(crate) const SAMPLE_MS: u64 = 100;
/// How long a new surface must hold before the box follows it.
const HOLD_MS: u64 = 400;
/// How long the box takes to fade from one surface to the next.
const FADE_MS: f32 = 150.0;
/// How far apart, in any one channel, two votes must be to be two surfaces:
/// wide enough that a compressed recording's noise is one surface.
const APART: u8 = 12;

fn close(a: [u8; 3], b: [u8; 3]) -> bool {
  a.iter().zip(b).all(|(a, b)| a.abs_diff(b) <= APART)
}

/// The timeline from `samples`, each the ring's vote `ms` into the clip in
/// order, `None` where nothing opaque surrounds the box. Empty where the ring
/// never had a surface.
pub(crate) fn timeline(samples: &[(u64, Option<[u8; 3]>)]) -> Vec<[f32; 2]> {
  let Some(mut current) = samples.iter().find_map(|(_, colour)| *colour) else {
    return Vec::new();
  };
  let mut entries = vec![[0.0, packed(Some(&current))]];
  let mut index = 0;
  while index < samples.len() {
    let (at, colour) = samples[index];
    let Some(colour) = colour.filter(|colour| !close(*colour, current)) else {
      index += 1;
      continue;
    };
    let window = samples[index..]
      .iter()
      .take_while(|(time, _)| *time <= at + HOLD_MS)
      .count();
    let reaches_end = index + window == samples.len();
    let lasts = samples[index + window - 1].0 >= at + HOLD_MS || reaches_end;
    let holds = samples[index..index + window]
      .iter()
      .all(|(_, sampled)| sampled.is_some_and(|sampled| close(sampled, colour)));
    if lasts && holds && window > 1 {
      current = colour;
      entries.push([at as f32, packed(Some(&current))]);
      index += window;
    } else {
      index += 1;
    }
  }
  entries
}

/// The surface `elapsed_ms` into the clip, as RGB from 0 to 1, fading from
/// the surface before a change over the change's first [`FADE_MS`]. `None`
/// for an empty timeline.
pub(crate) fn surface_at(entries: &[[f32; 2]], elapsed_ms: f32) -> Option<[f32; 3]> {
  let unpack = |packed: f32| {
    let rgb = packed.max(0.0) as u32;
    [16, 8, 0].map(|shift| ((rgb >> shift) & 0xff) as f32 / 255.0)
  };
  let at = entries
    .iter()
    .rposition(|entry| entry[0] <= elapsed_ms)
    .or((!entries.is_empty()).then_some(0))?;
  let colour = unpack(entries[at][1]);
  let into = elapsed_ms - entries[at][0];
  if at == 0 || !(into < FADE_MS) {
    return Some(colour);
  }
  let before = unpack(entries[at - 1][1]);
  let share = (into / FADE_MS).clamp(0.0, 1.0);
  Some([0, 1, 2].map(|channel| before[channel] + (colour[channel] - before[channel]) * share))
}

/// [`surface_at`] for the video export's per-frame pass: `entries` is a
/// record's timeline in the side buffer, and `out` three floats.
///
/// # Safety
/// `entries` must point at `count` entries, and `out` at three floats.
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_redaction_surface_at(
  entries: *const [f32; 2],
  count: u32,
  elapsed_ms: f32,
  out: *mut f32,
) -> u32 {
  if entries.is_null() || out.is_null() || count == 0 {
    return 0;
  }
  let entries = std::slice::from_raw_parts(entries, count as usize);
  match surface_at(entries, elapsed_ms) {
    Some(colour) => {
      std::ptr::copy_nonoverlapping(colour.as_ptr(), out, 3);
      1
    }
    None => 0,
  }
}

#[cfg(test)]
#[path = "surface_timeline_tests.rs"]
mod tests;
