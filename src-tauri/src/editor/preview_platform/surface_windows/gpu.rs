// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Gpu {
  pub(super) fn new(host: HWND, editor: HWND) -> Result<Self, String> {
    let shared = screenwide_gpu::shared()?;
    let d3d11 = screenwide_gpu::d3d11()?;
    // Every visual's content is a swap chain, so the composition device needs
    // no Direct3D device of its own.
    let composition: IDCompositionDevice =
      unsafe { DCompositionCreateDevice(None::<&IDXGIDevice>) }
        .map_err(|error| format!("DirectComposition could not open for the preview: {error}"))?;
    // The non-topmost target is the critical Windows equivalent of inserting
    // the macOS preview view immediately below WKWebView: WebView2 remains a child
    // window above this GPU visual tree, so its DOM OSCs paint last.
    let target = unsafe { composition.CreateTargetForHwnd(host, false) }
      .map_err(|error| format!("The Windows preview compositor could not attach: {error}"))?;
    let root = unsafe { composition.CreateVisual() }
      .map_err(|error| format!("The Windows preview visual tree could not be created: {error}"))?;
    unsafe { target.SetRoot(&root) }
      .map_err(|error| format!("The Windows preview visual tree could not be attached: {error}"))?;
    let backdrop = Backdrop::new(shared, &composition, &root)?;
    let workspace = WorkspaceView::new(shared, &composition, &root, &backdrop.visual)?;
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
    let compositor = compositor::Compositor::new(shared, cursor_artwork::native_cursors()?)?;
    // Drawn as it is edited: a set of kinds new to it is compiled on another
    // thread while the frames go on with a variant that holds it.
    compositor.compile_annotation_kinds_in_background();
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
      selection: Mutex::new(selection),
      shared,
      _editor_target: editor_target,
      _target: target,
      workspace,
    })
  }
}
