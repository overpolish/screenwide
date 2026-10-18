// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a file's presentation timeline starts. Media Foundation's MPEG-4
//! source ignores the edit list that moves a B-frame stream's first frame to
//! zero, which ffmpeg and AVFoundation both apply, so it reports the first
//! frame a couple of frames late. Every reader here subtracts this origin, so
//! a time means the same frame on every platform.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use windows::core::PCWSTR;
use windows::Win32::Media::MediaFoundation::*;

use super::decoder::{win, VIDEO_STREAM};

/// How many compressed samples are read for the earliest presentation time:
/// more than any B-frame reordering puts ahead of the first frame shown.
const PROBED_SAMPLES: usize = 16;

/// A time Media Foundation reported, in 100 ns units, as milliseconds from
/// the start of the file's timeline at `origin`.
pub(crate) fn milliseconds(timestamp: i64, origin: i64) -> u64 {
  u64::try_from((timestamp - origin).max(0) / 10_000).unwrap_or_default()
}

/// The presentation time of `path`'s first frame, in 100 ns units, as Media
/// Foundation reports it. Zero where the stream starts at zero or cannot be
/// read. Worked out once a path, from the compressed samples, so nothing is
/// decoded.
pub(crate) fn origin_100ns(path: &Path) -> i64 {
  static ORIGINS: OnceLock<Mutex<HashMap<PathBuf, i64>>> = OnceLock::new();
  let origins = ORIGINS.get_or_init(Mutex::default);
  if let Some(origin) = origins.lock().ok().and_then(|map| map.get(path).copied()) {
    return origin;
  }
  let origin = probe(path).unwrap_or(0);
  if let Ok(mut map) = origins.lock() {
    map.insert(path.to_path_buf(), origin);
  }
  origin
}

fn probe(path: &Path) -> Result<i64, String> {
  let path = path
    .to_str()
    .ok_or_else(|| "The recording path is not valid UTF-8".to_owned())?;
  let wide = path.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
  let reader = win(unsafe { MFCreateSourceReaderFromURL(PCWSTR(wide.as_ptr()), None) })?;
  win(unsafe { reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false) })?;
  win(unsafe { reader.SetStreamSelection(VIDEO_STREAM, true) })?;
  let mut earliest: Option<i64> = None;
  for _ in 0..PROBED_SAMPLES {
    let mut flags = 0_u32;
    let mut timestamp = 0_i64;
    let mut sample = None;
    win(unsafe {
      reader.ReadSample(
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
    if sample.is_some() {
      earliest = Some(earliest.map_or(timestamp, |earliest| earliest.min(timestamp)));
    }
  }
  Ok(earliest.unwrap_or(0).max(0))
}
