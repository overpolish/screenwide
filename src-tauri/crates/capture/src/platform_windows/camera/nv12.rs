// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! NV12 camera frames into the BGRA textures the recording writers take, on
//! the GPU.
//!
//! Media Foundation offers a webcam's larger modes as NV12, decoded from the
//! camera's MJPEG. Converting those on the CPU costs several milliseconds a
//! frame even optimized; uploading the planes as they are and letting the
//! video processor convert them costs one copy. The writers still receive
//! BGRA, the format screen capture gives them, so their encoders see one kind
//! of input whatever the source.

use windows::core::Interface;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::{
  DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_NV12, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
};

use super::super::writer::win;

/// Studio-range BT.601 in, as Media Foundation's MJPEG decoder writes it and
/// the recording's encoder writes it back out. The first field of
/// `D3D11_VIDEO_PROCESSOR_COLOR_SPACE` leaves the matrix bit (2) at BT.601;
/// the nominal range sits at bits 4 and 5.
const INPUT_COLOR_SPACE: u32 = (D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE_16_235.0 as u32) << 4;
/// Full-range RGB out, as screen capture delivers it.
const OUTPUT_COLOR_SPACE: u32 = 0;

pub(super) struct Nv12Converter {
  context: ID3D11DeviceContext,
  device: ID3D11Device,
  enumerator: ID3D11VideoProcessorEnumerator,
  flipped: bool,
  height: u32,
  input: ID3D11Texture2D,
  input_view: ID3D11VideoProcessorInputView,
  /// The mirrored frame, kept between frames so mirroring never allocates.
  mirrored: Vec<u8>,
  processor: ID3D11VideoProcessor,
  video_context: ID3D11VideoContext,
  video_device: ID3D11VideoDevice,
  width: u32,
}

impl Nv12Converter {
  pub(super) fn new(
    device: &ID3D11Device,
    width: u32,
    height: u32,
    fps: u32,
    flipped: bool,
  ) -> Result<Self, String> {
    let video_device = win(device.cast::<ID3D11VideoDevice>())?;
    let context = win(unsafe { device.GetImmediateContext() })?;
    let video_context = win(context.cast::<ID3D11VideoContext>())?;
    let rate = DXGI_RATIONAL {
      Numerator: fps.max(1),
      Denominator: 1,
    };
    let description = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
      InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
      InputFrameRate: rate,
      InputWidth: width,
      InputHeight: height,
      OutputFrameRate: rate,
      OutputWidth: width,
      OutputHeight: height,
      Usage: D3D11_VIDEO_USAGE_OPTIMAL_SPEED,
    };
    let enumerator = win(unsafe { video_device.CreateVideoProcessorEnumerator(&description) })?;
    let processor = win(unsafe { video_device.CreateVideoProcessor(&enumerator, 0) })?;
    let whole = RECT {
      left: 0,
      top: 0,
      right: width as i32,
      bottom: height as i32,
    };
    unsafe {
      video_context.VideoProcessorSetStreamFrameFormat(
        &processor,
        0,
        D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
      );
      video_context.VideoProcessorSetStreamAutoProcessingMode(&processor, 0, false);
      video_context.VideoProcessorSetStreamColorSpace(
        &processor,
        0,
        &D3D11_VIDEO_PROCESSOR_COLOR_SPACE {
          _bitfield: INPUT_COLOR_SPACE,
        },
      );
      video_context.VideoProcessorSetOutputColorSpace(
        &processor,
        &D3D11_VIDEO_PROCESSOR_COLOR_SPACE {
          _bitfield: OUTPUT_COLOR_SPACE,
        },
      );
      video_context.VideoProcessorSetStreamSourceRect(&processor, 0, true, Some(&whole));
      video_context.VideoProcessorSetStreamDestRect(&processor, 0, true, Some(&whole));
      video_context.VideoProcessorSetOutputTargetRect(&processor, true, Some(&whole));
    }
    let input = texture(device, width, height, DXGI_FORMAT_NV12, 0)?;
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
    let mut input_view = None;
    win(unsafe {
      video_device.CreateVideoProcessorInputView(
        &win(input.cast::<ID3D11Resource>())?,
        &enumerator,
        &view_description,
        Some(&mut input_view),
      )
    })?;
    let input_view =
      input_view.ok_or_else(|| "Direct3D created no camera input view".to_owned())?;
    Ok(Self {
      context,
      device: device.clone(),
      enumerator,
      flipped,
      height,
      input,
      input_view,
      mirrored: Vec::new(),
      processor,
      video_context,
      video_device,
      width,
    })
  }

  /// A new BGRA texture holding `nv12`. Each frame gets its own texture
  /// because the writer keeps the last one to repeat and Media Foundation may
  /// still be encoding earlier ones.
  pub(super) fn convert(&mut self, nv12: &[u8]) -> Result<ID3D11Texture2D, String> {
    let (width, height) = (self.width as usize, self.height as usize);
    if nv12.len() != width * height * 3 / 2 {
      return Err("The camera produced an incomplete video frame".to_owned());
    }
    let source = if self.flipped {
      self.mirrored.resize(nv12.len(), 0);
      mirror_nv12(nv12, &mut self.mirrored, width, height);
      self.mirrored.as_slice()
    } else {
      nv12
    };
    // Both planes form NV12's one subresource: the chroma rows follow the
    // luma rows at the same pitch, exactly as Media Foundation packs them.
    unsafe {
      self
        .context
        .UpdateSubresource(&self.input, 0, None, source.as_ptr().cast(), self.width, 0);
    }
    let output = texture(
      &self.device,
      self.width,
      self.height,
      DXGI_FORMAT_B8G8R8A8_UNORM,
      // Matches the textures screen capture hands the writers; the Sink
      // Writer rejects a shader-resource-only texture with E_INVALIDARG.
      (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as u32,
    )?;
    let view_description = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
      ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
      Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
        Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
      },
    };
    let mut output_view = None;
    win(unsafe {
      self.video_device.CreateVideoProcessorOutputView(
        &win(output.cast::<ID3D11Resource>())?,
        &self.enumerator,
        &view_description,
        Some(&mut output_view),
      )
    })?;
    let output_view =
      output_view.ok_or_else(|| "Direct3D created no camera output view".to_owned())?;
    let mut streams = [D3D11_VIDEO_PROCESSOR_STREAM {
      Enable: true.into(),
      pInputSurface: std::mem::ManuallyDrop::new(Some(self.input_view.clone())),
      ..Default::default()
    }];
    let result = unsafe {
      self
        .video_context
        .VideoProcessorBlt(&self.processor, &output_view, 0, &streams)
    };
    // SAFETY: the view clone was moved into the stream description above and
    // is released here, once.
    drop(unsafe { std::mem::ManuallyDrop::take(&mut streams[0].pInputSurface) });
    win(result)?;
    Ok(output)
  }
}

fn texture(
  device: &ID3D11Device,
  width: u32,
  height: u32,
  format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
  bind_flags: u32,
) -> Result<ID3D11Texture2D, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: width,
    Height: height,
    MipLevels: 1,
    ArraySize: 1,
    Format: format,
    SampleDesc: DXGI_SAMPLE_DESC {
      Count: 1,
      Quality: 0,
    },
    Usage: D3D11_USAGE_DEFAULT,
    BindFlags: bind_flags,
    CPUAccessFlags: 0,
    MiscFlags: 0,
  };
  let mut texture = None;
  win(unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) })?;
  texture.ok_or_else(|| "Direct3D created no camera texture".to_owned())
}

/// Mirrors an NV12 frame left to right. Chroma is stored as U,V pairs, each
/// covering two pixels, so pairs move as a unit and keep their order.
pub(super) fn mirror_nv12(source: &[u8], target: &mut [u8], width: usize, height: usize) {
  let luma = width * height;
  let (source_y, source_uv) = source.split_at(luma);
  let (target_y, target_uv) = target.split_at_mut(luma);
  for (from, to) in source_y
    .chunks_exact(width)
    .zip(target_y.chunks_exact_mut(width))
  {
    to.copy_from_slice(from);
    to.reverse();
  }
  for (from, to) in source_uv
    .chunks_exact(width)
    .zip(target_uv.chunks_exact_mut(width))
  {
    for (pair, slot) in from.chunks_exact(2).rev().zip(to.chunks_exact_mut(2)) {
      slot.copy_from_slice(pair);
    }
  }
}
