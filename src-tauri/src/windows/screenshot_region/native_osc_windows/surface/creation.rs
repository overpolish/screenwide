// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(crate) fn root(gpu: Arc<Gpu>, host: HWND, overlay: HWND) -> Result<Self, String> {
    // The overlay child is `WS_EX_NOREDIRECTIONBITMAP`, so DirectComposition
    // owns all of its content and the target is created topmost - that is what
    // keeps the region frame above the WebView2 sibling.
    Self::new(gpu, Kind::Root { host }, overlay, 0)
  }

  pub(crate) fn peer(
    gpu: Arc<Gpu>,
    hwnd: HWND,
    display_id: u32,
    bounds: Rect,
    scale: f64,
  ) -> Result<Self, String> {
    Self::new(gpu, Kind::Peer { bounds, scale }, hwnd, display_id)
  }

  pub(super) fn new(
    gpu: Arc<Gpu>,
    kind: Kind,
    hwnd: HWND,
    display_id: u32,
  ) -> Result<Self, String> {
    let target = unsafe { gpu.composition.CreateTargetForHwnd(hwnd, true) }
      .map_err(|error| format!("The Windows region OSC could not attach: {error}"))?;
    let root = unsafe { gpu.composition.CreateVisual() }.map_err(|error| error.to_string())?;
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
    let swap_chain = unsafe {
      gpu
        .factory
        .CreateSwapChainForComposition(&gpu.device, &description, None)
    }
    .and_then(|chain| chain.cast::<IDXGISwapChain3>())
    .map_err(|error| format!("The Windows region OSC swap chain failed: {error}"))?;
    let visual = unsafe { gpu.composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe {
      visual
        .SetContent(&swap_chain)
        .map_err(|error| error.to_string())?;
      // DirectComposition defaults to nearest-neighbour bitmap sampling.
      visual
        .SetBitmapInterpolationMode(DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR)
        .map_err(|error| error.to_string())?;
      root
        .AddVisual(&visual, true, None::<&IDCompositionVisual>)
        .map_err(|error| error.to_string())?;
      gpu
        .composition
        .Commit()
        .map_err(|error| error.to_string())?;
    }
    Ok(Self {
      gpu,
      kind,
      hwnd,
      swap_chain,
      _target: target,
      _root: root,
      _visual: visual,
      vertex_buffer: None,
      vertex_capacity: 0,
      vertices: Vec::new(),
      buffer_size: (2, 2),
      magnifier_source: None,
      snapshot: None,
      display_id,
      region: Rect::default(),
      visible: false,
      show_frame: true,
      show_handles: true,
      input_enabled: false,
      exclusion_rect: Rect::default(),
      magnifier: None,
      snapshot_presented: false,
      snapshot_composited: false,
      desktop_presented: false,
      desktop_offset: Point::default(),
      gesture_active: false,
      cursor: input::CursorShape::None,
      ocr: ocr::Chrome::default(),
      ruler: ruler::Ruler::default(),
      animating: false,
      shown: false,
      window_size: (0, 0),
      drawing: false,
      pending: false,
    })
  }
}
