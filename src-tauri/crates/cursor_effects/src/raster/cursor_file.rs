// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sizes a Windows cursor file carries. A `.cur` holds one picture for
//! each size it was drawn at; drawn at the largest of those, the cursor is
//! its own pixels rather than a smaller picture stretched by Windows, which
//! the upscaling model can then draw larger still without blurring.

/// The largest picture's side in the cursor file `bytes`, in pixels; `None`
/// where it is not a static cursor or icon file, as an animated `.ani` is
/// not.
pub(super) fn largest_side(bytes: &[u8]) -> Option<u32> {
  let word = |at: usize| Some(u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]));
  // The file starts with zero, then 2 for a cursor or 1 for an icon, then
  // how many pictures follow, each described in sixteen bytes.
  if word(0)? != 0 || !matches!(word(2)?, 1 | 2) {
    return None;
  }
  let count = usize::from(word(4)?);
  (0..count)
    .filter_map(|index| {
      let entry = 6 + index * 16;
      // A side is one byte, and 0 means 256.
      let width = *bytes.get(entry)?;
      let height = *bytes.get(entry + 1)?;
      let side = |byte: u8| if byte == 0 { 256 } else { u32::from(byte) };
      Some(side(width).max(side(height)))
    })
    .max()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A cursor file's header and `sides` entries, the pictures themselves
  /// left out.
  fn cursor_file(sides: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0, 0, 2, 0, sides.len() as u8, 0];
    for &side in sides {
      let mut entry = [0_u8; 16];
      entry[0] = side;
      entry[1] = side;
      bytes.extend_from_slice(&entry);
    }
    bytes
  }

  #[test]
  fn finds_the_largest_picture_a_cursor_carries() {
    assert_eq!(largest_side(&cursor_file(&[32, 48, 128, 64])), Some(128));
  }

  #[test]
  fn a_side_of_zero_is_256() {
    assert_eq!(largest_side(&cursor_file(&[32, 0])), Some(256));
  }

  #[test]
  fn an_animated_cursor_is_not_read() {
    assert_eq!(largest_side(b"RIFF\x10\x00\x00\x00ACON"), None);
  }
}
