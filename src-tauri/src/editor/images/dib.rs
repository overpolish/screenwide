// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A Windows device-independent bitmap, as drags and the clipboard carry
//! it, made into the BMP file the `image` crate reads: the same bytes behind
//! the file header a DIB leaves out, which says where its pixels start.

/// Bytes of a BMP file's own header.
const FILE_HEADER: usize = 14;
/// `BI_BITFIELDS` and `BI_ALPHABITFIELDS`: behind a plain info header, the
/// channel masks follow it, three or four of them.
const BITFIELDS: u32 = 3;
const ALPHA_BITFIELDS: u32 = 6;
/// The size of the plain `BITMAPINFOHEADER`, and of the old core header.
const INFO_HEADER: usize = 40;
const CORE_HEADER: usize = 12;

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
  Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
  Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// `dib` as a BMP file, or `None` for bytes no bitmap header starts.
pub(crate) fn bmp_from_dib(dib: &[u8]) -> Option<Vec<u8>> {
  let header = u32_at(dib, 0)? as usize;
  if header < CORE_HEADER || dib.len() < header {
    return None;
  }
  let (bits, compression, colours, entry) = if header == CORE_HEADER {
    (u16_at(dib, 10)?, 0, 0, 3)
  } else {
    (
      u16_at(dib, 14)?,
      u32_at(dib, 16)?,
      u32_at(dib, 32)? as usize,
      4,
    )
  };
  let masks = match compression {
    BITFIELDS if header == INFO_HEADER => 12,
    ALPHA_BITFIELDS if header == INFO_HEADER => 16,
    _ => 0,
  };
  // A palette is a bitmap of few colours' own; a deeper one may carry the
  // colours it uses most, which the count says.
  let palette = match (bits, colours) {
    (1..=8, 0) => (1_usize << bits) * entry,
    _ => colours * entry,
  };
  let offset = FILE_HEADER + header + masks + palette;
  let size = u32::try_from(FILE_HEADER + dib.len()).ok()?;
  let mut bmp = Vec::with_capacity(FILE_HEADER + dib.len());
  bmp.extend_from_slice(b"BM");
  bmp.extend_from_slice(&size.to_le_bytes());
  bmp.extend_from_slice(&[0; 4]);
  bmp.extend_from_slice(&u32::try_from(offset).ok()?.to_le_bytes());
  bmp.extend_from_slice(dib);
  Some(bmp)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_dib_reads_back_as_the_picture_it_holds() {
    // Two pixels, 24 bits, bottom row first: blue then red, each row padded
    // to four bytes.
    let mut dib = Vec::new();
    dib.extend_from_slice(&40_u32.to_le_bytes());
    dib.extend_from_slice(&2_i32.to_le_bytes());
    dib.extend_from_slice(&1_i32.to_le_bytes());
    dib.extend_from_slice(&1_u16.to_le_bytes());
    dib.extend_from_slice(&24_u16.to_le_bytes());
    dib.extend_from_slice(&[0; 24]);
    dib.extend_from_slice(&[255, 0, 0, 0, 0, 255, 0, 0]);
    let picture = image::load_from_memory(&bmp_from_dib(&dib).expect("a bitmap"))
      .expect("a picture")
      .into_rgba8();
    assert_eq!(picture.dimensions(), (2, 1));
    assert_eq!(picture.get_pixel(0, 0).0, [0, 0, 255, 255]);
    assert_eq!(picture.get_pixel(1, 0).0, [255, 0, 0, 255]);
    assert!(bmp_from_dib(&[1, 2]).is_none());
  }
}
