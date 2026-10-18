// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview backend: Media Foundation hardware decode into D3D11
//! textures, which the workspace scene copies into its own and presents
//! through a native flip-model swap chain. Live frames never enter system
//! memory or cross the Tauri IPC boundary.

mod pane_decoder;
mod playback;
mod still;
mod thumbnails;

use tauri::ipc::Channel;

use super::super::PlayerSources;
use super::present::{self, Moment, PaneFrame};
use screenwide_export::decode::windows::gpu_decoder::GpuFrame;
use screenwide_workspace::scene::LayerPicture;

pub(super) use playback::open as open_playback;

pub(super) use still::Stills;
pub(super) type Frame = GpuFrame;

/// The scene composes at the output's full size, and the swap chain scales
/// the result to the pane.
pub(super) const COMPOSES_AT_PANE_SIZE: bool = false;

impl PaneFrame for GpuFrame {
  fn picture(&self) -> LayerPicture<'_> {
    LayerPicture::Decoded((&self.texture, self.subresource, (self.width, self.height)))
  }

  fn size(&self) -> (u32, u32) {
    (self.width, self.height)
  }
}

/// Presents a paused moment: the frames' own time, nothing moving.
fn present_still(
  sources: &PlayerSources,
  screen: Option<&GpuFrame>,
  camera: Option<&GpuFrame>,
) -> bool {
  let Some(timestamp_ms) = screen.or(camera).map(|frame| frame.timestamp_ms) else {
    return false;
  };
  present::present(
    sources,
    screen,
    camera,
    Moment {
      timestamp_ms,
      frame_ms: 0.0,
      factors: (1.0, 1.0),
    },
  )
}

pub(crate) fn generate_thumbnails(sources: PlayerSources, count: u32, channel: Channel) {
  thumbnails::generate(sources, count, channel);
}
