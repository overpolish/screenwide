// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pin tracker's decoder: brightness alone, at the tracking size.
//!
//! The decoder is asked for full-range 4:2:0 at the tracking size, so the
//! hardware scales each frame down and the tracker copies the brightness
//! plane out as it stands: no colour conversion and no full-size pixels ever
//! reach the CPU.

use std::path::Path;

use cidre::{arc, av, cm, cv};

use super::{open_asset, NativeVideoReader};
use crate::editor::annotations::pin::{FrameSource, LumaFrame};

pub(crate) struct LumaReader {
  asset: arc::R<av::UrlAsset>,
  duration_ms: u64,
  source: (u32, u32),
  size: (u32, u32),
  reader: Option<NativeVideoReader>,
  /// Whether the last range was read to its end. AVFoundation only lets a
  /// random-access output move to a new range once the old one is spent;
  /// otherwise a fresh reader is opened.
  drained: bool,
}

impl LumaReader {
  /// A reader for `path` whose frames are at most `longest_side` pixels on
  /// their longer side, and never larger than the source.
  pub(crate) fn open(path: &Path, duration_ms: u64, longest_side: u32) -> Result<Self, String> {
    let asset = open_asset(path)?;
    let tracks =
      tauri::async_runtime::block_on(asset.load_tracks_with_media_type(av::MediaType::video()))
        .map_err(|error| error.to_string())?;
    let track = tracks
      .get(0)
      .map_err(|_| "The recording has no video track".to_owned())?;
    let natural = track.natural_size();
    let source = (natural.width.round() as u32, natural.height.round() as u32);
    let longest = source.0.max(source.1).max(1);
    let factor = (f64::from(longest_side) / f64::from(longest)).min(1.0);
    let even = |value: u32| ((f64::from(value) * factor).round() as u32).max(2) & !1;
    Ok(Self {
      asset,
      duration_ms,
      source,
      size: (even(source.0), even(source.1)),
      reader: None,
      drained: false,
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
    // AVFoundation hands over the frame already showing at the start of a
    // range stamped with the range's start rather than its own time. Two legs
    // reading the same moment from different starts would then disagree about
    // when a frame was, and the path would take one leg's answer alone there.
    // Reading from a millisecond early puts that stamp before `start_ms`,
    // where it is skipped, so every frame keeps its own time, as Media
    // Foundation's do on Windows.
    let from_ms = start_ms.saturating_sub(1);
    let reused = match self.reader.take() {
      Some(mut reader) if self.drained => reader.reset(from_ms, end_ms).is_ok().then_some(reader),
      _ => None,
    };
    let mut reader = match reused {
      Some(reader) => reader,
      None => NativeVideoReader::open_as(
        &self.asset,
        self.size.0,
        self.size.1,
        cv::PixelFormat::_420F,
        from_ms,
        end_ms,
      )?,
    };
    self.drained = false;
    loop {
      let Some(sample) = reader
        .output
        .next_sample_buf()
        .map_err(|error| error.to_string())?
      else {
        self.drained = true;
        break;
      };
      let ms = (sample.pts().as_secs().max(0.0) * 1_000.0).round() as u64;
      if ms < start_ms {
        continue;
      }
      if !each(luma_frame(sample, ms)?) {
        break;
      }
    }
    self.reader = Some(reader);
    Ok(())
  }
}

/// The brightness plane of a full-range 4:2:0 sample.
fn luma_frame(mut sample: arc::R<cm::SampleBuf>, ms: u64) -> Result<LumaFrame, String> {
  let pixel_buffer = sample
    .image_buf_mut()
    .ok_or_else(|| "AVFoundation returned a video sample without pixels".to_owned())?;
  let flags = cv::pixel_buffer::LockFlags::READ_ONLY;
  unsafe { pixel_buffer.lock_base_addr(flags) }
    .result()
    .map_err(|error| error.to_string())?;
  let (width, height) = (pixel_buffer.plane_width(0), pixel_buffer.plane_height(0));
  let stride = pixel_buffer.plane_bytes_per_row(0);
  let base = pixel_buffer.plane_base_address(0);
  if base.is_null() || stride < width {
    unsafe { pixel_buffer.unlock_lock_base_addr(flags) };
    return Err("AVFoundation returned an empty video frame".to_owned());
  }
  let mut pixels = Vec::with_capacity(width * height);
  for row in 0..height {
    pixels.extend_from_slice(unsafe { std::slice::from_raw_parts(base.add(row * stride), width) });
  }
  unsafe { pixel_buffer.unlock_lock_base_addr(flags) };
  Ok(LumaFrame {
    ms,
    width: width as u32,
    height: height as u32,
    pixels,
  })
}

#[cfg(test)]
#[path = "luma_bench_tests.rs"]
mod bench_tests;

#[cfg(test)]
mod tests {
  use super::*;

  /// A range that starts between two frames yields only frames with their own
  /// times, the same ones a read from the start of the movie sees: legs read
  /// the same moments from different starts, and have to agree on them.
  #[test]
  fn a_range_starting_between_frames_keeps_every_frames_own_time() {
    let directory = std::env::temp_dir().join(format!("screenwide-luma-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("frames.mp4");
    let status = std::process::Command::new(crate::editor::media_preview::ffmpeg_path())
      .args([
        "-y",
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "testsrc2=size=64x64:rate=10:duration=1",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
      ])
      .arg(&path)
      .status()
      .unwrap();
    assert!(status.success());
    let mut reader = LumaReader::open(&path, 1_000, 64).unwrap();
    let mut read = |start, end| {
      let mut frames = Vec::new();
      reader
        .read(start, end, &mut |frame| {
          frames.push((frame.ms, frame.pixels));
          true
        })
        .unwrap();
      frames
    };
    let whole = read(0, 1_000);
    let part = read(250, 1_000);
    assert_eq!(part.first().map(|(ms, _)| *ms), Some(300));
    for (ms, pixels) in &part {
      let same = whole.iter().find(|(whole_ms, _)| whole_ms == ms);
      assert!(
        same.is_some_and(|(_, whole_pixels)| whole_pixels == pixels),
        "{ms}"
      );
    }
    std::fs::remove_dir_all(directory).unwrap();
  }
}
