// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS's playback decoders: an AVFoundation reader per pane, decoding at
//! the on-screen pane size. Composing there too keeps playback frames
//! pixel-identical to paused frames and avoids Core Animation minifying a
//! full-resolution drawable, which visibly brightens thin glyphs.

use std::path::PathBuf;

use crate::editor::recording_preview_player::platform::playback::PaneDecoders;
use crate::editor::recording_preview_player::PlayerSources;
use screenwide_export::decode::macos::scrubber::NativePixelFrame;
use screenwide_export::decode::macos::video::{open_asset, NativeVideoReader};

pub(in crate::editor::recording_preview_player::platform) struct Readers {
  screen: NativeVideoReader,
  camera: Option<NativeVideoReader>,
}

impl PaneDecoders for Readers {
  type Frame = NativePixelFrame;
  // Each frame owns a Core Video reference of its own.
  const HOLDS_SAMPLES: bool = false;

  fn frames_at(&mut self, position_ms: u64) -> Result<[Option<NativePixelFrame>; 2], ()> {
    // Playback ends with the screen.
    let screen = match self.screen.pixel_frame_at(position_ms) {
      Ok(Some(frame)) => frame,
      Ok(None) | Err(_) => return Err(()),
    };
    let camera = match self.camera.as_mut() {
      Some(reader) => reader.pixel_frame_at(position_ms).map_err(|_| ())?,
      None => None,
    };
    Ok([Some(screen), camera])
  }
}

/// One pane's reader as it is to be opened.
struct Track {
  path: PathBuf,
  width: u32,
  height: u32,
  start_ms: u64,
  duration_ms: u64,
}

impl Track {
  fn open(self) -> Result<NativeVideoReader, String> {
    NativeVideoReader::open(
      &*open_asset(&self.path)?,
      self.width,
      self.height,
      self.start_ms,
      self.duration_ms,
    )
  }
}

/// What opens the playback readers at `start_ms` on the playback thread,
/// each pane's decode shrunk by its factor.
pub(in crate::editor::recording_preview_player::platform) fn open(
  sources: &PlayerSources,
  factors: (f64, f64),
  start_ms: u64,
) -> Result<impl FnOnce() -> Result<Readers, String> + Send + 'static, String> {
  let screen_pane = &sources.playback_layout.panes[0];
  let screen = Track {
    path: sources.screen_path.clone(),
    width: scaled_dimension(screen_pane.source_width, factors.0),
    height: scaled_dimension(screen_pane.source_height, factors.0),
    start_ms,
    duration_ms: sources.duration_ms,
  };
  let camera = match (
    sources.camera_path.as_ref(),
    sources.camera_duration_ms,
    sources.playback_layout.panes.get(1),
  ) {
    (Some(path), Some(duration_ms), Some(pane)) => Some(Track {
      path: path.clone(),
      width: scaled_dimension(pane.source_width, factors.1),
      height: scaled_dimension(pane.source_height, factors.1),
      start_ms: start_ms.min(duration_ms.saturating_sub(1)),
      duration_ms,
    }),
    _ => None,
  };
  Ok(move || {
    Ok(Readers {
      screen: screen.open()?,
      camera: camera.map(Track::open).transpose()?,
    })
  })
}

fn scaled_dimension(source: u32, factor: f64) -> u32 {
  ((f64::from(source) * factor.clamp(0.0, 1.0))
    .round()
    .max(2.0) as u32)
    .min(source)
}
