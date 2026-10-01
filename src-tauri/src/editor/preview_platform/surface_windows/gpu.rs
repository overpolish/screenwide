// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Gpu {
  pub(super) fn new(host: HWND, editor: HWND) -> Result<Self, String> {
    let shared = crate::gpu::shared()?;
    let d3d11 = crate::gpu::d3d11()?;
    // Every visual's content is a swap chain, so the composition device needs
    // no Direct3D device of its own.
    let composition: IDCompositionDevice =
      unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>) }
        .map_err(|error| format!("DirectComposition could not open for the preview: {error}"))?;
    // The non-topmost target is the critical Windows equivalent of inserting
    // the Metal view immediately below WKWebView: WebView2 remains a child
    // window above this GPU visual tree, so its DOM OSCs paint last.
    let target = unsafe { composition.CreateTargetForHwnd(host, false) }
      .map_err(|error| format!("The Windows preview compositor could not attach: {error}"))?;
    let root = unsafe { composition.CreateVisual() }
      .map_err(|error| format!("The Windows preview visual tree could not be created: {error}"))?;
    unsafe { target.SetRoot(&root) }
      .map_err(|error| format!("The Windows preview visual tree could not be attached: {error}"))?;
    let backdrop = Backdrop::new(shared, &composition, &root)?;
    backdrop.paint(shared, [0.09, 0.09, 0.10, 1.0])?;
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
    let compositor = compositor::Compositor::new(shared)?;
    let audio_ribbon = audio_ribbon::AudioRibbon::new(shared, &composition, &root)?;
    let selection = selection::SelectionOverlay::new(shared, &composition, &editor_root)?;
    unsafe { composition.Commit() }
      .map_err(|error| format!("The Windows preview compositor could not start: {error}"))?;
    Ok(Self {
      audio_ribbon: std::sync::Mutex::new(audio_ribbon),
      backdrop,
      d3d11,
      compositor,
      composition,
      root,
      selection: Mutex::new(selection),
      shared,
      _editor_target: editor_target,
      _target: target,
    })
  }

  pub(super) fn pane(&self, below: Option<&IDCompositionVisual>) -> Result<Pane, String> {
    let visual = unsafe { self.composition.CreateVisual() }
      .map_err(|error| format!("The Windows preview pane visual could not be created: {error}"))?;
    let scale_transform = unsafe { self.composition.CreateScaleTransform() }.map_err(|error| {
      format!("The Windows preview pane transform could not be created: {error}")
    })?;
    let clip = unsafe { self.composition.CreateRectangleClip() }
      .map_err(|error| format!("The Windows preview pane clip could not be created: {error}"))?;
    (|| -> windows::core::Result<()> {
      unsafe {
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
          .AddVisual(&visual, true, Some(below.unwrap_or(&self.backdrop.visual)))
      }
    })()
    .map_err(|error| format!("The Windows preview pane could not be attached: {error}"))?;
    let surface = VisualSurface::new(self.shared, &visual)?;
    unsafe { self.composition.Commit() }
      .map_err(|error| format!("The Windows preview pane could not be shown: {error}"))?;
    Ok(Pane {
      annotation_halo: None,
      annotation_typing: None,
      base_rect: PreviewSurfaceRect {
        height: 0.0,
        width: 0.0,
        x: 0.0,
        y: 0.0,
      },
      clip,
      clip_edges: (0, 0, 2, 2),
      content_size: (2, 2),
      display_size: (2, 2),
      last_camera: None,
      last_composition: None,
      settings: None,
      magnifier: None,
      parked: None,
      pending_geometry: false,
      position: (0, 0),
      scale: 1.0,
      scale_transform,
      selection_stale: false,
      seen: true,
      source: None,
      source_token: None,
      surface,
      visual,
    })
  }
}
