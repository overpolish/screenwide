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

/// What one format is drawn by and into: its own compositor, the texture it
/// draws, and the copy the encoder takes.
struct FormatTarget {
  compositor: Compositor,
  /// What each frame is drawn into, which the export's device copies out of.
  target: BridgedTexture,
  /// The drawn frame, copied out for the encoder. One serves every frame:
  /// the encoder copies it into a picture of its own on this device before
  /// taking the next, and the device runs its copies in order.
  out: ID3D11Texture2D,
}

/// The export's compositors and the textures they read and draw, made once
/// and reused for every frame. Each decoded frame is carried over once and
/// drawn into every format from there.
pub(crate) struct WindowsExportCanvas {
  gpu: std::sync::Arc<screenwide_gpu::Gpu>,
  /// The baked camera's frame, and its Direct3D 11 side.
  camera: Option<(compositor::SourceTexture, ID3D11Texture2D)>,
  /// The screen's frame, and its Direct3D 11 side.
  source: (compositor::SourceTexture, ID3D11Texture2D),
  /// The device Media Foundation decodes and encodes on.
  device: ID3D11Device,
  bridge: D3d11Bridge,
  /// Each format's, in the order the frame's draws are given.
  formats: Vec<FormatTarget>,
}

fn output_texture(device: &ID3D11Device, size: (u32, u32)) -> Result<ID3D11Texture2D, String> {
  let description = D3D11_TEXTURE2D_DESC {
    Width: size.0,
    Height: size.1,
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
  let mut texture = None;
  unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }
    .map_err(|error| format!("The Windows export target could not be created: {error}"))?;
  texture.ok_or_else(|| "D3D11 created no Windows export target".to_owned())
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
  /// A canvas drawing a format of each of `output_sizes` from every frame.
  pub(crate) fn new(
    source_size: (u32, u32),
    camera_size: Option<(u32, u32)>,
    output_sizes: &[(u32, u32)],
  ) -> Result<Self, String> {
    let gpu = screenwide_gpu::shared(screenwide_gpu::Workload::Heavy)?;
    let device = screenwide_gpu::video_device_on_gpu_adapter(&gpu)?;
    let bridge = D3d11Bridge::new(&gpu, &device)?;
    let source = bridged_source(&bridge, &gpu, source_size, "export source")?;
    let camera = camera_size
      .map(|size| bridged_source(&bridge, &gpu, size, "export camera"))
      .transpose()?;
    let formats = output_sizes
      .iter()
      .map(|&size| {
        let compositor = Compositor::new(&gpu)?;
        compositor.draw_generators_at_half_resolution();
        Ok(FormatTarget {
          compositor,
          target: bridge.shared_texture(
            &gpu,
            size,
            wgpu::TextureFormat::Bgra8Unorm,
            "export target",
          )?,
          out: output_texture(&device, size)?,
        })
      })
      .collect::<Result<Vec<_>, String>>()?;
    Ok(Self {
      gpu,
      camera,
      source,
      device,
      bridge,
      formats,
    })
  }

  /// The device Media Foundation decodes and encodes on.
  pub(crate) fn device(&self) -> &ID3D11Device {
    &self.device
  }

  /// The GPU the export draws on.
  pub(crate) fn gpu(&self) -> &std::sync::Arc<screenwide_gpu::Gpu> {
    &self.gpu
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
  /// in, drawn by each of `draws` at `moment` (its source time and the
  /// output time it covers) into its format's texture the encoder reads.
  pub(crate) fn compose(
    &self,
    draws: &mut [FrameDraw<'_>],
    screen: (&ID3D11Texture2D, u32),
    camera: Option<(&ID3D11Texture2D, u32)>,
    moment: (u64, f32),
  ) -> Result<Vec<ID3D11Texture2D>, String> {
    if draws.len() != self.formats.len() {
      return Err("The export frame has the wrong number of formats to draw".to_owned());
    }
    // Both frames are copied in on the export's device, and wgpu waits for
    // the copies before anything below reads them.
    self.take(&self.source.1, screen.0, screen.1)?;
    let camera = self.camera.as_ref().zip(camera);
    if let Some(((_, destination), (frame, subresource))) = camera {
      self.take(destination, frame, subresource)?;
    }
    let gpu = self.gpu();
    self.bridge.d3d11_to_gpu(gpu)?;
    for (format, draw) in self.formats.iter().zip(draws.iter_mut()) {
      draw.draw(
        &format.compositor,
        &format.target.view,
        &self.source.0,
        camera.map(|((camera, _), _)| camera),
        moment,
      )?;
    }
    // The export's device waits for the frames drawn above before copying
    // them out. The next frame's drawing waits for these copies too: they are
    // recorded before the next frame's own copies, which wgpu waits for.
    self.bridge.gpu_to_d3d11(gpu)?;
    let mut drawn = Vec::with_capacity(self.formats.len());
    for format in &self.formats {
      let destination: ID3D11Resource = format.out.cast().map_err(|error| error.to_string())?;
      let target: ID3D11Resource = format
        .target
        .d3d11
        .cast()
        .map_err(|error| error.to_string())?;
      unsafe { self.bridge.context.CopyResource(&destination, &target) };
      drawn.push(format.out.clone());
    }
    unsafe { self.bridge.context.Flush() };
    Ok(drawn)
  }
}
