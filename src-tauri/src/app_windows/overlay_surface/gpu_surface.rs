// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! wgpu swap chains on DirectComposition visuals. wgpu creates the swap chain
//! and sets it as the visual's content the first time a surface is
//! configured; later configurations resize it in place.

use windows::{
  core::Interface,
  Win32::{
    Foundation::HWND,
    Graphics::{
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
        DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
      },
      Dxgi::IDXGIDevice,
    },
  },
};

use crate::gpu::Gpu;

/// The swap chain format: premultiplied BGRA, what DirectComposition blends.
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub(crate) enum Frame {
  Ready(wgpu::SurfaceTexture),
  /// Nothing can be drawn now; the next frame tries again.
  Skipped,
}

/// A wgpu surface on a visual someone else owns and places.
pub(crate) struct VisualSurface {
  surface: wgpu::Surface<'static>,
  config: wgpu::SurfaceConfiguration,
}

// The surface is reached behind its owner's lock.
unsafe impl Send for VisualSurface {}
unsafe impl Sync for VisualSurface {}

impl VisualSurface {
  /// A placeholder-sized surface, configured, so `visual` already shows its
  /// swap chain once the caller commits.
  pub(crate) fn new(gpu: &Gpu, visual: &IDCompositionVisual) -> Result<Self, String> {
    let surface = unsafe {
      gpu
        .instance
        .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CompositionVisual(
          visual.as_raw(),
        ))
    }
    .map_err(|error| format!("A composition surface could not be created: {error}"))?;
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: FORMAT,
      color_space: Default::default(),
      width: 2,
      height: 2,
      // Mailbox presents without waiting for the compositor: overlay frames
      // are drawn from the pointer's thread, and preview frames inside a
      // batch must all reach the compositor in one pass.
      present_mode: wgpu::PresentMode::Mailbox,
      desired_maximum_frame_latency: 2,
      alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
      view_formats: Vec::new(),
    };
    surface.configure(&gpu.device, &config);
    Ok(Self { surface, config })
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
        Err("A composition surface could not acquire a frame".to_owned())
      }
    }
  }
}

/// One overlay child drawn by wgpu. The composition tree is ours, so its
/// target stays topmost above a WebView2 sibling.
pub(crate) struct GpuSurface {
  surface: VisualSurface,
  _composition: IDCompositionDevice,
  /// Held only to keep the composition tree alive.
  _target: IDCompositionTarget,
  _root: IDCompositionVisual,
  _visual: IDCompositionVisual,
}

// The COM interfaces are process-wide tokens; each consumer reaches the
// surface behind its own registry lock.
unsafe impl Send for GpuSurface {}
unsafe impl Sync for GpuSurface {}

impl GpuSurface {
  /// A placeholder-sized surface until the first frame resizes it to the
  /// window. The child is `WS_EX_NOREDIRECTIONBITMAP`, so DirectComposition
  /// owns all of its content.
  pub(crate) fn new(gpu: &Gpu, hwnd: HWND) -> Result<Self, String> {
    // A swap chain is the visual's only content, so the composition device
    // needs no Direct3D device of its own.
    let composition: IDCompositionDevice =
      unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>) }
        .map_err(|error| format!("DirectComposition could not open: {error}"))?;
    let target = unsafe { composition.CreateTargetForHwnd(hwnd, true) }
      .map_err(|error| format!("The Windows overlay could not attach: {error}"))?;
    let root = unsafe { composition.CreateVisual() }.map_err(|error| error.to_string())?;
    let visual = unsafe { composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe {
      target.SetRoot(&root).map_err(|error| error.to_string())?;
      // DirectComposition defaults to nearest-neighbour bitmap sampling.
      visual
        .SetBitmapInterpolationMode(DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR)
        .map_err(|error| error.to_string())?;
      root
        .AddVisual(&visual, true, None::<&IDCompositionVisual>)
        .map_err(|error| error.to_string())?;
    }
    let surface = VisualSurface::new(gpu, &visual)?;
    unsafe { composition.Commit() }.map_err(|error| error.to_string())?;
    Ok(Self {
      surface,
      _composition: composition,
      _target: target,
      _root: root,
      _visual: visual,
    })
  }

  /// Matches the swap chain to a physical size.
  pub(crate) fn resize(&mut self, gpu: &Gpu, size: (u32, u32)) {
    self.surface.resize(gpu, size);
  }

  pub(crate) fn acquire(&self, gpu: &Gpu) -> Result<Frame, String> {
    self.surface.acquire(gpu)
  }
}
