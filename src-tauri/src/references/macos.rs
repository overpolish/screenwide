// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! A reference on macOS: a view over its webview carrying two Metal layers,
//! the picture's and the chrome's, which the engine draws through the shared
//! OSC renderer, and takes the pointer over the picture. Desktop lengths are
//! in points from the primary display's top left, as Tauri gives a window's
//! logical position.

mod desktop;
mod host;
mod view;

use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2::MainThreadMarker;
use objc2_app_kit::{
  NSMainMenuWindowLevel, NSView, NSWindow, NSWindowDidChangeOcclusionStateNotification,
  NSWindowOrderingMode,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSRect};
use objc2_quartz_core::CAMetalLayer;
use screenwide_osc::geometry::Size;
use tauri::{Manager, Position, WebviewWindow};

use view::{metal_layer, surface_on, ReferenceView};

pub(crate) struct MacHost {
  mtm: MainThreadMarker,
  window: Retained<NSWindow>,
  /// The window's content view, which the view is laid in over the page.
  content: Retained<NSView>,
  view: Retained<ReferenceView>,
  picture_layer: Retained<CAMetalLayer>,
  chrome_layer: Retained<CAMetalLayer>,
  /// Watches for the window coming on screen.
  on_screen: Retained<ProtocolObject<dyn NSObjectProtocol>>,
}

/// Gives the reference's `window` its view and layers, showing `texture`
/// at `size` with its corner at `origin`, or centred on the main display.
/// Runs on the main thread.
pub(super) fn attach(
  window: &WebviewWindow,
  texture: &wgpu::Texture,
  size: Size,
  origin: Option<Position>,
) -> Result<(), String> {
  let mtm = MainThreadMarker::new().ok_or("A reference's view is made on the main thread")?;
  let gpu = screenwide_gpu::shared()?;
  let ns_window = window.ns_window().map_err(|error| error.to_string())?;
  let content = window.ns_view().map_err(|error| error.to_string())?;
  // Safety: Tauri hands out the live window and its content view.
  let (ns_window, content) = unsafe {
    (
      Retained::retain(ns_window.cast::<NSWindow>()),
      Retained::retain(content.cast::<NSView>()),
    )
  };
  let (Some(ns_window), Some(content)) = (ns_window, content) else {
    return Err("The reference's window has no view".to_owned());
  };
  // At the menu bar's level AppKit no longer keeps the window's top under
  // the menu bar, so a picture of the top of the screen can lie over it,
  // and the room for shadows above the picture never holds it down. Below
  // that level every frame set is held under it.
  ns_window.setLevel(NSMainMenuWindowLevel);
  let scale = ns_window.backingScaleFactor();
  let view = ReferenceView::new(mtm, window.label(), NSRect::ZERO);
  view.setWantsLayer(true);
  let root = view.layer().ok_or("The reference's view has no layer")?;
  let picture_layer = metal_layer(scale);
  let chrome_layer = metal_layer(scale);
  root.addSublayer(&picture_layer);
  root.addSublayer(&chrome_layer);
  let page = crate::app_windows::platform::webview_view(&content);
  content.addSubview_positioned_relativeTo(&view, NSWindowOrderingMode::Above, page.as_deref());
  // Safety: on the main thread, on layers the view keeps for as long as the
  // surfaces live.
  let surfaces = unsafe {
    (
      surface_on(&gpu, &picture_layer)?,
      surface_on(&gpu, &chrome_layer)?,
    )
  };
  // wgpu hands out no frame while the window is not on screen, and it is
  // not yet: the window shows after this, and macOS says so a moment later.
  // The reference is drawn again then, or it stays empty until something
  // else draws it.
  let label = window.label().to_owned();
  let shown = RcBlock::new(move |_: NonNull<NSNotification>| super::engine::redraw(&label));
  // Safety: the name is AppKit's own constant, and the block lives as long
  // as the observer, which is removed as the reference detaches.
  let on_screen = unsafe {
    NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
      Some(NSWindowDidChangeOcclusionStateNotification),
      Some(&ns_window),
      None,
      &shown,
    )
  };
  let host = MacHost {
    mtm,
    window: ns_window,
    content,
    view,
    picture_layer,
    chrome_layer,
    on_screen,
  };
  super::engine::attach(
    window.app_handle(),
    window.label(),
    host,
    surfaces,
    texture,
    size,
    origin,
  )
}
