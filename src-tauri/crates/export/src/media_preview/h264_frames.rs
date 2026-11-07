// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// How many frames `stream` holds, an Annex B H.264 stream with one slice to
/// a frame, as x264 writes with frame threads. Parameter sets and other
/// headers are not frames.
pub(crate) fn frame_count(stream: &[u8]) -> u64 {
  let starts = nal_starts(stream);
  starts
    .iter()
    .enumerate()
    .filter(|&(index, &start)| {
      let end = starts.get(index + 1).map_or(stream.len(), |next| *next);
      let header = stream[start..end]
        .iter()
        .skip_while(|byte| **byte == 0)
        .nth(1)
        .copied()
        .unwrap_or_default();
      matches!(header & 0x1f, 1 | 5)
    })
    .count() as u64
}

/// Where each NAL unit starts in `stream`, at its start code.
fn nal_starts(stream: &[u8]) -> Vec<usize> {
  let mut starts = Vec::new();
  let mut index = 0;
  while index + 3 <= stream.len() {
    if stream[index] == 0 && stream[index + 1] == 0 && stream[index + 2] == 1 {
      // A four-byte start code's leading zero belongs to it.
      let start = if index > 0 && stream[index - 1] == 0 {
        index - 1
      } else {
        index
      };
      starts.push(start);
      index += 3;
    } else {
      index += 1;
    }
  }
  starts
}

#[cfg(test)]
mod tests {
  use super::*;

  fn nal(kind: u8, bytes: usize) -> Vec<u8> {
    let mut unit = vec![0, 0, 0, 1, kind];
    unit.resize(4 + bytes, 0xAA);
    unit
  }

  #[test]
  fn counts_slices_but_not_headers() {
    let stream = [nal(7, 10), nal(8, 5), nal(5, 1_000), nal(1, 20), nal(1, 30)].concat();
    assert_eq!(frame_count(&stream), 3);
  }
}
