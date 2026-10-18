// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Windows side of an exported frame: Media Foundation's decoded frames
//! carried onto the shared GPU, the shared [`FrameDraw`] over them, and the
//! canvas carried back for the encoder.
//!
//! Media Foundation decodes and encodes on a plain Direct3D 11 device of the
//! export's own, on wgpu's adapter, rather than on the D3D11On12 layer the
//! preview decodes on. The hardware encoder on the layer took about 5 ms a
//! frame whatever it encoded: 188 frames a second at 1080p and 66 at 4K,
//! against 546 and 155 on a plain device, so the layer, not the GPU, bounded
//! every export. Decoding there too keeps its work off wgpu's queue. Frames
//! cross between the devices through textures both open, ordered by the
//! bridge's fences.

use ::windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Device, ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
      D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
  },
};

use super::super::frame_draw::FrameDraw;
use screenwide_compositor::compositor::{self, Compositor};
use screenwide_gpu::{BridgedTexture, D3d11Bridge};

/// The export's own compositor and the textures it reads and draws, made
/// once and reused for every frame.
pub(crate) struct WindowsExportCanvas {
  compositor: Compositor,
  /// The baked camera's frame, and its Direct3D 11 side.
  camera: Option<(compositor::SourceTexture, ID3D11Texture2D)>,
  output_size: (u32, u32),
  /// The screen's frame, and its Direct3D 11 side.
  source: (compositor::SourceTexture, ID3D11Texture2D),
  /// The device Media Foundation decodes and encodes on.
  device: ID3D11Device,
  bridge: D3d11Bridge,
  /// What each frame is drawn into, which the export's device copies out of.
  target: BridgedTexture,
}

/// A texture decoded frames are copied into on the export's device, as the
/// compositor reads it.
fn bridged_source(
  bridge: &D3d11Bridge,
  gpu: &screenwide_gpu::Gpu,
  size: (u32, u32),
  label: &'static str,
) -> Result<(compositor::SourceTexture, ID3D11Texture2D), String> {
  let bridged = bridge.shared_texture(gpu, size, compositor::FORMAT, label)?;
  Ok((
    compositor::SourceTexture {
      size,
      texture: bridged.texture,
      view: bridged.view,
      shared: None,
      picture: None,
      enhanced: false,
      cut_out: false,
    },
    bridged.d3d11,
  ))
}

impl WindowsExportCanvas {
  pub(crate) fn new(
    source_size: (u32, u32),
    camera_size: Option<(u32, u32)>,
    output_size: (u32, u32),
  ) -> Result<Self, String> {
    let gpu = screenwide_gpu::shared()?;
    let device = screenwide_gpu::video_device_on_gpu_adapter(gpu)?;
    let bridge = D3d11Bridge::new(gpu, &device)?;
    let source = bridged_source(&bridge, gpu, source_size, "export source")?;
    let camera = camera_size
      .map(|size| bridged_source(&bridge, gpu, size, "export camera"))
      .transpose()?;
    let target = bridge.shared_texture(
      gpu,
      output_size,
      wgpu::TextureFormat::Bgra8Unorm,
      "export target",
    )?;
    Ok(Self {
      compositor: Compositor::new(gpu)?,
      camera,
      output_size,
      source,
      device,
      bridge,
      target,
    })
  }

  /// The device Media Foundation decodes and encodes on.
  pub(crate) fn device(&self) -> &ID3D11Device {
    &self.device
  }

  /// The GPU the export draws on.
  pub(crate) fn gpu(&self) -> &'static screenwide_gpu::Gpu {
    self.compositor.gpu()
  }

  /// Copies a decoded frame into the texture the compositor reads it from.
  fn take(
    &self,
    destination: &ID3D11Texture2D,
    frame: &ID3D11Texture2D,
    subresource: u32,
  ) -> Result<(), String> {
    let destination: ID3D11Resource = destination.cast().map_err(|error| error.to_string())?;
    let frame: ID3D11Resource = frame.cast().map_err(|error| error.to_string())?;
    unsafe {
      self.bridge.context.CopySubresourceRegion(
        &destination,
        0,
        0,
        0,
        0,
        &frame,
        subresource,
        None,
      );
    }
    Ok(())
  }

  /// The decoded `screen` frame, and `camera`'s where the camera is baked
  /// in, drawn by `draw` at `moment` (its source time and the output time it
  /// covers) into a texture of its own for the encoder.
  pub(crate) fn compose(
    &self,
    draw: &mut FrameDraw<'_>,
    screen: (&ID3D11Texture2D, u32),
    camera: Option<(&ID3D11Texture2D, u32)>,
    moment: (u64, f32),
  ) -> Result<ID3D11Texture2D, String> {
    // Both frames are copied in on the export's device, and wgpu waits for
    // the copies before anything below reads them.
    self.take(&self.source.1, screen.0, screen.1)?;
    let camera = self.camera.as_ref().zip(camera);
    if let Some(((_, destination), (frame, subresource))) = camera {
      self.take(destination, frame, subresource)?;
    }
    let gpu = self.gpu();
    self.bridge.d3d11_to_gpu(gpu)?;
    // Sink Writer retains DXGI surfaces and feeds the hardware encoder
    // asynchronously. A single repainted render target therefore lets a later
    // frame overwrite an earlier sample before Media Foundation consumes it.
    // Give each submitted sample its own texture; MF's sample owns that texture
    // until encoding completes and naturally bounds outstanding allocations
    // through Sink Writer backpressure.
    let description = D3D11_TEXTURE2D_DESC {
      Width: self.output_size.0,
      Height: self.output_size.1,
      MipLevels: 1,
      ArraySize: 1,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      Usage: D3D11_USAGE_DEFAULT,
      BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
      ..Default::default()
    };
    let mut copy = None;
    unsafe {
      self
        .device
        .CreateTexture2D(&description, None, Some(&mut copy))
    }
    .map_err(|error| format!("The Windows export target could not be created: {error}"))?;
    let copy = copy.ok_or_else(|| "D3D11 created no Windows export target".to_owned())?;
    draw.draw(
      &self.compositor,
      &self.target.view,
      &self.source.0,
      camera.map(|((camera, _), _)| camera),
      moment,
    )?;
    // The export's device waits for the frame drawn above before copying it
    // out. The next frame's drawing waits for this copy too: it is recorded
    // before the next frame's own copies, which wgpu waits for.
    let destination: ID3D11Resource = copy.cast().map_err(|error| error.to_string())?;
    let drawn: ID3D11Resource = self
      .target
      .d3d11
      .cast()
      .map_err(|error| error.to_string())?;
    self.bridge.gpu_to_d3d11(gpu)?;
    unsafe {
      self.bridge.context.CopyResource(&destination, &drawn);
      self.bridge.context.Flush();
    }
    Ok(copy)
  }
}
