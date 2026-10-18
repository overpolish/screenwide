// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Media Foundation source-reader wrapper shared by Windows playback, stills
//! and timeline thumbnails.

use std::path::Path;

use windows::core::{Interface, GUID, PCWSTR};
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

use screenwide_imaging::CapturedImage;

mod luma;
pub use luma::LumaReader;

const HUNDRED_NS_PER_MS: i64 = 10_000;
const SEEK_PREROLL_MS: u64 = 1_500;
pub(super) const VIDEO_STREAM: u32 = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;

pub(super) fn win<T>(result: windows::core::Result<T>) -> Result<T, String> {
  result.map_err(|error| error.to_string())
}

pub(super) struct MediaFoundation {
  uninitialize_com: bool,
}

impl MediaFoundation {
  pub(super) fn start() -> Result<Self, String> {
    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    // Tauri's blocking pool can hand export a thread that another Windows
    // component already initialized as STA. COM is available in that case;
    // only changing its apartment is forbidden. Media Foundation's synchronous
    // source reader works in either apartment, so retain the existing one and
    // do not balance an initialization this runtime did not perform.
    let uninitialize_com = if initialized == RPC_E_CHANGED_MODE {
      false
    } else {
      initialized.ok().map_err(|error| error.to_string())?;
      true
    };
    if let Err(error) = unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) } {
      if uninitialize_com {
        unsafe { CoUninitialize() };
      }
      return Err(error.to_string());
    }
    Ok(Self { uninitialize_com })
  }
}

impl Drop for MediaFoundation {
  fn drop(&mut self) {
    let _ = unsafe { MFShutdown() };
    if self.uninitialize_com {
      unsafe { CoUninitialize() };
    }
  }
}

pub struct NativeVideoReader {
  height: u32,
  /// The last sample read, with its time in milliseconds.
  last_sample: Option<(IMFSample, u64)>,
  /// Where the file's timeline starts as Media Foundation counts it, in
  /// 100 ns units, subtracted from every time it reports.
  origin: i64,
  reader: IMFSourceReader,
  width: u32,
  // Fields drop in declaration order. Keep the process-wide MF reference
  // alive until after the source reader and every sample it owns are gone.
  _runtime: MediaFoundation,
}

impl NativeVideoReader {
  pub fn open(
    path: &Path,
    requested_width: u32,
    requested_height: u32,
    start_ms: u64,
  ) -> Result<Self, String> {
    let runtime = MediaFoundation::start()?;
    let attributes = attributes(2)?;
    win(unsafe { attributes.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1) })?;
    win(unsafe { attributes.SetUINT32(&MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, 1) })?;
    let path = path
      .to_str()
      .ok_or_else(|| "The recording path is not valid UTF-8".to_owned())?;
    let wide = path.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let reader = win(unsafe { MFCreateSourceReaderFromURL(PCWSTR(wide.as_ptr()), &attributes) })?;
    win(unsafe { reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false) })?;
    win(unsafe { reader.SetStreamSelection(VIDEO_STREAM, true) })?;
    let native = win(unsafe { reader.GetNativeMediaType(VIDEO_STREAM, 0) })?;
    let native_size = win(unsafe { native.GetUINT64(&MF_MT_FRAME_SIZE) })?;
    let native_width = (native_size >> 32) as u32;
    let native_height = native_size as u32;

    let output = unsafe { MFCreateMediaType() }.map_err(|error| error.to_string())?;
    win(unsafe { output.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) })?;
    win(unsafe { output.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32) })?;
    let requested_width = if requested_width == 0 {
      native_width
    } else {
      requested_width.min(native_width)
    }
    .max(2)
      & !1;
    let requested_height = if requested_height == 0 {
      native_height
    } else {
      requested_height.min(native_height)
    }
    .max(2)
      & !1;
    win(unsafe {
      output.SetUINT64(
        &MF_MT_FRAME_SIZE,
        (u64::from(requested_width) << 32) | u64::from(requested_height),
      )
    })?;
    win(unsafe { reader.SetCurrentMediaType(VIDEO_STREAM, None, &output) })?;
    let negotiated = win(unsafe { reader.GetCurrentMediaType(VIDEO_STREAM) })?;
    let packed = win(unsafe { negotiated.GetUINT64(&MF_MT_FRAME_SIZE) })?;
    let width = (packed >> 32) as u32;
    let height = packed as u32;
    if width == 0 || height == 0 {
      return Err("Media Foundation negotiated an empty preview frame".to_owned());
    }
    let mut value = Self {
      height,
      last_sample: None,
      origin: super::origin::origin_100ns(Path::new(path)),
      reader,
      width,
      _runtime: runtime,
    };
    value.seek(start_ms)?;
    Ok(value)
  }

  pub fn seek(&mut self, position_ms: u64) -> Result<(), String> {
    self.seek_to(position_ms.saturating_sub(SEEK_PREROLL_MS))
  }

  /// Seeks with no rewind, for a caller that takes any frame near
  /// `position_ms`. Media Foundation's MPEG-4 source lands on the keyframe
  /// at or before the position, the least decoding to a frame close by; a
  /// fragmented MP4 may land on the keyframe after it instead.
  pub fn seek_near(&mut self, position_ms: u64) -> Result<(), String> {
    self.seek_to(position_ms)
  }

  fn seek_to(&mut self, seek_ms: u64) -> Result<(), String> {
    win(unsafe { self.reader.Flush(VIDEO_STREAM) })?;
    let position = PROPVARIANT::from(
      i64::try_from(seek_ms)
        .unwrap_or(i64::MAX / HUNDRED_NS_PER_MS)
        .saturating_mul(HUNDRED_NS_PER_MS)
        .saturating_add(self.origin),
    );
    win(unsafe { self.reader.SetCurrentPosition(&GUID::zeroed(), &position) })?;
    self.last_sample = None;
    Ok(())
  }

  pub fn frame_at(&mut self, target_ms: u64) -> Result<Option<CapturedImage>, String> {
    Ok(self.timed_frame_at(target_ms)?.map(|(frame, _)| frame))
  }

  /// The first frame at or after `target_ms`, or the last one before the
  /// stream ends, with the time in milliseconds it is from.
  pub fn timed_frame_at(&mut self, target_ms: u64) -> Result<Option<(CapturedImage, u64)>, String> {
    loop {
      let mut flags = 0_u32;
      let mut timestamp = 0_i64;
      let mut sample = None;
      win(unsafe {
        self.reader.ReadSample(
          VIDEO_STREAM,
          0,
          None,
          Some(&mut flags),
          Some(&mut timestamp),
          Some(&mut sample),
        )
      })?;
      if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
        return self
          .last_sample
          .as_ref()
          .map(|(sample, ms)| Ok((sample_image(sample, self.width, self.height)?, *ms)))
          .transpose();
      }
      if flags & MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED.0 as u32 != 0 {
        let negotiated = win(unsafe { self.reader.GetCurrentMediaType(VIDEO_STREAM) })?;
        let packed = win(unsafe { negotiated.GetUINT64(&MF_MT_FRAME_SIZE) })?;
        self.width = (packed >> 32) as u32;
        self.height = packed as u32;
        self.last_sample = None;
        if self.width == 0 || self.height == 0 {
          return Err("Media Foundation changed to an empty preview frame".to_owned());
        }
      }
      let Some(sample) = sample else {
        continue;
      };
      let timestamp_ms = super::origin::milliseconds(timestamp, self.origin);
      // Retain the native sample; copying a full RGBA frame just to cache
      // the last image doubles the allocation for every requested frame.
      self.last_sample = Some((sample.clone(), timestamp_ms));
      if timestamp_ms.saturating_add(2) >= target_ms {
        return Ok(Some((
          sample_image(&sample, self.width, self.height)?,
          timestamp_ms,
        )));
      }
    }
  }
}

fn attributes(capacity: u32) -> Result<IMFAttributes, String> {
  let mut value = None;
  unsafe { MFCreateAttributes(&mut value, capacity) }.map_err(|error| error.to_string())?;
  value.ok_or_else(|| "Media Foundation created no preview attributes".to_owned())
}

fn sample_image(sample: &IMFSample, width: u32, height: u32) -> Result<CapturedImage, String> {
  let buffer = win(unsafe { sample.ConvertToContiguousBuffer() })?;
  let surface = buffer
    .cast::<IMF2DBuffer>()
    .map_err(|error| error.to_string())?;
  let mut first_row = std::ptr::null_mut();
  let mut pitch = 0_i32;
  win(unsafe { surface.Lock2D(&mut first_row, &mut pitch) })?;
  if first_row.is_null() || pitch == 0 {
    let _ = unsafe { surface.Unlock2D() };
    return Err("Media Foundation returned an empty preview frame".to_owned());
  }
  let row_bytes = usize::try_from(width).unwrap_or_default().saturating_mul(4);
  let mut rgba = vec![0_u8; row_bytes.saturating_mul(height as usize)];
  for row in 0..height as usize {
    let source = unsafe {
      std::slice::from_raw_parts(first_row.offset(pitch as isize * row as isize), row_bytes)
    };
    let target = &mut rgba[row * row_bytes..(row + 1) * row_bytes];
    for (source, target) in source.chunks_exact(4).zip(target.chunks_exact_mut(4)) {
      target[0] = source[2];
      target[1] = source[1];
      target[2] = source[0];
      target[3] = source[3];
    }
  }
  let unlock = unsafe { surface.Unlock2D() };
  win(unlock)?;
  Ok(CapturedImage {
    height,
    rgba,
    width,
  })
}

pub fn encoded_jpeg(image: &CapturedImage, quality: u8) -> Result<Vec<u8>, String> {
  let rgba = image::RgbaImage::from_raw(image.width, image.height, image.rgba.clone())
    .ok_or_else(|| "Media Foundation returned invalid preview pixels".to_owned())?;
  let mut bytes = Vec::new();
  image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, quality)
    .encode_image(&rgba)
    .map_err(|error| error.to_string())?;
  Ok(bytes)
}

#[cfg(test)]
mod tests;
