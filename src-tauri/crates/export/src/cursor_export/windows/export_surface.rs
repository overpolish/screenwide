// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a Windows export borrows from the editor's live preview: the
//! compositor it draws each frame with and the GPU that compositor draws on.
//! The preview's own window owns both, so the app hands them over through
//! [`ExportSurface`].
//!
//! Media Foundation decodes and encodes on a plain Direct3D 11 device of the
//! export's own, on wgpu's adapter, rather than on the D3D11On12 layer the
//! preview decodes on. The hardware encoder on the layer took about 5 ms a
//! frame whatever it encoded: 188 frames a second at 1080p and 66 at 4K,
//! against 546 and 155 on a plain device, so the layer, not the GPU, bounded
//! every export. Decoding there too keeps its work off wgpu's queue. Frames
//! cross between the devices through textures both open, ordered by the
//! bridge's fences.

use std::sync::Arc;

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

use super::super::camera_effects::FrameEffects;
use screenwide_compositor::compositor::{self, BakeGeometry, ComposedFrame};
use screenwide_gpu::{BridgedTexture, D3d11Bridge};
use screenwide_output::ScreenshotOutputSettings;

/// The preview surface an export composites on.
pub trait ExportSurface: Send + Sync {
  fn compositor(&self) -> &compositor::Compositor;
  fn gpu(&self) -> &'static screenwide_gpu::Gpu;
  /// Holds the preview off the compositor until the guard drops, so an
  /// export frame and a preview frame never draw at once.
  fn hold(&self) -> Result<Box<dyn Held + '_>, String>;
}

/// A guard [`ExportSurface::hold`] returns; dropping it lets the preview draw.
pub trait Held {}

impl<T> Held for T {}

/// An offscreen instance of the live preview compositor. Its source and target
/// textures are allocated once and reused for every exported frame.
pub(crate) struct WindowsExportCompositor {
  /// The baked camera's frame, and its Direct3D 11 side.
  camera: Option<(compositor::SourceTexture, ID3D11Texture2D)>,
  output_size: (u32, u32),
  /// The screen's frame, and its Direct3D 11 side.
  source: (compositor::SourceTexture, ID3D11Texture2D),
  surface: Arc<dyn ExportSurface>,
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

impl WindowsExportCompositor {
  pub(crate) fn new(
    surface: Arc<dyn ExportSurface>,
    source_size: (u32, u32),
    camera_size: Option<(u32, u32)>,
    output_size: (u32, u32),
  ) -> Result<Self, String> {
    let gpu = surface.gpu();
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
      camera,
      output_size,
      source,
      surface,
      device,
      bridge,
      target,
    })
  }

  /// The device Media Foundation decodes and encodes on.
  pub(crate) fn device(&self) -> &ID3D11Device {
    &self.device
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

  /// The GPU the export draws on, which the camera's effects draw on too.
  pub(crate) fn gpu(&self) -> &'static screenwide_gpu::Gpu {
    self.surface.gpu()
  }

  /// `camera_composition` is what the baked camera has composed into its
  /// frame before the canvas draws it. `effects` are the camera's, drawn on
  /// the baked camera or on the screen where the screen is the camera.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn compose_with_camera(
    &self,
    texture: &ID3D11Texture2D,
    subresource: u32,
    settings: &ScreenshotOutputSettings,
    composition: ComposedFrame,
    camera: Option<(&ID3D11Texture2D, u32, BakeGeometry, bool, bool)>,
    camera_composition: Option<&compositor::CameraComposition>,
    // What this frame draws. A timed annotation has already had its reveal
    // resolved for this frame by the caller, which owns the timeline.
    annotations: &[screenwide_annotations::Annotation],
    // The captions over the frame, and the moment of the edited video it is.
    captions: Option<(
      &screenwide_text::captions::CaptionTrack,
      screenwide_text::captions::CaptionMoment,
    )>,
    mut effects: Option<&mut FrameEffects>,
  ) -> Result<ID3D11Texture2D, String> {
    let _held = self.surface.hold()?;
    let compositor = self.surface.compositor();
    let _every = compositor.hold_every_annotation_kind();
    // Both frames are copied in on the export's device, and wgpu waits for
    // the copies before anything below reads them.
    self.take(&self.source.1, texture, subresource)?;
    let camera_frame = self.camera.as_ref().zip(camera);
    if let Some(((_, destination), (frame, frame_subresource, ..))) = camera_frame {
      self.take(destination, frame, frame_subresource)?;
    }
    let gpu = self.surface.gpu();
    self.bridge.d3d11_to_gpu(gpu)?;
    let ms = (composition.seconds * 1_000.0).round().max(0.0) as u64;
    let source = match effects.as_deref_mut().filter(|effects| effects.on_screen()) {
      Some(effects) => effects.drawn(compositor, &self.source.0, ms)?,
      None => self.source.0.clone(),
    };
    let mut drawn_camera = None;
    if let Some(((camera_source, _), _)) = camera_frame {
      let camera_source = match effects.filter(|effects| !effects.on_screen()) {
        Some(effects) => effects.drawn(compositor, camera_source, ms)?,
        None => camera_source.clone(),
      };
      // The preview composes its camera in slot 0; the export keeps its own.
      drawn_camera = Some(match camera_composition {
        Some(composed) => compositor.composed_camera(&camera_source, composed, None, 1, false)?,
        None => camera_source,
      });
    }
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
    let mut prepared = screenwide_compositor::annotation_gpu::prepared_arrows(
      annotations,
      self.source.0.size,
      settings,
      None,
      None,
      None,
    )?;
    if let Some((track, moment)) = captions {
      prepared.draw_above(screenwide_compositor::annotation_gpu::caption_overlay(
        track,
        moment,
        self.output_size,
      ));
    }
    compositor.draw_with_camera(
      &self.target.view,
      &source,
      settings,
      composition,
      camera.and_then(|(_, _, geometry, drop_shadow, camera_on_top)| {
        drawn_camera
          .as_ref()
          .map(|source| (source, geometry, drop_shadow, camera_on_top))
      }),
      None,
      // Each frame bakes the annotations its settings carry. A timed annotation
      // has already had its reveal resolved for this frame, so the export draws
      // the same arrow the preview showed at that moment.
      &prepared,
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
