// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Device {
  /// The composition target for one overlay child: a premultiplied BGRA
  /// flip-discard chain under a topmost target, at a placeholder size until
  /// the first frame resizes it to the window.
  pub(crate) fn create_swap_chain(&self, hwnd: HWND) -> Result<CompositionSwapChain, String> {
    // The child is `WS_EX_NOREDIRECTIONBITMAP`, so DirectComposition owns all
    // of its content and the target is created topmost - that is what keeps
    // overlay content above a WebView2 sibling.
    let target = unsafe { self.composition.CreateTargetForHwnd(hwnd, true) }
      .map_err(|error| format!("The Windows overlay could not attach: {error}"))?;
    let root = unsafe { self.composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe { target.SetRoot(&root) }.map_err(|error| error.to_string())?;
    let description = DXGI_SWAP_CHAIN_DESC1 {
      Width: 2,
      Height: 2,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
      BufferCount: 2,
      Scaling: DXGI_SCALING_STRETCH,
      SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
      AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
      ..Default::default()
    };
    let chain = unsafe {
      self
        .factory
        .CreateSwapChainForComposition(&self.device, &description, None)
    }
    .and_then(|chain| chain.cast::<IDXGISwapChain3>())
    .map_err(|error| format!("The Windows overlay swap chain failed: {error}"))?;
    let visual = unsafe { self.composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe {
      visual
        .SetContent(&chain)
        .map_err(|error| error.to_string())?;
      // DirectComposition defaults to nearest-neighbour bitmap sampling.
      visual
        .SetBitmapInterpolationMode(DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR)
        .map_err(|error| error.to_string())?;
      root
        .AddVisual(&visual, true, None::<&IDCompositionVisual>)
        .map_err(|error| error.to_string())?;
    }
    self.commit()?;
    Ok(CompositionSwapChain {
      chain,
      size: (2, 2),
      _target: target,
      _root: root,
      _visual: visual,
    })
  }
}

impl CompositionSwapChain {
  /// Matches the buffers to a physical size. A resize is a reallocation and
  /// consecutive frames are usually the same size, so the current size is
  /// checked first. Every reference to a back buffer must be released before
  /// this is called.
  pub(crate) fn resize(&mut self, size: (u32, u32)) -> Result<(), String> {
    if size == self.size {
      return Ok(());
    }
    unsafe {
      self.chain.ResizeBuffers(
        2,
        size.0,
        size.1,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        DXGI_SWAP_CHAIN_FLAG(0),
      )
    }
    .map_err(|error| format!("The Windows overlay could not resize: {error}"))?;
    self.size = size;
    Ok(())
  }

  /// A render target view on the buffer the next present will show.
  pub(crate) fn back_buffer_view(
    &self,
    device: &ID3D11Device,
  ) -> Result<ID3D11RenderTargetView, String> {
    let index = unsafe { self.chain.GetCurrentBackBufferIndex() };
    let texture = unsafe { self.chain.GetBuffer::<ID3D11Texture2D>(index) }
      .map_err(|error| error.to_string())?;
    let resource: ID3D11Resource = texture.cast().map_err(|error| error.to_string())?;
    let mut view: Option<ID3D11RenderTargetView> = None;
    unsafe { device.CreateRenderTargetView(&resource, None, Some(&mut view)) }
      .map_err(|error| error.to_string())?;
    view.ok_or_else(|| "D3D11 created no overlay render target".to_owned())
  }

  /// Puts the drawn buffer on screen without ever blocking the calling thread
  /// on the compositor: overlay frames are driven from the pointer's thread.
  pub(crate) fn present(&self) -> Result<(), String> {
    unsafe { self.chain.Present(0, DXGI_PRESENT(0)) }
      .ok()
      .map_err(|error| error.to_string())
  }
}
