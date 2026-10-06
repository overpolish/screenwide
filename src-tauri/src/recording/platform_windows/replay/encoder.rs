// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The H.264 encoder transform, driven directly so its encoded samples can be
//! kept. The recording writer's Sink Writer encodes into a file and never
//! hands the samples back.
//!
//! Hardware encoders take the NV12 textures straight from the GPU and are
//! usually asynchronous: they ask for input and announce output through
//! events. Microsoft's software encoder is the fallback; it is synchronous
//! and reads system memory, so only that path copies frames off the GPU.

mod feed;
mod input;
mod output;
mod setup;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use windows::core::{Interface, GUID, PWSTR};
use windows::Win32::Graphics::Direct3D11::{
  ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D, D3D11_CPU_ACCESS_READ,
  D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Variant::{VARIANT, VT_UI4};

use super::super::writer::{attributes, win};
use super::EncodedFrame;
use crate::recording::encoding::bitrate_bps;
use setup::Candidate;

/// How long the encoder may take to ask for a frame or hand one back before
/// it is treated as stuck.
const STALL_TIMEOUT: Duration = Duration::from_secs(2);
/// How long a flush waits for frames a pipelining encoder still holds before
/// asking it to drain them.
const SETTLE_TIMEOUT: Duration = Duration::from_millis(250);
const POLL: Duration = Duration::from_micros(500);

#[derive(Clone, Copy)]
pub(super) struct Size {
  pub width: u32,
  pub height: u32,
  pub fps: u32,
}

#[derive(Clone, Copy)]
struct Streams {
  input: u32,
  output: u32,
}

/// Whether the encoder starts a keyframe when asked to through its codec
/// API. Some only honour codec values given before streaming starts.
#[derive(Clone, Copy, Eq, PartialEq)]
enum ForcedKeyframes {
  Untested,
  Honoured,
  Ignored,
}

/// How a frame handed to [`Encoder::encode`] has to be encoded.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Picture {
  /// Whatever the encoder chooses.
  Any,
  /// A keyframe if the encoder starts one when asked.
  PreferKeyframe,
  /// A keyframe, restarting the encoder if that is the only way to get one.
  Keyframe,
}

enum Input {
  /// Textures go to the encoder as they are.
  Gpu { _manager: IMFDXGIDeviceManager },
  /// Textures are read back into system memory first.
  Cpu {
    context: ID3D11DeviceContext,
    staging: ID3D11Texture2D,
  },
}

pub(super) struct Encoder {
  candidate: Candidate,
  codec: Option<ICodecAPI>,
  device: ID3D11Device,
  draining: bool,
  events: Option<IMFMediaEventGenerator>,
  forced: ForcedKeyframes,
  /// A frame asked to be a keyframe through the codec API, until the
  /// encoder hands it back and shows whether it is.
  forced_pts: Option<i64>,
  input: Input,
  /// Bind flags the transform needs on the textures it is given.
  input_bind_flags: u32,
  name: String,
  need_input: u32,
  output_size: u32,
  parameter_sets: Option<Vec<u8>>,
  /// Frames handed over and not yet back, with the texture slot each read.
  pending: VecDeque<(i64, usize)>,
  provides_samples: bool,
  size: Size,
  streams: Streams,
  transform: IMFTransform,
}

impl Encoder {
  /// The first encoder on `device`'s adapter that accepts the settings,
  /// hardware before software.
  pub(super) fn new(device: &ID3D11Device, size: Size) -> Result<Self, String> {
    let mut failures = Vec::new();
    for candidate in setup::candidates(device) {
      let name = setup::name(&candidate);
      match Self::start(device, candidate, size, name.clone()) {
        Ok(encoder) => {
          eprintln!(
            "Replay buffer encoding {}x{} at {} fps with {name}",
            size.width, size.height, size.fps
          );
          return Ok(encoder);
        }
        Err(error) => failures.push(format!("{name}: {error}")),
      }
    }
    Err(format!(
      "No H.264 encoder could keep the replay buffer ({})",
      failures.join("; ")
    ))
  }

  fn start(
    device: &ID3D11Device,
    candidate: Candidate,
    size: Size,
    name: String,
  ) -> Result<Self, String> {
    let transform = setup::transform(&candidate)?;
    let events = setup::unlock(&transform)?;
    let input = if matches!(candidate, Candidate::Hardware(_)) && setup::is_d3d11_aware(&transform)
    {
      let mut token = 0;
      let mut manager = None;
      win(unsafe { MFCreateDXGIDeviceManager(&mut token, &mut manager) })?;
      let manager = manager.ok_or_else(|| "Media Foundation created no D3D manager".to_owned())?;
      win(unsafe { manager.ResetDevice(device, token) })?;
      win(unsafe {
        transform.ProcessMessage(MFT_MESSAGE_SET_D3D_MANAGER, manager.as_raw() as usize)
      })?;
      Input::Gpu { _manager: manager }
    } else {
      Input::Cpu {
        context: win(unsafe { device.GetImmediateContext() })?,
        staging: input::staging_texture(device, size)?,
      }
    };
    let codec = transform.cast::<ICodecAPI>().ok();
    if let Some(codec) = &codec {
      setup::configure_codec(codec, size.fps);
    }
    let streams = setup::streams(&transform);
    setup::set_types(&transform, streams, size)?;
    let info = win(unsafe { transform.GetOutputStreamInfo(streams.output) })?;
    let provides_samples = info.dwFlags
      & (MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 | MFT_OUTPUT_STREAM_CAN_PROVIDE_SAMPLES.0) as u32
      != 0;
    win(unsafe { transform.ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0) })?;
    win(unsafe { transform.ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0) })?;
    let parameter_sets = unsafe { transform.GetOutputCurrentType(streams.output) }
      .ok()
      .and_then(|current| output::blob(&current, &MF_MT_MPEG_SEQUENCE_HEADER))
      .filter(|header| !header.is_empty());
    Ok(Self {
      input_bind_flags: setup::input_bind_flags(&transform),
      candidate,
      codec,
      device: device.clone(),
      draining: false,
      events,
      forced: ForcedKeyframes::Untested,
      forced_pts: None,
      input,
      name,
      need_input: 0,
      output_size: info.cbSize.max(size.width * size.height),
      parameter_sets,
      pending: VecDeque::new(),
      provides_samples,
      size,
      streams,
      transform,
    })
  }

  pub(super) fn input_bind_flags(&self) -> u32 {
    self.input_bind_flags
  }

  /// Whether the frame in texture `slot` is still being read by the encoder.
  pub(super) fn holds(&self, slot: usize) -> bool {
    self.pending.iter().any(|&(_, held)| held == slot)
  }

  pub(super) fn ignores_forced_keyframes(&self) -> bool {
    self.forced == ForcedKeyframes::Ignored
  }

  /// Whether the encoder still holds frames it has not handed back.
  pub(super) fn is_busy(&self) -> bool {
    !self.pending.is_empty()
  }

  fn shut_down(&self) {
    let _ = unsafe {
      self
        .transform
        .ProcessMessage(MFT_MESSAGE_NOTIFY_END_STREAMING, 0)
    };
    if let Candidate::Hardware(activate) = &self.candidate {
      let _ = unsafe { activate.ShutdownObject() };
    }
  }
}

impl Drop for Encoder {
  fn drop(&mut self) {
    self.shut_down();
  }
}
