// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Random-access frame decoding for paused frames and scrubbing, built on the
//! same `AVAssetReader` pipeline that playback uses.

use screenwide_imaging::CapturedImage;
use std::path::{Path, PathBuf};

use cidre::{arc, av};

use super::image::frame_position;
use screenwide_export::decode::macos::scrubber::{NativeFrameScrubber, NativePixelFrame};
use screenwide_export::decode::macos::video::{open_asset, NativeVideoReader};
use screenwide_workspace::scene::LayerPicture;

/// How far an exact seek rewinds before the target so the frame straddling a
/// seek position is still delivered.
const BACKWARD_TOLERANCE_MS: u64 = 100;

/// One video track with a cached `AVAsset` and a resumable sequential reader.
pub(super) struct PaneDecoder {
  asset: arc::R<av::UrlAsset>,
  duration_ms: u64,
  path: PathBuf,
  reader: Option<ScrubReader>,
  scrubber: Option<ScrubberReader>,
  pub(super) source_height: u32,
  pub(super) source_width: u32,
}

pub(super) enum DecodedFrame {
  Pixels(NativePixelFrame),
  Rgba(CapturedImage),
}

impl DecodedFrame {
  pub(super) fn dimensions(&self) -> (u32, u32) {
    match self {
      Self::Pixels(frame) => (frame.width, frame.height),
      Self::Rgba(frame) => (frame.width, frame.height),
    }
  }

  /// The frame as a workspace layer is drawn from it.
  pub(super) fn picture(&self) -> LayerPicture<'_> {
    match self {
      Self::Pixels(frame) => LayerPicture::Pixels(frame.as_ptr()),
      Self::Rgba(frame) => LayerPicture::Image(frame),
    }
  }

  pub(super) fn metadata(&self) -> CapturedImage {
    let (width, height) = self.dimensions();
    CapturedImage {
      height,
      rgba: Vec::new(),
      width,
    }
  }
}

struct ScrubReader {
  height: u32,
  last_target_ms: u64,
  reader: NativeVideoReader,
  width: u32,
}

struct ScrubberReader {
  height: u32,
  scrubber: NativeFrameScrubber,
  width: u32,
}

impl PaneDecoder {
  pub(super) fn open(
    path: &Path,
    source_width: u32,
    source_height: u32,
    duration_ms: u64,
  ) -> Result<Self, String> {
    let asset = open_asset(path)?;
    Ok(Self {
      asset,
      duration_ms,
      path: path.to_owned(),
      reader: None,
      scrubber: None,
      source_height,
      source_width,
    })
  }

  pub(super) fn decode_size(&self, factor: f64) -> (u32, u32) {
    let width = (f64::from(self.source_width) * factor).round().max(2.0) as u32;
    let height = (f64::from(self.source_height) * factor).round().max(2.0) as u32;
    (width.min(self.source_width), height.min(self.source_height))
  }

  fn reusable_forward(&self, target_ms: u64, width: u32, height: u32) -> bool {
    self.reader.as_ref().is_some_and(|reader| {
      reader.width == width
        && reader.height == height
        && target_ms >= reader.last_target_ms
        && target_ms - reader.last_target_ms < 2_000
    })
  }

  /// The frame at `position_ms`, `width` by `height`. An `exact` one comes
  /// from the reader playback uses rather than the player output scrubbing
  /// normally takes: that output can hand over a neighbouring frame while
  /// reporting the time asked for, which a matte made for each frame shows
  /// as a rim where the person was a frame before.
  pub(super) fn frame_at(
    &mut self,
    position_ms: u64,
    width: u32,
    height: u32,
    rough: bool,
    exact: bool,
  ) -> Result<Option<DecodedFrame>, String> {
    let target_ms = frame_position(position_ms, self.duration_ms);
    let scrubber_matches = exact
      || self
        .scrubber
        .as_ref()
        .is_some_and(|scrubber| scrubber.width == width && scrubber.height == height);
    if !scrubber_matches {
      // A pane changing on-screen size (zoom, bake toggle relayout) keeps its
      // warm player and only swaps the video output for the new dimensions.
      let resized = match self.scrubber.as_mut() {
        Some(existing) if existing.scrubber.resize(width, height) => {
          existing.width = width;
          existing.height = height;
          true
        }
        _ => false,
      };
      if !resized {
        self.scrubber = NativeFrameScrubber::open(&self.path, width, height)
          .ok()
          .map(|scrubber| ScrubberReader {
            height,
            scrubber,
            width,
          });
      }
    }
    if let Some(scrubber) = self.scrubber.as_ref().filter(|_| !exact) {
      if let Ok(frame) = scrubber.scrubber.frame_at(target_ms, rough) {
        return Ok(Some(DecodedFrame::Pixels(frame)));
      }
      self.scrubber = None;
    }
    // Besides exact frames, the reader is a fallback for codecs that a player
    // output cannot vend.
    if !self.reusable_forward(target_ms, width, height) {
      let start_ms = target_ms.saturating_sub(BACKWARD_TOLERANCE_MS);
      let reset = match self.reader.as_mut() {
        Some(reader) if reader.width == width && reader.height == height => {
          let reset = reader.reader.reset(start_ms, self.duration_ms).is_ok();
          if reset {
            reader.last_target_ms = target_ms;
          }
          reset
        }
        _ => false,
      };
      if !reset {
        self.reader = Some(ScrubReader {
          height,
          last_target_ms: target_ms,
          reader: NativeVideoReader::open(&self.asset, width, height, start_ms, self.duration_ms)?,
          width,
        });
      }
    }
    let reader = self
      .reader
      .as_mut()
      .expect("a scrub reader exists after opening");
    reader.last_target_ms = target_ms;
    if exact {
      return reader
        .reader
        .pixel_frame_at(target_ms)
        .map(|frame| frame.map(DecodedFrame::Pixels));
    }
    reader
      .reader
      .frame_at(target_ms)
      .map(|frame| frame.map(DecodedFrame::Rgba))
  }
}
