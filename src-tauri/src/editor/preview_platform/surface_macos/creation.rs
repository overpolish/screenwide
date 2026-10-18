// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making an editor window's surface: the views either side of the webview,
//! the layers and drawables, and the engine over them. AppKit's views are
//! made on the main thread, whichever thread asks.

use std::sync::Arc;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSAutoresizingMaskOptions, NSView, NSWindowOrderingMode};

use super::audio_ribbon::AudioRibbon;
use super::host::{light_appearance, Gpu, MacHost, Views};
use super::layers::Layers;
use super::main_thread::OnMain;
use super::picture_view::PictureView;
use super::registry::slot_for;
use super::view::EditorView;
use super::workspace_view::{surface_on, WorkspaceView};
use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn from_window(window: &WebviewWindow) -> Result<Self, String> {
    let key = window.ns_window().map_err(|error| error.to_string())? as usize;
    // The slot is handed out from under the registry's lock, since making a
    // surface waits on the main thread, which must not want that lock.
    let slot = slot_for(key)?;
    let inner = slot
      .get_or_init(|| create(window))
      .as_ref()
      .map_err(Clone::clone)?;
    Ok(Self {
      editor: WorkspaceEditor {
        inner: Arc::clone(inner),
      },
    })
  }
}

fn create(window: &WebviewWindow) -> Result<Arc<SurfaceInner>, String> {
  let host = window.ns_view().map_err(|error| error.to_string())? as usize;
  let webview = window.clone();
  dispatch2::run_on_main(move |mtm| build(mtm, host, webview))
}

fn build(
  mtm: MainThreadMarker,
  host: usize,
  webview: WebviewWindow,
) -> Result<Arc<SurfaceInner>, String> {
  // Safety: Tauri hands over the window's live content view.
  let host = unsafe { Retained::retain(host as *mut NSView) }
    .ok_or_else(|| "The editor window has no content view".to_owned())?;
  let shared = screenwide_gpu::shared()?;
  let layers = Layers::new();

  let below = PictureView::new(mtm, host.bounds());
  // A layer-hosting view: the layer is set before the view wants one.
  below.setLayer(Some(&layers.root));
  below.setWantsLayer(true);
  below.setAutoresizingMask(
    NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
  );
  let editor = EditorView::new(mtm);
  editor.setLayer(Some(&layers.chrome_root));
  editor.setWantsLayer(true);
  // The picture goes immediately below the webview, so its DOM controls
  // paint over it, and the input view immediately above it.
  let page = crate::app_windows::platform::webview_view(&host);
  host.addSubview_positioned_relativeTo(&below, NSWindowOrderingMode::Below, page.as_deref());
  host.addSubview_positioned_relativeTo(&editor, NSWindowOrderingMode::Above, page.as_deref());

  let scale = host
    .window()
    .map_or(2.0, |window| window.backingScaleFactor());
  layers.set_scale(scale);
  // Safety: on the main thread, as wgpu configures each layer it is given.
  let workspace = unsafe { WorkspaceView::new(shared, Retained::clone(&layers.workspace)) }?;
  let chrome = unsafe { surface_on(shared, &layers.chrome) }?;
  let ribbon = unsafe { AudioRibbon::new(shared, Retained::clone(&layers.ribbon)) }?;
  let gpu = Gpu {
    layers,
    workspace,
    selection: std::sync::Mutex::new(ChromeRenderer::new(shared, chrome)),
    audio_ribbon: std::sync::Mutex::new(ribbon),
  };
  let views = OnMain::new(
    Views {
      host,
      editor: Retained::clone(&editor),
    },
    mtm,
  );
  let light = light_appearance(mtm);
  let engine = WorkspaceEditor::new(Box::new(MacHost::new(views, gpu, webview, scale, light)));
  editor.attach(&engine.inner);
  Ok(engine.inner)
}
