// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
//! macOS's paused frames and timeline scrubs.
//!
//! Stills are decoded by the same `AVAssetReader` pipeline as playback and
//! composed by the same GPU compositor, so a paused frame is pixel-identical
//! to the playing frame at that position. Scrubbing decodes at the presented
//! size and the settled frame is refined at full resolution.
mod camera_pane;
mod worker;

use super::super::present::{pane_factor, scaled_output};
use super::super::still::{announce_ready, Seek, StillFrames};
use super::composition::gpu_still_overlay;
use super::image::frame_position;
use super::still_decode::{DecodedFrame, PaneDecoder};
use crate::editor::recording_preview_player::{PlayerSources, RecordingPreviewPlayerEvent};
use screenwide_workspace::scene::{NativeWorkspacePlacement, RecordingWorkspaceLayer};
use std::sync::atomic::Ordering;
use tauri::ipc::Channel;

struct CachedImages {
  camera: Option<DecodedFrame>,
  camera_ms: Option<u64>,
  /// Whether the frames were decoded exactly, for a matte drawn over them.
  exact: bool,
  screen: DecodedFrame,
  screen_ms: u64,
  sizes: (u32, u32, Option<(u32, u32)>),
}

pub(in crate::editor::recording_preview_player::platform) struct Stills {
  screen: PaneDecoder,
  camera: Option<PaneDecoder>,
  image_cache: Option<CachedImages>,
}

impl StillFrames for Stills {
  fn open(sources: &PlayerSources) -> Result<Self, String> {
    let screen_pane = &sources.playback_layout.panes[0];
    let screen = PaneDecoder::open(
      &sources.screen_path,
      screen_pane.source_width,
      screen_pane.source_height,
      sources.duration_ms,
    )?;
    let camera = match (
      sources.camera_path.as_deref(),
      sources.camera_duration_ms,
      sources.playback_layout.panes.get(1),
    ) {
      (Some(path), Some(duration_ms), Some(pane)) => Some(PaneDecoder::open(
        path,
        pane.source_width,
        pane.source_height,
        duration_ms,
      )?),
      _ => None,
    };
    Ok(Self {
      screen,
      camera,
      image_cache: None,
    })
  }

  fn show(
    &mut self,
    sources: &PlayerSources,
    seek: Seek,
    events: &Channel<RecordingPreviewPlayerEvent>,
  ) {
    worker::show(self, sources, seek, events);
  }
}
