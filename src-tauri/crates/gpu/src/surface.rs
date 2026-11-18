// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A wgpu surface on a platform presentation target someone else owns and
//! places: a DirectComposition visual on Windows, a `CAMetalLayer` on macOS.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::Gpu;

/// How often a configuration is tried before a surface gives up until its
/// next frame. wgpu waits for the GPU to go idle before configuring, and
/// fails the configuration when another thread submits work during that
/// wait; the device is shared, so that is a matter of timing, and the next
/// try a moment later usually finds the queue empty.
const CONFIGURE_ATTEMPTS: u32 = 5;
const CONFIGURE_RETRY: Duration = Duration::from_millis(5);

/// Every surface's format. Shaders write premultiplied BGRA, which is what
/// both DirectComposition and Core Animation blend.
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// How far past the picture drawn in it a swap chain on Windows reaches, in
/// pixels. Windows reallocates a swap chain only once every frame queued on
/// the shared device has finished, so a surface whose picture changes size
/// from frame to frame, as one following a window's dragged edge does, is
/// kept larger than its picture, in steps, and the picture drawn in its
/// top-left corner; DirectComposition shows the swap chain unscaled, clipped
/// to the picture. macOS resizes a drawable without waiting and stretches it
/// over its layer, so there the two match.
#[derive(Clone, Copy)]
pub struct Headroom {
  /// The steps the swap chain grows and shrinks in.
  pub step: u32,
  /// How far past the picture it reaches at least.
  pub margin: u32,
  /// How much larger than the picture's next step it may stay, so a picture
  /// growing and shrinking back does not reallocate.
  pub slack: u32,
}

impl Headroom {
  /// For a picture over an area a window's dragged edge resizes.
  pub const WINDOW: Self = Self {
    step: 256,
    margin: 0,
    slack: 256,
  };

  /// The swap chain to show `picture` in, given the `current` one.
  pub fn swap_chain(self, current: (u32, u32), picture: (u32, u32)) -> (u32, u32) {
    #[cfg(target_os = "windows")]
    {
      let edge = |current: u32, needed: u32| {
        let stepped = (needed + self.margin).div_ceil(self.step) * self.step;
        if current >= needed + self.margin && current <= stepped + self.slack {
          current
        } else {
          stepped
        }
      };
      (edge(current.0, picture.0), edge(current.1, picture.1))
    }
    #[cfg(not(target_os = "windows"))]
    {
      let _ = (self, current);
      picture
    }
  }
}

pub enum Frame {
  Ready(wgpu::SurfaceTexture),
  /// Nothing can be drawn now; the next frame tries again.
  Skipped,
}

pub struct Surface {
  surface: wgpu::Surface<'static>,
  config: wgpu::SurfaceConfiguration,
  /// Whether the last configuration took. Asking a surface whose
  /// configuration failed for a frame panics inside wgpu, so such a surface
  /// configures again before it hands out a frame.
  configured: AtomicBool,
  _live: super::lifetime::LiveSurface,
}

// The surface is reached behind its owner's lock.
unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
  /// A placeholder-sized surface on `visual`, configured, so the visual
  /// already shows its swap chain once the caller commits.
  #[cfg(target_os = "windows")]
  pub fn on_visual(
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
  pub unsafe fn on_metal_layer(
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
    // Metal offers only Fifo and Immediate. Fifo holds every present for a
    // display refresh, so a picture drawn twice for one pointer sample moves
    // at half the rate of the chrome over it. Immediate hands each drawable
    // to Core Animation at once, which composites the newest at the next
    // refresh, as Mailbox does on Windows. `PostMultiplied` is wgpu's name
    // for a non-opaque layer; Core Animation still blends its pixels as
    // premultiplied.
    Ok(Self::configured(
      gpu,
      surface,
      wgpu::PresentMode::Immediate,
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
    let surface = Self {
      surface,
      config,
      configured: AtomicBool::new(false),
      _live: super::lifetime::LiveSurface::new(gpu),
    };
    surface.configure(gpu);
    surface
  }

  /// Applies the configuration, trying again a few times if wgpu fails it,
  /// and records whether it took.
  fn configure(&self, gpu: &Gpu) {
    for attempt in 1..=CONFIGURE_ATTEMPTS {
      let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
      self.surface.configure(&gpu.device, &self.config);
      match pollster::block_on(scope.pop()) {
        None => {
          self.configured.store(true, Ordering::Release);
          return;
        }
        Some(error) if attempt == CONFIGURE_ATTEMPTS => {
          eprintln!("A surface could not be configured: {error}");
        }
        Some(_) => std::thread::sleep(CONFIGURE_RETRY),
      }
    }
    self.configured.store(false, Ordering::Release);
  }

  pub fn size(&self) -> (u32, u32) {
    (self.config.width, self.config.height)
  }

  /// Lets the frames be the destination of a copy as well as drawn into, for
  /// a picture composed elsewhere and copied in whole.
  pub fn accept_copies(&mut self, gpu: &Gpu) {
    self.config.usage |= wgpu::TextureUsages::COPY_DST;
    self.configure(gpu);
  }

  /// Presents each frame at a display refresh, one a refresh, and hands out
  /// a frame only once a refresh has freed one, so a caller drawing in a
  /// loop is paced by the display rather than a timer. For a picture that
  /// moves on its own, such as a recording played back; one that follows
  /// the pointer keeps presenting at once.
  pub fn wait_for_display(&mut self, gpu: &Gpu) {
    self.config.present_mode = wgpu::PresentMode::Fifo;
    self.configure(gpu);
  }

  /// Matches the swap chain to a physical size. No frame may be held across
  /// a resize. Consecutive frames are usually the same size, and a resize is
  /// a reallocation.
  pub fn resize(&mut self, gpu: &Gpu, size: (u32, u32)) {
    let size = (size.0.max(1), size.1.max(1));
    if size == self.size() {
      return;
    }
    self.config.width = size.0;
    self.config.height = size.1;
    self.configure(gpu);
  }

  /// Makes the swap chain large enough for a picture of `picture` physical
  /// pixels, with `headroom` on Windows. The picture is drawn in the frame's
  /// top-left corner; the rest of it is the caller's to clear or fill.
  pub fn fit(&mut self, gpu: &Gpu, picture: (u32, u32), headroom: Headroom) {
    let swap_chain = headroom.swap_chain(self.size(), (picture.0.max(1), picture.1.max(1)));
    self.resize(gpu, swap_chain);
  }

  pub fn acquire(&self, gpu: &Gpu) -> Result<Frame, String> {
    if !self.configured.load(Ordering::Acquire) {
      self.configure(gpu);
      if !self.configured.load(Ordering::Acquire) {
        return Ok(Frame::Skipped);
      }
    }
    match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Success(frame)
      | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Ok(Frame::Ready(frame)),
      wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
        Ok(Frame::Skipped)
      }
      wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
        self.configure(gpu);
        Ok(Frame::Skipped)
      }
      wgpu::CurrentSurfaceTexture::Validation => {
        Err("A surface could not acquire a frame".to_owned())
      }
    }
  }
}
