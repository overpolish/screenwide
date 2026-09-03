// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Which redraw classes a dataset replacement triggered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DataChange {
  pub geometry: bool,
  pub labels: bool,
}

/// Byte equality, the comparison macOS used (`isEqualToData:`). `PartialEq`
/// would report every unanchored label as changed, because an absent anchor is
/// `NaN` and `NaN != NaN`.
pub(super) fn same<T>(left: &[T], right: &[T]) -> bool {
  bytes(left) == bytes(right)
}

pub(super) fn bytes<T>(items: &[T]) -> &[u8] {
  // Every packet is `#[repr(C)]` with explicit padding fields, so there are no
  // uninitialised holes to read.
  unsafe { std::slice::from_raw_parts(items.as_ptr().cast::<u8>(), std::mem::size_of_val(items)) }
}

/// The "labelled projection": the flags a label's text and placement depend on,
/// with everything that only moves geometry masked away (`+ruler.m:252-430`).
pub(super) fn labelled_measurements(items: &[MeasurementPacket]) -> Vec<MeasurementPacket> {
  items
    .iter()
    .map(|item| {
      let mut value = *item;
      value.flags &= 11;
      value.padding[0] = 0;
      value
    })
    .collect()
}

pub(super) fn labelled_radii(items: &[RadiusPacket]) -> Vec<RadiusPacket> {
  items
    .iter()
    .map(|item| {
      let mut value = *item;
      value.flags &= 11;
      value.padding[0] = 0;
      value
    })
    .collect()
}

pub(super) fn labelled_guide_gaps(items: &[GuideGapPacket]) -> Vec<GuideGapPacket> {
  items
    .iter()
    .map(|item| {
      let mut value = *item;
      value.flags &= 2;
      value.padding[0] = 0;
      value
    })
    .collect()
}

/// Live probes and anonymous non-draft probes carry no label at all, so they
/// are dropped rather than masked.
pub(super) fn labelled_probes(items: &[ProbePacket]) -> Vec<ProbePacket> {
  items
    .iter()
    .filter_map(|item| {
      let draft = item.flags & 1 != 0;
      let live = item.flags & 4 != 0;
      if live || (item.id == 0 && !draft) {
        return None;
      }
      let mut value = *item;
      value.flags = u8::from(draft) | (item.flags & 8);
      value.padding[0] = 0;
      Some(value)
    })
    .collect()
}
