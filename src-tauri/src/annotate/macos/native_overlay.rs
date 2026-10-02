// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Rust side of the overlay: the displays it covers, and one wgpu surface
//! per host window on the `CAMetalLayer` native code gives it.
//!
//! Native code owns the windows, the layers and the events; this owns the
//! annotations and draws every frame through [`crate::annotate::overlay`],
//! pulling them each time, so there is no second copy of the document to
//! keep in step with [`super::super::live_clips`].

use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock, RwLock};

use objc2_app_kit::NSWindow;
use tauri::{AppHandle, WebviewWindow};

use crate::annotate::overlay::{Renderer, Underlays};
use crate::gpu::surface::{Frame, Surface};

/// One display the overlay covers: the capture display it is, where it
/// starts and how far it reaches in desktop points, and how many pixels one
/// point is.
#[derive(Clone, Copy)]
pub(in super::super) struct Display {
  pub id: u32,
  pub origin: (f64, f64),
  pub size: (f64, f64),
  pub scale: f64,
}

/// One host window's layer and the surface drawn into it.
struct Host {
  /// The host's view, which is how a surface is found again.
  view: *mut c_void,
  /// Which [`Display`] the annotations are mapped into.
  display: u32,
  surface: Surface,
  underlays: Underlays,
}

/// The surfaces on screen and the renderer they share. The renderer is made
/// by the first attach and dropped by the last detach: a session that is not
/// drawing holds no GPU resources.
#[derive(Default)]
struct Overlay {
  renderer: Option<Renderer>,
  hosts: Vec<Host>,
}

// The view pointers are only compared, and every surface is reached under
// this mutex from the main thread.
unsafe impl Send for Overlay {}

static OVERLAY: LazyLock<Mutex<Overlay>> = LazyLock::new(|| Mutex::new(Overlay::default()));
static DISPLAYS: LazyLock<RwLock<Vec<Display>>> = LazyLock::new(|| RwLock::new(Vec::new()));
/// The key callback is a bare C function pointer, so the handle a tool key
/// needs to write a setting cannot ride along with it.
static APP: OnceLock<AppHandle> = OnceLock::new();

unsafe extern "C" {
  fn screenwide_annotate_attach(view: *mut c_void) -> *mut c_void;
  fn screenwide_annotate_detach(view: *mut c_void);
  fn screenwide_annotate_install_input(
    pointer: extern "C" fn(u32, f64, f64),
    key: extern "C" fn(u16, u32) -> u32,
  );
  fn screenwide_annotate_teardown_input();
  fn screenwide_annotate_set_text_cursor(text: u32);
}

fn overlay() -> MutexGuard<'static, Overlay> {
  OVERLAY.lock().unwrap_or_else(|error| error.into_inner())
}

fn display(index: u32) -> Option<Display> {
  DISPLAYS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .get(index as usize)
    .copied()
}

/// Draws one host's display: its annotations at the display's own
/// resolution, over what its highlights recolour.
fn draw(renderer: &Renderer, host: &mut Host) -> Result<(), String> {
  let Some(display) = display(host.display) else {
    return Ok(());
  };
  let size = (
    (display.size.0 * display.scale).round() as u32,
    (display.size.1 * display.scale).round() as u32,
  );
  if size.0 == 0 || size.1 == 0 {
    return Ok(());
  }
  let gpu = renderer.gpu();
  host.surface.resize(gpu, size);
  let Frame::Ready(frame) = host.surface.acquire(gpu)? else {
    return Ok(());
  };
  renderer.draw_display(
    &frame.texture.create_view(&Default::default()),
    size,
    host.display,
    display.origin,
    display.scale,
    &mut host.underlays,
  )?;
  gpu.queue.present(frame);
  Ok(())
}

extern "C" fn pointer(phase: u32, x: f64, y: f64) {
  // A press on the picture is drawing, so the keyboard belongs to the canvas
  // again. It has to be said here: the monitor swallows this press, so AppKit
  // never sees it and a toolbar field holding key status would keep it.
  if phase == super::super::input::PHASE_DOWN {
    if let Some(app) = APP.get() {
      super::super::toolbar::return_keyboard(app);
    }
  }
  super::super::input::pointer(APP.get(), phase, x, y);
}

extern "C" fn key(key_code: u16, modifiers: u32) -> u32 {
  let Some(app) = APP.get() else {
    return 0;
  };
  u32::from(super::super::input::key(app, key_code, modifiers))
}

/// Names the displays the overlay is about to cover. Their order is the order
/// the host windows are created in, which is what `display` indexes.
pub(in super::super) fn set_displays(displays: Vec<Display>) {
  *DISPLAYS.write().unwrap_or_else(|error| error.into_inner()) = displays;
}

/// Gives one host window its layer and the surface drawn into it. Main
/// thread only.
pub(in super::super) fn attach(window: &WebviewWindow, display: u32) -> Result<(), String> {
  let view = window.ns_view().map_err(|error| error.to_string())?;
  let layer = NonNull::new(unsafe { screenwide_annotate_attach(view) })
    .ok_or_else(|| "The annotate overlay could not open a layer".to_owned())?;
  let attached = (|| {
    let mut overlay = overlay();
    let overlay = &mut *overlay;
    if overlay.renderer.is_none() {
      overlay.renderer = Some(Renderer::new()?);
    }
    let renderer = overlay
      .renderer
      .as_ref()
      .expect("the renderer was just made");
    // The layer is the one just added to `view`, which outlives the surface.
    let surface = unsafe { Surface::on_metal_layer(renderer.gpu(), layer) }?;
    overlay.hosts.push(Host {
      view,
      display,
      surface,
      underlays: Underlays::default(),
    });
    let host = overlay.hosts.last_mut().expect("the host was just added");
    draw(renderer, host)
  })();
  if attached.is_err() {
    detach(window);
  }
  attached
}

/// Takes one host window's layer away, and the renderer with it once the last
/// host is gone. Main thread only.
pub(in super::super) fn detach(window: &WebviewWindow) {
  let Ok(view) = window.ns_view() else {
    return;
  };
  {
    let mut overlay = overlay();
    overlay.hosts.retain(|host| host.view != view);
    if overlay.hosts.is_empty() {
      overlay.renderer = None;
    }
  }
  unsafe { screenwide_annotate_detach(view) };
}

/// Called by the native input after each press, drag and key it routes.
#[no_mangle]
extern "C" fn screenwide_annotate_redraw() {
  redraw();
}

/// Starts swallowing pointer and key events. Main thread only.
pub(in super::super) fn install_input(app: &AppHandle) {
  let _ = APP.set(app.clone());
  unsafe { screenwide_annotate_install_input(pointer, key) };
}

/// Stops swallowing input. The displays stay: annotations left on screen are still
/// drawn on them. Main thread only.
pub(in super::super) fn teardown_input() {
  unsafe { screenwide_annotate_teardown_input() };
}

/// Forgets the displays, once nothing is drawn on them any more.
pub(in super::super) fn forget_displays() {
  set_displays(Vec::new());
  super::super::highlight::forget();
}

/// The display under a point in global desktop points, and its place in the
/// overlay's order.
pub(in super::super) fn display_at(x: f64, y: f64) -> Option<(usize, Display)> {
  DISPLAYS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .iter()
    .copied()
    .enumerate()
    .find(|(_, display)| {
      (display.origin.0..display.origin.0 + display.size.0).contains(&x)
        && (display.origin.1..display.origin.1 + display.size.1).contains(&y)
    })
}

/// Redraws every display from any thread.
pub(in super::super) fn request_redraw(app: &AppHandle) {
  let _ = app.run_on_main_thread(redraw);
}

/// Whether the pointer over the canvas is the I-beam. Main thread only.
pub(in super::super) fn set_text_cursor(text: bool) {
  unsafe { screenwide_annotate_set_text_cursor(u32::from(text)) };
}

/// Redraws every attached display. Main thread only.
pub(in super::super) fn redraw() {
  let mut overlay = overlay();
  let overlay = &mut *overlay;
  let Some(renderer) = overlay.renderer.as_ref() else {
    return;
  };
  for host in &mut overlay.hosts {
    if let Err(error) = draw(renderer, host) {
      eprintln!("The annotate overlay could not draw a display: {error}");
    }
  }
}

/// Puts a peer host on screen without asking for keyboard focus: only the
/// anchor window is made key, and `makeKeyAndOrderFront:` on the others would
/// take it straight back off it. Main thread only.
pub(in super::super) fn order_front(window: &WebviewWindow) {
  if let Ok(raw) = window.ns_window() {
    let native: &NSWindow = unsafe { &*raw.cast() };
    native.orderFrontRegardless();
  }
}

/// Lets everything under a host through while its annotations stay drawn: the
/// display-only state annotations kept after exiting are shown in. Main thread only.
pub(in super::super) fn set_click_through(window: &WebviewWindow, through: bool) {
  if let Ok(raw) = window.ns_window() {
    let native: &NSWindow = unsafe { &*raw.cast() };
    native.setIgnoresMouseEvents(through);
  }
}
