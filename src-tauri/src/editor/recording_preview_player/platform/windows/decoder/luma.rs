// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pin tracker's decoder: brightness alone, at the tracking size.
//!
//! Media Foundation's video processor scales each frame to the tracking size
//! on the GPU, so only the small frame is read back and reduced to its
//! brightness here.

use std::path::{Path, PathBuf};

use windows::core::{Interface, GUID};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;

use super::{win, NativeVideoReader, HUNDRED_NS_PER_MS, VIDEO_STREAM};
use crate::editor::annotations::pin::{FrameSource, LumaFrame};

pub(crate) struct LumaReader {
  path: PathBuf,
  duration_ms: u64,
  source: (u32, u32),
  size: (u32, u32),
  reader: Option<NativeVideoReader>,
}

impl LumaReader {
  /// A reader for `path` whose frames are at most `longest_side` pixels on
  /// their longer side, and never larger than the source.
  pub(crate) fn open(path: &Path, duration_ms: u64, longest_side: u32) -> Result<Self, String> {
    let native = NativeVideoReader::open(path, 0, 0, 0)?;
    let source = (native.width, native.height);
    drop(native);
    let longest = source.0.max(source.1).max(1);
    let factor = (f64::from(longest_side) / f64::from(longest)).min(1.0);
    let even = |value: u32| ((f64::from(value) * factor).round() as u32).max(2) & !1;
    Ok(Self {
      path: path.to_path_buf(),
      duration_ms,
      source,
      size: (even(source.0), even(source.1)),
      reader: None,
    })
  }
}

impl FrameSource for LumaReader {
  fn source_size(&self) -> (u32, u32) {
    self.source
  }

  fn read(
    &mut self,
    start_ms: u64,
    end_ms: u64,
    each: &mut dyn FnMut(LumaFrame) -> bool,
  ) -> Result<(), String> {
    let end_ms = end_ms.min(self.duration_ms);
    if start_ms >= end_ms {
      return Ok(());
    }
    let mut reader = match self.reader.take() {
      Some(reader) => reader,
      None => NativeVideoReader::open(&self.path, self.size.0, self.size.1, 0)?,
    };
    seek(&mut reader, start_ms)?;
    loop {
      let mut flags = 0_u32;
      let mut timestamp = 0_i64;
      let mut sample = None;
      win(unsafe {
        reader.reader.ReadSample(
          VIDEO_STREAM,
          0,
          None,
          Some(&mut flags),
          Some(&mut timestamp),
          Some(&mut sample),
        )
      })?;
      if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
        break;
      }
      if flags & MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED.0 as u32 != 0 {
        let negotiated = win(unsafe { reader.reader.GetCurrentMediaType(VIDEO_STREAM) })?;
        let packed = win(unsafe { negotiated.GetUINT64(&MF_MT_FRAME_SIZE) })?;
        reader.width = (packed >> 32) as u32;
        reader.height = packed as u32;
      }
      let Some(sample) = sample else {
        continue;
      };
      let ms = u64::try_from(timestamp.max(0) / HUNDRED_NS_PER_MS).unwrap_or_default();
      // A seek lands on the keyframe before the start.
      if ms < start_ms {
        continue;
      }
      if ms >= end_ms {
        break;
      }
      if !each(luma_frame(&sample, reader.width, reader.height, ms)?) {
        break;
      }
    }
    self.reader = Some(reader);
    Ok(())
  }
}

/// Moves `reader` to `start_ms`. Media Foundation's MPEG-4 source lands on
/// the keyframe before the position, so no preroll is added: the frames
/// before the start are decoded and skipped either way.
fn seek(reader: &mut NativeVideoReader, start_ms: u64) -> Result<(), String> {
  win(unsafe { reader.reader.Flush(VIDEO_STREAM) })?;
  let position = PROPVARIANT::from(
    i64::try_from(start_ms)
      .unwrap_or(i64::MAX / HUNDRED_NS_PER_MS)
      .saturating_mul(HUNDRED_NS_PER_MS),
  );
  win(unsafe { reader.reader.SetCurrentPosition(&GUID::zeroed(), &position) })
}

/// The brightness of an ARGB32 sample, with the BT.709 weights.
fn luma_frame(sample: &IMFSample, width: u32, height: u32, ms: u64) -> Result<LumaFrame, String> {
  let buffer = win(unsafe { sample.ConvertToContiguousBuffer() })?;
  let surface = buffer
    .cast::<IMF2DBuffer>()
    .map_err(|error| error.to_string())?;
  let mut first_row = std::ptr::null_mut();
  let mut pitch = 0_i32;
  win(unsafe { surface.Lock2D(&mut first_row, &mut pitch) })?;
  if first_row.is_null() || pitch == 0 {
    let _ = unsafe { surface.Unlock2D() };
    return Err("Media Foundation returned an empty video frame".to_owned());
  }
  let (columns, rows) = (width as usize, height as usize);
  let mut pixels = Vec::with_capacity(columns * rows);
  for row in 0..rows {
    let source = unsafe {
      std::slice::from_raw_parts(first_row.offset(pitch as isize * row as isize), columns * 4)
    };
    pixels.extend(source.as_chunks::<4>().0.iter().map(|bgra| {
      let luma = 18 * u32::from(bgra[0]) + 183 * u32::from(bgra[1]) + 55 * u32::from(bgra[2]);
      (luma >> 8) as u8
    }));
  }
  win(unsafe { surface.Unlock2D() })?;
  Ok(LumaFrame {
    ms,
    width,
    height,
    pixels,
  })
}
