// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Gpu {
  pub(super) fn new(host: HWND, editor: HWND) -> Result<Self, String> {
    let mut device = None;
    let mut context = None;
    unsafe {
      D3D11CreateDevice(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
        Some(&[D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0]),
        D3D11_SDK_VERSION,
        Some(&mut device),
        None,
        Some(&mut context),
      )
    }
    .map_err(|error| format!("The Windows preview GPU could not be opened: {error}"))?;
    let device = device.ok_or_else(|| "D3D11 returned no preview device".to_owned())?;
    let context = context.ok_or_else(|| "D3D11 returned no preview context".to_owned())?;
    let multithread: ID3D10Multithread = device.cast().map_err(|error| error.to_string())?;
    let _ = unsafe { multithread.SetMultithreadProtected(true) };
    let dxgi: IDXGIDevice = device.cast().map_err(|error| error.to_string())?;
    let adapter: IDXGIAdapter = unsafe { dxgi.GetAdapter() }.map_err(|error| error.to_string())?;
    let factory: IDXGIFactory2 =
      unsafe { adapter.GetParent() }.map_err(|error| error.to_string())?;
    let composition: IDCompositionDevice = unsafe { DCompositionCreateDevice(&dxgi) }
      .map_err(|error| format!("DirectComposition could not use the preview GPU: {error}"))?;
    // The non-topmost target is the critical Windows equivalent of inserting
    // the Metal view immediately below WKWebView: WebView2 remains a child
    // window above this GPU visual tree, so its DOM OSCs paint last.
    let target = unsafe { composition.CreateTargetForHwnd(host, false) }
      .map_err(|error| format!("The Windows preview compositor could not attach: {error}"))?;
    let root = unsafe { composition.CreateVisual() }
      .map_err(|error| format!("The Windows preview visual tree could not be created: {error}"))?;
    unsafe { target.SetRoot(&root) }
      .map_err(|error| format!("The Windows preview visual tree could not be attached: {error}"))?;
    let backdrop = Backdrop::new(&composition, &factory, &device, &root)?;
    backdrop.paint(&context, [0.09, 0.09, 0.10, 1.0])?;
    // The selection OSC lives in its own composition target on the editor
    // child window, which is kept above the WebView2 sibling. That window is
    // `WS_EX_NOREDIRECTIONBITMAP`, so DirectComposition owns all of its
    // content and the target is created topmost.
    let editor_target = unsafe { composition.CreateTargetForHwnd(editor, true) }
      .map_err(|error| format!("The Windows selection compositor could not attach: {error}"))?;
    let editor_root = unsafe { composition.CreateVisual() }
      .map_err(|error| format!("The Windows selection visual could not be created: {error}"))?;
    unsafe { editor_target.SetRoot(&editor_root) }
      .map_err(|error| format!("The Windows selection visual could not be attached: {error}"))?;
    let compositor = compositor::Compositor::new(&device)?;
    let audio_ribbon =
      audio_ribbon::AudioRibbon::new(&device, &context, &factory, &composition, &root)?;
    let selection =
      selection::SelectionOverlay::new(&device, &factory, &composition, &editor_root)?;
    unsafe { composition.Commit() }
      .map_err(|error| format!("The Windows preview compositor could not start: {error}"))?;
    Ok(Self {
      audio_ribbon: std::sync::Mutex::new(audio_ribbon),
      backdrop,
      compositor,
      composition,
      context,
      device,
      factory,
      root,
      selection: Mutex::new(selection),
      _editor_target: editor_target,
      _target: target,
    })
  }

  pub(super) fn pane(&self, below: Option<&IDCompositionVisual>) -> Result<Pane, String> {
    let description = DXGI_SWAP_CHAIN_DESC1 {
      Width: 2,
      Height: 2,
      Format: DXGI_FORMAT_B8G8R8A8_UNORM,
      Stereo: false.into(),
      SampleDesc: DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
      },
      BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
      BufferCount: 2,
      Scaling: DXGI_SCALING_STRETCH,
      SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
      AlphaMode: windows::Win32::Graphics::Dxgi::Common::DXGI_ALPHA_MODE_PREMULTIPLIED,
      Flags: 0,
    };
    let swap_chain = unsafe {
      self
        .factory
        .CreateSwapChainForComposition(&self.device, &description, None)
    }
    .map_err(|error| format!("The Windows preview swap chain could not be created: {error}"))?;
    let swap_chain = swap_chain
      .cast::<IDXGISwapChain3>()
      .map_err(|error| format!("The Windows preview requires a flip-model swap chain: {error}"))?;
    let visual = unsafe { self.composition.CreateVisual() }
      .map_err(|error| format!("The Windows preview pane visual could not be created: {error}"))?;
    let scale_transform = unsafe { self.composition.CreateScaleTransform() }.map_err(|error| {
      format!("The Windows preview pane transform could not be created: {error}")
    })?;
    let clip = unsafe { self.composition.CreateRectangleClip() }
      .map_err(|error| format!("The Windows preview pane clip could not be created: {error}"))?;
    (|| -> windows::core::Result<()> {
      unsafe {
        visual.SetContent(&swap_chain)?;
        // DirectComposition defaults to nearest-neighbour bitmap sampling.
        // Whatever residual scale the pane transform carries must resample
        // the frame rather than decimate it.
        visual.SetBitmapInterpolationMode(DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR)?;
        visual.SetTransform(&scale_transform)?;
        visual.SetClip(&clip)?;
        // Screenshot layer panes share one full-canvas rect, so sibling order
        // is the layer order: each pane sits directly above the nearest
        // lower-index pane, leaving higher indices frontmost as on macOS.
        self
          .root
          .AddVisual(&visual, true, Some(below.unwrap_or(&self.backdrop.visual)))?;
        self.composition.Commit()?;
      }
      Ok(())
    })()
    .map_err(|error| format!("The Windows preview pane could not be attached: {error}"))?;
    Ok(Pane {
      annotation_halo: None,
      base_rect: PreviewSurfaceRect {
        height: 0.0,
        width: 0.0,
        x: 0.0,
        y: 0.0,
      },
      buffer_size: (2, 2),
      clip,
      clip_edges: (0, 0, 2, 2),
      content_size: (2, 2),
      display_size: (2, 2),
      last_camera: None,
      last_composition: None,
      settings: None,
      magnifier: None,
      pending_geometry: false,
      pending_present: false,
      position: (0, 0),
      scale: 1.0,
      scale_transform,
      selection_stale: false,
      seen: true,
      source: None,
      source_token: None,
      swap_chain,
      visual,
    })
  }
}
