// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview backend: Media Foundation hardware decode into D3D11
//! textures presented by native flip-model swap chains. Live frames never enter
//! system memory or cross the Tauri IPC boundary.

mod composed_frame;
mod decoder;
mod gpu_decoder;
mod present_frames;
mod still;
mod thumbnails;

pub(crate) use composed_frame::composed_frame_image;
pub(crate) use decoder::LumaReader;
pub(crate) use thumbnails::{each_source_frame, source_frame_image};

mod video_playback;
pub(crate) use video_playback::playback_factors;
pub(crate) use video_playback::spawn_video;

use std::{
  process::Child,
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, SyncSender, TrySendError},
    Arc, Mutex,
  },
  time::Duration,
};

use tauri::ipc::Channel;

use self::gpu_decoder::GpuFrame;
pub(crate) use self::gpu_decoder::GpuVideoReader;
use self::present_frames::present_native_frames;
use super::super::{
  video::{presentation_elapsed_ms, source_position_ms, VideoFrame},
  PlayerSources,
};

pub(crate) const NATIVE_STILLS: bool = true;
pub(crate) type StillDecoder = still::NativeStillDecoder;

/// One playback tick: the frame each stream decoded for it, presented
/// together, and the decoder's retained samples released once `presented`
/// hears back or is dropped.
pub(crate) enum VideoFramePayload {
  Native {
    screen: Option<GpuFrame>,
    camera: Option<GpuFrame>,
    presented: Option<SyncSender<()>>,
  },
}

pub(crate) fn send_frame(sources: &PlayerSources, payload: VideoFramePayload) -> bool {
  let VideoFramePayload::Native {
    screen,
    camera,
    presented,
  } = payload;
  // Playback exposes each frame for one frame interval, which is the window
  // a moving annotation smears over; a paused still exposes nothing.
  let frame_ms = sources
    .frames_per_second
    .filter(|rate| *rate > 0.0)
    .map_or(0.0, |rate| (1_000.0 / rate) as f32);
  let result = present_native_frames(sources, screen.as_ref(), camera.as_ref(), frame_ms);
  if let Some(presented) = presented {
    let _ = presented.send(());
  }
  result
}

pub(crate) fn generate_thumbnails(sources: PlayerSources, count: u32, channel: Channel) {
  thumbnails::generate(sources, count, channel);
}
