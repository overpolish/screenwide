// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "video/luma.rs"]
mod luma;
#[path = "video/playback.rs"]
mod playback;
pub(crate) use luma::LumaReader;
pub(super) use playback::spawn;

use super::{
  composition::gpu_still_overlay,
  cursor::{gpu_cursor_preview, GpuCursorPreview},
  VideoFramePayload,
};
use crate::editor::recording_preview_player::{
  video::{presentation_elapsed_ms, source_position_ms, VideoFrame},
  PlayerSources,
};
use crate::screenshots::CapturedImage;
use cidre::{arc, av, cm, cv, ns};
use std::{
  path::Path,
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{SyncSender, TrySendError},
    Arc,
  },
};
unsafe extern "C" {
  fn screenwide_preview_reader_enable_random_access(output: *mut std::ffi::c_void);
  fn screenwide_preview_reader_reset_range(
    output: *mut std::ffi::c_void,
    start_milliseconds: i64,
    duration_milliseconds: i64,
  ) -> i32;
}
pub(super) struct NativeVideoReader {
  _reader: arc::R<av::AssetReader>,
  last_sample: Option<arc::R<cm::SampleBuf>>,
  output: arc::R<av::AssetReaderTrackOutput>,
  pending: Option<arc::R<cm::SampleBuf>>,
}
pub(super) fn open_asset(path: &Path) -> Result<arc::R<av::UrlAsset>, String> {
  let path_text = path
    .to_str()
    .ok_or_else(|| "The recording path is not valid UTF-8".to_owned())?;
  let url = ns::Url::with_fs_path_str(path_text, false);
  av::UrlAsset::with_url(&url, None)
    .ok_or_else(|| format!("AVFoundation could not open {}", path.display()))
}

fn output_settings(
  width: u32,
  height: u32,
  format: cv::PixelFormat,
) -> arc::R<ns::Dictionary<ns::String, ns::Id>> {
  let pixel_format = format.to_ns_number();
  let width = ns::Number::with_u32(width);
  let height = ns::Number::with_u32(height);
  ns::Dictionary::with_keys_values(
    &[
      cv::pixel_buffer_keys::pixel_format().as_ns(),
      cv::pixel_buffer_keys::width().as_ns(),
      cv::pixel_buffer_keys::height().as_ns(),
    ],
    &[
      pixel_format.as_id_ref(),
      width.as_id_ref(),
      height.as_id_ref(),
    ],
  )
}

impl NativeVideoReader {
  pub(super) fn open(
    asset: &av::UrlAsset,
    width: u32,
    height: u32,
    start_ms: u64,
    duration_ms: u64,
  ) -> Result<Self, String> {
    Self::open_as(
      asset,
      width,
      height,
      cv::PixelFormat::_32_BGRA,
      start_ms,
      duration_ms,
    )
  }

  /// A reader decoding to `format`, scaled by the decoder to `width` by
  /// `height`.
  pub(super) fn open_as(
    asset: &av::UrlAsset,
    width: u32,
    height: u32,
    format: cv::PixelFormat,
    start_ms: u64,
    duration_ms: u64,
  ) -> Result<Self, String> {
    let tracks =
      tauri::async_runtime::block_on(asset.load_tracks_with_media_type(av::MediaType::video()))
        .map_err(|error| error.to_string())?;
    let track = tracks
      .get(0)
      .map_err(|_| "The recording has no video track".to_owned())?;
    let settings = output_settings(width, height, format);
    let mut output = av::AssetReaderTrackOutput::with_track(&track, Some(&settings))
      .map_err(|error| error.to_string())?;
    output.set_always_copies_sample_data(false);
    unsafe {
      screenwide_preview_reader_enable_random_access(output.as_ptr().cast());
    }
    let mut reader = av::AssetReader::with_asset(asset).map_err(|error| error.to_string())?;
    reader
      .set_time_range(cm::TimeRange {
        start: cm::Time::new(start_ms as i64, 1_000),
        duration: cm::Time::new(duration_ms.saturating_sub(start_ms).max(1) as i64, 1_000),
      })
      .map_err(|error| error.to_string())?;
    reader
      .add_output(&output)
      .map_err(|error| error.to_string())?;
    if !reader.start_reading().map_err(|error| error.to_string())? {
      return Err(reader.error().map_or_else(
        || "AVFoundation could not start preview playback".to_owned(),
        |error| error.to_string(),
      ));
    }
    Ok(Self {
      _reader: reader,
      last_sample: None,
      output,
      pending: None,
    })
  }

  /// Repositions the existing AVFoundation decode pipeline. Random-access
  /// outputs keep their decoder and asset I/O state alive across scrubs, which
  /// avoids paying AVAssetReader construction cost for every backward jump.
  pub(super) fn reset(&mut self, start_ms: u64, duration_ms: u64) -> Result<(), String> {
    self.pending = None;
    self.last_sample = None;
    let reset = unsafe {
      screenwide_preview_reader_reset_range(
        self.output.as_ptr().cast(),
        start_ms as i64,
        duration_ms.saturating_sub(start_ms).max(1) as i64,
      )
    };
    if reset == 0 {
      Err("AVFoundation could not reposition the preview decoder".to_owned())
    } else {
      Ok(())
    }
  }

  pub(super) fn frame_at(&mut self, target_ms: u64) -> Result<Option<CapturedImage>, String> {
    self.sample_at(target_ms)?.map(Self::converted).transpose()
  }

  pub(super) fn pixel_frame_at(
    &mut self,
    target_ms: u64,
  ) -> Result<Option<super::scrubber::NativePixelFrame>, String> {
    self
      .sample_at(target_ms)?
      .map(|sample| super::scrubber::NativePixelFrame::from_sample(&sample))
      .transpose()
  }

  fn sample_at(&mut self, target_ms: u64) -> Result<Option<arc::R<cm::SampleBuf>>, String> {
    loop {
      if self.pending.is_none() {
        self.pending = self
          .output
          .next_sample_buf()
          .map_err(|error| error.to_string())?;
      }
      let Some(sample) = self.pending.as_ref() else {
        return Ok(self.last_sample.clone());
      };
      let pts_ms = (sample.pts().as_secs().max(0.0) * 1_000.0).round() as u64;
      if pts_ms > target_ms.saturating_add(2) && self.last_sample.is_some() {
        return Ok(self.last_sample.clone());
      }
      let sample = self.pending.take().expect("the pending sample exists");
      // Cache the native sample, not a deep copy of its full RGBA image.
      // Samples skipped while seeking never need CPU pixel conversion.
      self.last_sample = Some(sample.clone());
      if pts_ms.saturating_add(2) >= target_ms {
        return Ok(Some(sample));
      }
    }
  }

  fn converted(mut sample: arc::R<cm::SampleBuf>) -> Result<CapturedImage, String> {
    {
      let pixel_buffer = sample
        .image_buf_mut()
        .ok_or_else(|| "AVFoundation returned a video sample without pixels".to_owned())?;
      let width = pixel_buffer.width();
      let height = pixel_buffer.height();
      let stride = pixel_buffer.bytes_per_row();
      let flags = cv::pixel_buffer::LockFlags::READ_ONLY;
      unsafe { pixel_buffer.lock_base_addr(flags) }
        .result()
        .map_err(|error| error.to_string())?;
      let base = unsafe { pixel_buffer.base_address() } as *const u8;
      if base.is_null() {
        unsafe { pixel_buffer.unlock_lock_base_addr(flags) };
        return Err("AVFoundation returned an empty video frame".to_owned());
      }
      let mut rgba = vec![0_u8; width * height * 4];
      for row in 0..height {
        let source = unsafe { std::slice::from_raw_parts(base.add(row * stride), width * 4) };
        let target = &mut rgba[row * width * 4..(row + 1) * width * 4];
        for (source, target) in source.chunks_exact(4).zip(target.chunks_exact_mut(4)) {
          target[0] = source[2];
          target[1] = source[1];
          target[2] = source[0];
          target[3] = source[3];
        }
      }
      unsafe { pixel_buffer.unlock_lock_base_addr(flags) };
      Ok(CapturedImage {
        height: height as u32,
        rgba,
        width: width as u32,
      })
    }
  }
}

#[cfg(test)]
#[path = "video/tests.rs"]
mod tests;
