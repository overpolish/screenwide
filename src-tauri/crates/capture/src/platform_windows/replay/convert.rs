// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Captured BGRA frames into the NV12 pictures the encoder takes, on the GPU.
//!
//! The Sink Writer a recording uses converts colour itself; the encoder
//! transform the replay buffer drives does not. The conversion also takes the
//! frame out of the capture's own surface, which Windows Graphics Capture
//! recycles, into a texture the replay buffer owns for as long as the
//! encoder or a quiet-screen refresh needs it.

use windows::core::Interface;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_NV12, DXGI_RATIONAL, DXGI_SAMPLE_DESC};

use super::super::writer::win;
use crate::capture_geometry::CaptureRect;

/// Enough textures for an encoder that holds a few frames in flight, the
/// last frame kept for a refresh, and the next one being converted.
pub(super) const SLOTS: usize = 6;

/// Studio-range BT.601, the colour the recording writer's Sink Writer
/// converts to, so a clip's pixels match a recording's exactly. The first
/// field of `D3D11_VIDEO_PROCESSOR_COLOR_SPACE` leaves the matrix bit (2) at
/// BT.601, and the nominal range sits at bits 4 and 5.
const OUTPUT_COLOR_SPACE: u32 = (D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE_16_235.0 as u32) << 4;
/// Full-range RGB, as captured.
const INPUT_COLOR_SPACE: u32 = 0;

struct Slot {
  texture: ID3D11Texture2D,
  view: ID3D11VideoProcessorOutputView,
}

pub(super) struct Converter {
  context: ID3D11VideoContext,
  crop: Option<CaptureRect>,
  device: ID3D11VideoDevice,
  enumerator: ID3D11VideoProcessorEnumerator,
  processor: ID3D11VideoProcessor,
  slots: Vec<Slot>,
}

impl Converter {
  pub(super) fn new(
    device: &ID3D11Device,
    width: u32,
    height: u32,
    fps: u32,
    crop: Option<CaptureRect>,
    bind_flags: u32,
  ) -> Result<Self, String> {
    let video = win(device.cast::<ID3D11VideoDevice>())?;
    let immediate = win(unsafe { device.GetImmediateContext() })?;
    let context = win(immediate.cast::<ID3D11VideoContext>())?;
    let rate = DXGI_RATIONAL {
      Numerator: fps.max(1),
      Denominator: 1,
    };
    let description = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
      InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
      InputFrameRate: rate,
      InputWidth: crop.map_or(width, |crop| crop.width),
      InputHeight: crop.map_or(height, |crop| crop.height),
      OutputFrameRate: rate,
      OutputWidth: width,
      OutputHeight: height,
      Usage: D3D11_VIDEO_USAGE_OPTIMAL_SPEED,
    };
    let enumerator = win(unsafe { video.CreateVideoProcessorEnumerator(&description) })?;
    let processor = win(unsafe { video.CreateVideoProcessor(&enumerator, 0) })?;
    unsafe {
      context.VideoProcessorSetStreamFrameFormat(
        &processor,
        0,
        D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
      );
      context.VideoProcessorSetStreamAutoProcessingMode(&processor, 0, false);
      context.VideoProcessorSetStreamColorSpace(
        &processor,
        0,
        &D3D11_VIDEO_PROCESSOR_COLOR_SPACE {
          _bitfield: INPUT_COLOR_SPACE,
        },
      );
      context.VideoProcessorSetOutputColorSpace(
        &processor,
        &D3D11_VIDEO_PROCESSOR_COLOR_SPACE {
          _bitfield: OUTPUT_COLOR_SPACE,
        },
      );
      let whole = RECT {
        left: 0,
        top: 0,
        right: width as i32,
        bottom: height as i32,
      };
      context.VideoProcessorSetStreamDestRect(&processor, 0, true, Some(&whole));
      context.VideoProcessorSetOutputTargetRect(&processor, true, Some(&whole));
    }
    let slots = (0..SLOTS)
      .map(|_| slot(device, &video, &enumerator, width, height, bind_flags))
      .collect::<Result<_, _>>()?;
    Ok(Self {
      context,
      crop,
      device: video,
      enumerator,
      processor,
      slots,
    })
  }

  pub(super) fn texture(&self, slot: usize) -> &ID3D11Texture2D {
    &self.slots[slot].texture
  }

  /// Converts `source` into texture `slot`, cropped and scaled to the
  /// encoder's picture.
  pub(super) fn convert(&self, source: &ID3D11Texture2D, slot: usize) -> Result<(), String> {
    let mut source_description = D3D11_TEXTURE2D_DESC::default();
    unsafe { source.GetDesc(&mut source_description) };
    let area = match self.crop {
      Some(crop) => RECT {
        left: crop.x as i32,
        top: crop.y as i32,
        right: crop.x.saturating_add(crop.width) as i32,
        bottom: crop.y.saturating_add(crop.height) as i32,
      },
      None => RECT {
        left: 0,
        top: 0,
        right: source_description.Width as i32,
        bottom: source_description.Height as i32,
      },
    };
    let view_description = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
      FourCC: 0,
      ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
      Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
        Texture2D: D3D11_TEX2D_VPIV {
          MipSlice: 0,
          ArraySlice: 0,
        },
      },
    };
    let resource = win(source.cast::<ID3D11Resource>())?;
    let mut view = None;
    win(unsafe {
      self.device.CreateVideoProcessorInputView(
        &resource,
        &self.enumerator,
        &view_description,
        Some(&mut view),
      )
    })?;
    let stream = D3D11_VIDEO_PROCESSOR_STREAM {
      Enable: true.into(),
      pInputSurface: std::mem::ManuallyDrop::new(view),
      ..Default::default()
    };
    unsafe {
      self
        .context
        .VideoProcessorSetStreamSourceRect(&self.processor, 0, true, Some(&area));
    }
    let mut streams = [stream];
    let result = unsafe {
      self
        .context
        .VideoProcessorBlt(&self.processor, &self.slots[slot].view, 0, &streams)
    };
    // SAFETY: the view was moved into the stream description above and is
    // released here, once.
    drop(unsafe { std::mem::ManuallyDrop::take(&mut streams[0].pInputSurface) });
    win(result)
  }
}

fn slot(
  device: &ID3D11Device,
  video: &ID3D11VideoDevice,
  enumerator: &ID3D11VideoProcessorEnumerator,
  width: u32,
  height: u32,
  bind_flags: u32,
) -> Result<Slot, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: width,
    Height: height,
    MipLevels: 1,
    ArraySize: 1,
    Format: DXGI_FORMAT_NV12,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32 | bind_flags,
    CPUAccessFlags: 0,
    MiscFlags: 0,
  };
  let mut texture = None;
  win(unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) })?;
  let texture = texture.ok_or_else(|| "Direct3D created no NV12 texture".to_owned())?;
  let view_description = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
    ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
    Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
      Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
    },
  };
  let resource = win(texture.cast::<ID3D11Resource>())?;
  let mut view = None;
  win(unsafe {
    video.CreateVideoProcessorOutputView(&resource, enumerator, &view_description, Some(&mut view))
  })?;
  let view = view.ok_or_else(|| "Direct3D created no NV12 view".to_owned())?;
  Ok(Slot { texture, view })
}
