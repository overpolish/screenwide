// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS preview backend: AVFoundation decode, shared wgpu composition.
//!
//! Playback and stills share one `AVAssetReader` pipeline and one GPU
//! compositor, so a paused frame is pixel-identical to the playing frame at
//! that position. Decoded planes stay in Core Video and are drawn by the
//! workspace scene without a copy; nothing crosses IPC.

mod composition;
mod image;
mod playback;
mod still;
mod still_decode;
mod thumbnails;

use tauri::ipc::Channel;

use super::super::PlayerSources;
use super::present::PaneFrame;
use screenwide_export::decode::macos::scrubber::NativePixelFrame;
use screenwide_workspace::scene::LayerPicture;

pub(super) use playback::open as open_playback;

pub(super) use still::Stills;
pub(super) type Frame = NativePixelFrame;

/// Playback decodes and composes at the on-screen pane size; see
/// [`playback`].
pub(super) const COMPOSES_AT_PANE_SIZE: bool = true;

impl PaneFrame for NativePixelFrame {
  fn picture(&self) -> LayerPicture<'_> {
    LayerPicture::Pixels(self.as_ptr())
  }

  fn size(&self) -> (u32, u32) {
    (self.width, self.height)
  }
}

pub(crate) fn generate_thumbnails(sources: PlayerSources, count: u32, channel: Channel) {
  thumbnails::generate(sources, count, channel);
}
