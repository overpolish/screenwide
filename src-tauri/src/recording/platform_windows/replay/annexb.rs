// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The H.264 bits a clip needs to decode on its own: every clip starts on a
//! keyframe, and that keyframe has to carry the stream's parameter sets.
//! Media Foundation encoders write Annex B, but not all of them repeat the
//! sequence and picture parameter sets in front of every keyframe.

const NAL_SLICE: u8 = 1;
const NAL_IDR: u8 = 5;
const NAL_SPS: u8 = 7;
const NAL_PPS: u8 = 8;

/// Where the next start code at or after `from` begins, and where the unit
/// it opens begins.
fn next_start(data: &[u8], from: usize) -> Option<(usize, usize)> {
  let found = data
    .get(from..)?
    .windows(3)
    .position(|window| window == [0, 0, 1])?
    + from;
  // A four-byte start code belongs to the unit it opens.
  let start = if found > from && data[found - 1] == 0 {
    found - 1
  } else {
    found
  };
  Some((start, found + 3))
}

/// The NAL units of an Annex B access unit up to its first picture slice,
/// start codes included. Slices come last in an access unit, so the units
/// that matter here are all in front of the picture data, and the picture
/// data itself is never scanned.
fn header_units(data: &[u8]) -> impl Iterator<Item = (u8, &[u8])> {
  let mut next = next_start(data, 0);
  std::iter::from_fn(move || {
    let (start, body) = next?;
    let kind = data.get(body)? & 0x1f;
    if kind == NAL_SLICE || kind == NAL_IDR {
      next = None;
      return Some((kind, &data[start..]));
    }
    next = next_start(data, body);
    let end = next.map_or(data.len(), |(start, _)| start);
    Some((kind, &data[start..end]))
  })
}

/// Whether `data` holds an IDR picture.
pub(super) fn is_idr(data: &[u8]) -> bool {
  header_units(data).any(|(kind, _)| kind == NAL_IDR)
}

/// The SPS and PPS units in `data`, in order, if it has both.
pub(super) fn parameter_sets(data: &[u8]) -> Option<Vec<u8>> {
  let mut sets = Vec::new();
  let (mut sps, mut pps) = (false, false);
  for (kind, unit) in header_units(data) {
    match kind {
      NAL_SPS => sps = true,
      NAL_PPS => pps = true,
      _ => continue,
    }
    sets.extend_from_slice(unit);
  }
  (sps && pps).then_some(sets)
}

#[cfg(test)]
mod tests {
  use super::*;

  const SPS: [u8; 7] = [0, 0, 0, 1, 0x67, 0x64, 0x00];
  const PPS: [u8; 6] = [0, 0, 0, 1, 0x68, 0xee];
  const IDR: [u8; 6] = [0, 0, 1, 0x65, 0x88, 0x84];
  const SLICE: [u8; 6] = [0, 0, 0, 1, 0x41, 0x9a];

  fn joined(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
  }

  #[test]
  fn finds_the_parameter_sets_of_a_keyframe_that_carries_them() {
    let keyframe = joined(&[&SPS, &PPS, &IDR]);

    assert!(is_idr(&keyframe));
    assert_eq!(parameter_sets(&keyframe), Some(joined(&[&SPS, &PPS])));
  }

  #[test]
  fn a_picture_without_both_sets_has_none_to_offer() {
    assert_eq!(parameter_sets(&joined(&[&SPS, &IDR])), None);
    assert_eq!(parameter_sets(&SLICE), None);
    assert!(!is_idr(&SLICE));
  }
}
