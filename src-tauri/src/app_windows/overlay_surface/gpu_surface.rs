// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! wgpu swap chains on DirectComposition visuals. wgpu creates the swap chain
//! and sets it as the visual's content the first time a surface is
//! configured; later configurations resize it in place.

use windows::Win32::{
  Foundation::HWND,
  Graphics::{
    DirectComposition::{
      DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
      DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
    },
    Dxgi::IDXGIDevice,
  },
};

use crate::gpu::surface::{Frame, Surface};
use crate::gpu::Gpu;

/// One overlay child drawn by wgpu. The composition tree is ours, so its
/// target stays topmost above a WebView2 sibling.
pub(crate) struct GpuSurface {
  surface: Surface,
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
    let surface = Surface::on_visual(gpu, &visual)?;
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
