// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Preview decode, playback and thumbnails.
//!
//! [`super::super::preview_platform`] owns the *presentation* half of the
//! preview (the compositing surface below the webview). This module owns the
//! *production* half: getting pixels for a given timestamp, at a given pane
//! size, onto that surface. The playback loop ([`playback`]) and what each
//! moment shows ([`present`]) are shared; a backend supplies:
//!
//! - `Frame` - a decoded pane frame, which implements [`present::PaneFrame`]
//!   to say what picture the scene draws from it. Frames never cross IPC.
//! - `open_playback` - what opens the range's [`playback::PaneDecoders`] on
//!   the playback thread.
//! - `COMPOSES_AT_PANE_SIZE` - whether playback decodes and composes at the
//!   on-screen pane size rather than the output's.
//! - `Stills` - the paused-frame and scrub decoding behind [`StillDecoder`],
//!   a [`still::StillFrames`].
//! - [`generate_thumbnails`] - the timeline strip.
//!
//! Geometry, layout and settings math deliberately stay above this line, in
//! [`super::layout`] and the shared output validation, so a new backend never
//! reimplements them.

mod playback;
mod present;
mod still;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
use macos as backend;
#[cfg(target_os = "windows")]
use windows as backend;

use std::sync::{atomic::AtomicBool, mpsc::SyncSender, Arc};

use super::{video::VideoFrame, PlayerSources};
use present::Moment;

pub(super) use backend::generate_thumbnails;
pub(super) use still::StillDecoder;

/// A playback tick's frames, presented together, and the channel that tells
/// the decode thread they have been, where its decoders hold their samples
/// until then.
pub(super) struct VideoFramePayload {
  screen: Option<backend::Frame>,
  camera: Option<backend::Frame>,
  moment: Moment,
  presented: Option<SyncSender<()>>,
}

pub(super) fn send_frame(sources: &PlayerSources, payload: VideoFramePayload) -> bool {
  let presented = present::present(
    sources,
    payload.screen.as_ref(),
    payload.camera.as_ref(),
    payload.moment,
  );
  if let Some(sender) = payload.presented {
    let _ = sender.send(());
  }
  presented
}

/// Starts the playback decode thread at `start_ms`, blocking until the
/// platform's decoders are open.
pub(super) fn spawn_video(
  sources: &PlayerSources,
  playback_factors: &[f64],
  start_ms: u64,
  playback_rate: f64,
  cancelled: Arc<AtomicBool>,
  sender: SyncSender<VideoFrame>,
) -> Result<std::thread::JoinHandle<()>, String> {
  if sources.playback_layout.panes.is_empty() {
    return Err("The recording has no video pane".to_owned());
  }
  let factors = (
    playback_factors.first().copied().unwrap_or(1.0),
    playback_factors.get(1).copied().unwrap_or(1.0),
  );
  playback::spawn(
    backend::open_playback(sources, factors, start_ms)?,
    start_ms,
    playback_rate,
    factors,
    cancelled,
    sender,
  )
}

/// How far each pane's playback decode and composition shrink toward its
/// on-screen size, matching what the still decoder presents.
pub(super) fn playback_factors(
  pane_target_sizes: &[(u32, u32)],
  sources: &PlayerSources,
) -> Vec<f64> {
  let panes = &sources.playback_layout.panes;
  if !backend::COMPOSES_AT_PANE_SIZE {
    return vec![1.0; panes.len()];
  }
  let composition = sources
    .composition_settings
    .as_ref()
    .and_then(|settings| settings.read().ok().map(|settings| settings.clone()));
  panes
    .iter()
    .enumerate()
    .map(|(index, pane)| {
      let output_width = composition
        .as_ref()
        .map_or(pane.source_width, |composition| {
          if index == 0 {
            composition.recording_output.primary.width
          } else {
            composition.recording_output.camera.width
          }
        });
      present::pane_factor(pane_target_sizes, index, output_width)
    })
    .collect()
}
