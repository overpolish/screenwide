// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a Windows export borrows from the editor's live preview: the
//! Direct3D 11 layer it decodes and encodes on, and the compositor it draws
//! each frame with. The preview's own window owns both, so the app hands them
//! over through [`ExportSurface`].

use std::sync::Arc;

use ::windows::{
  core::Interface,
  Win32::Graphics::{
    Direct3D11::{
      ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET, D3D11_TEXTURE2D_DESC,
      D3D11_USAGE_DEFAULT,
    },
    Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
  },
};

use super::super::camera_effects::FrameEffects;
use screenwide_compositor::compositor::{self, BakeGeometry, ComposedFrame};
use screenwide_gpu::D3d11Layer;
use screenwide_output::ScreenshotOutputSettings;

/// The preview surface an export composites on.
pub trait ExportSurface: Send + Sync {
  fn d3d11(&self) -> &'static D3d11Layer;
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
  camera: Option<compositor::SourceTexture>,
  output_size: (u32, u32),
  source: compositor::SourceTexture,
  surface: Arc<dyn ExportSurface>,
  /// What each frame is drawn into, which Direct3D 11 copies out of.
  target: screenwide_gpu::SharedTexture,
}

impl WindowsExportCompositor {
  pub(crate) fn new(
    surface: Arc<dyn ExportSurface>,
    source_size: (u32, u32),
    camera_size: Option<(u32, u32)>,
    output_size: (u32, u32),
  ) -> Result<Self, String> {
    let compositor = surface.compositor();
    let source = compositor.source(surface.d3d11(), source_size)?;
    let target = surface.d3d11().shared_texture(
      surface.gpu(),
      output_size,
      wgpu::TextureFormat::Bgra8Unorm,
      "export target",
    )?;
    let camera = camera_size
      .map(|size| compositor.source(surface.d3d11(), size))
      .transpose()?;
    Ok(Self {
      camera,
      output_size,
      source,
      surface,
      target,
    })
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
      &screenwide_captions::CaptionTrack,
      screenwide_captions::CaptionMoment,
    )>,
    mut effects: Option<&mut FrameEffects>,
  ) -> Result<ID3D11Texture2D, String> {
    let _held = self.surface.hold()?;
    let compositor = self.surface.compositor();
    let _every = compositor.hold_every_annotation_kind();
    let d3d11 = self.surface.d3d11();
    compositor.copy_source(d3d11, &self.source, texture, subresource)?;
    let ms = (composition.seconds * 1_000.0).round().max(0.0) as u64;
    let source = match effects.as_deref_mut().filter(|effects| effects.on_screen()) {
      Some(effects) => effects.drawn(compositor, &self.source, ms)?,
      None => self.source.clone(),
    };
    let mut drawn_camera = None;
    if let (Some(camera_source), Some((camera_texture, camera_subresource, _, _, _))) =
      (&self.camera, camera)
    {
      compositor.copy_source(d3d11, camera_source, camera_texture, camera_subresource)?;
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
      d3d11
        .device
        .CreateTexture2D(&description, None, Some(&mut copy))
    }
    .map_err(|error| format!("The Windows export target could not be created: {error}"))?;
    let copy = copy.ok_or_else(|| "D3D11 created no Windows export target".to_owned())?;
    let mut prepared = screenwide_compositor::annotation_gpu::prepared_arrows(
      annotations,
      self.source.size,
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
    // The frame was submitted to the shared queue above, so the copy the
    // encoder reads, submitted after it, sees it whole.
    let destination: ID3D11Resource = copy.cast().map_err(|error| error.to_string())?;
    d3d11.with(&[&self.target], |context, textures| {
      let drawn: ID3D11Resource = textures[0].cast().map_err(|error| error.to_string())?;
      unsafe { context.CopyResource(&destination, &drawn) };
      Ok::<_, String>(())
    })??;
    Ok(copy)
  }
}
