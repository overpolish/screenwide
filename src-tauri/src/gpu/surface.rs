// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A wgpu surface on a platform presentation target someone else owns and
//! places: a DirectComposition visual on Windows, a `CAMetalLayer` on macOS.

use super::Gpu;

/// Every surface's format. Shaders write premultiplied BGRA, which is what
/// both DirectComposition and Core Animation blend.
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub(crate) enum Frame {
  Ready(wgpu::SurfaceTexture),
  /// Nothing can be drawn now; the next frame tries again.
  Skipped,
}

pub(crate) struct Surface {
  surface: wgpu::Surface<'static>,
  config: wgpu::SurfaceConfiguration,
}

// The surface is reached behind its owner's lock.
unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
  /// A placeholder-sized surface on `visual`, configured, so the visual
  /// already shows its swap chain once the caller commits.
  #[cfg(target_os = "windows")]
  pub(crate) fn on_visual(
    gpu: &Gpu,
    visual: &windows::Win32::Graphics::DirectComposition::IDCompositionVisual,
  ) -> Result<Self, String> {
    use windows::core::Interface;
    let surface = unsafe {
      gpu
        .instance
        .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CompositionVisual(
          visual.as_raw(),
        ))
    }
    .map_err(|error| format!("A composition surface could not be created: {error}"))?;
    // Mailbox presents without waiting for the compositor: overlay frames
    // are drawn from the pointer's thread, and preview frames inside a batch
    // must all reach the compositor in one pass.
    Ok(Self::configured(
      gpu,
      surface,
      wgpu::PresentMode::Mailbox,
      wgpu::CompositeAlphaMode::PreMultiplied,
    ))
  }

  /// A placeholder-sized surface on `layer`, a `CAMetalLayer`.
  ///
  /// # Safety
  /// `layer` must be a live `CAMetalLayer`; wgpu retains it. Configuring sets
  /// the layer's drawable size, format and opacity, so the owner must leave
  /// those to [`Surface::resize`]. Main thread only.
  #[cfg(target_os = "macos")]
  pub(crate) unsafe fn on_metal_layer(
    gpu: &Gpu,
    layer: std::ptr::NonNull<std::ffi::c_void>,
  ) -> Result<Self, String> {
    let surface = unsafe {
      gpu
        .instance
        .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
          layer.as_ptr(),
        ))
    }
    .map_err(|error| format!("A Metal layer surface could not be created: {error}"))?;
    // Metal offers only Fifo and Immediate. `PostMultiplied` is wgpu's name
    // for a non-opaque layer; Core Animation still blends its pixels as
    // premultiplied.
    Ok(Self::configured(
      gpu,
      surface,
      wgpu::PresentMode::Fifo,
      wgpu::CompositeAlphaMode::PostMultiplied,
    ))
  }

  fn configured(
    gpu: &Gpu,
    surface: wgpu::Surface<'static>,
    present_mode: wgpu::PresentMode,
    alpha_mode: wgpu::CompositeAlphaMode,
  ) -> Self {
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: FORMAT,
      color_space: Default::default(),
      width: 2,
      height: 2,
      present_mode,
      desired_maximum_frame_latency: 2,
      alpha_mode,
      view_formats: Vec::new(),
    };
    surface.configure(&gpu.device, &config);
    Self { surface, config }
  }

  pub(crate) fn size(&self) -> (u32, u32) {
    (self.config.width, self.config.height)
  }

  /// Matches the swap chain to a physical size. No frame may be held across
  /// a resize. Consecutive frames are usually the same size, and a resize is
  /// a reallocation.
  pub(crate) fn resize(&mut self, gpu: &Gpu, size: (u32, u32)) {
    let size = (size.0.max(1), size.1.max(1));
    if size == self.size() {
      return;
    }
    self.config.width = size.0;
    self.config.height = size.1;
    self.surface.configure(&gpu.device, &self.config);
  }

  pub(crate) fn acquire(&self, gpu: &Gpu) -> Result<Frame, String> {
    match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame)
      | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Ok(Frame::Ready(frame)),
      wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
        Ok(Frame::Skipped)
      }
      wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
        self.surface.configure(&gpu.device, &self.config);
        Ok(Frame::Skipped)
      }
      wgpu::CurrentSurfaceTexture::Validation => {
        Err("A surface could not acquire a frame".to_owned())
      }
    }
  }
}
