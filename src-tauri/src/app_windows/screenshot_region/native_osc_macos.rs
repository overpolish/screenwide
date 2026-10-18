// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The region OSC's macOS host: the anchor view over the region webview and
//! the panels on other displays, each with a Metal layer, a local event
//! monitor feeding the engine, run-loop timers, AppKit cursors and Core Text
//! labels. Everything the overlay decides lives in `region_osc`; this is the
//! twin of `native_osc_windows`.

mod events;
mod geometry;
mod text;
mod timers;
mod windows;

use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSCursor, NSEvent, NSView};
use screenwide_gpu::surface::{Frame, Surface};
use screenwide_osc::geometry::Rect;
use tauri::WebviewWindow;

use super::region_osc::host::{
  KeyboardOwner, RegionDrawable, RegionHost, ScreenPoint, TextRaster, WindowId,
};
use super::region_osc::input::{self as engine, CursorShape};

pub(crate) struct MacRegionHost;

/// Whether the system draws its chrome light, read on the main thread.
static LIGHT: AtomicBool = AtomicBool::new(false);

/// Runs `work` on the main thread's next turn, or now when already there.
fn on_main(work: impl FnOnce(MainThreadMarker) + Send + 'static) {
  if let Some(mtm) = MainThreadMarker::new() {
    work(mtm);
  } else {
    dispatch2::DispatchQueue::main().exec_async(move || {
      // Safety: the main queue runs its blocks on the main thread.
      work(unsafe { MainThreadMarker::new_unchecked() });
    });
  }
}

/// A Metal layer's drawable, configured on the main thread when made.
struct MetalDrawable(Surface);

// wgpu's Metal surface may be resized and acquired from any thread.
unsafe impl Send for MetalDrawable {}
unsafe impl Sync for MetalDrawable {}

impl RegionDrawable for MetalDrawable {
  fn resize(&mut self, gpu: &screenwide_gpu::Gpu, size: (u32, u32)) {
    self.0.resize(gpu, size);
  }

  fn acquire(&self, gpu: &screenwide_gpu::Gpu) -> Result<Frame, String> {
    self.0.acquire(gpu)
  }
}

fn refresh_light(mtm: MainThreadMarker) {
  use objc2_app_kit::{NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication};
  let appearance = NSApplication::sharedApplication(mtm).effectiveAppearance();
  // Safety: the names are AppKit's own constants.
  let names = unsafe {
    objc2_foundation::NSArray::from_slice(&[NSAppearanceNameAqua, NSAppearanceNameDarkAqua])
  };
  let light = appearance
    .bestMatchFromAppearancesWithNames(&names)
    .is_some_and(|name| &*name == unsafe { NSAppearanceNameAqua });
  LIGHT.store(light, Ordering::Relaxed);
}

fn cursor(shape: CursorShape) -> Option<Retained<NSCursor>> {
  unsafe extern "C" {
    /// The frame-resize cursor for `edges` (1 left, 2 right, 4 top, 8
    /// bottom), in `osc/src/gpu/macos/region_cursor_macos.m`.
    fn screenwide_region_resize_cursor(edges: u32) -> *mut NSCursor;
  }
  // Safety: the helper returns an autoreleased cursor or null.
  let resize = |edges: u32| unsafe { Retained::retain(screenwide_region_resize_cursor(edges)) };
  Some(match shape {
    CursorShape::None => return None,
    CursorShape::Crosshair => NSCursor::crosshairCursor(),
    CursorShape::Move => NSCursor::closedHandCursor(),
    CursorShape::ResizeHorizontal => resize(1)?,
    CursorShape::ResizeVertical => resize(4)?,
    CursorShape::ResizeNesw => resize(2 | 4)?,
    CursorShape::ResizeNwse => resize(1 | 4)?,
    CursorShape::Arrow => NSCursor::arrowCursor(),
    CursorShape::IBeam => NSCursor::IBeamCursor(),
    CursorShape::Hand => NSCursor::pointingHandCursor(),
  })
}

impl RegionHost for MacRegionHost {
  fn webview_window(&self, window: &WebviewWindow) -> Option<WindowId> {
    window.ns_view().ok().map(|view| WindowId(view as isize))
  }

  fn create_window(
    &self,
    _webview: &WebviewWindow,
    host: WindowId,
    peer: Option<(Rect, bool)>,
  ) -> Result<WindowId, String> {
    dispatch2::run_on_main(move |mtm| {
      // Safety: `host` is a content view Tauri handed over, alive while its
      // window is.
      let host = unsafe { Retained::retain(host.0 as *mut NSView) }
        .ok_or_else(|| "The region window has no content view".to_owned())?;
      geometry::refresh_screens(mtm);
      refresh_light(mtm);
      let window = match peer {
        None => windows::create_anchor(mtm, &host),
        Some((bounds, capturable)) => windows::create_peer(mtm, &host, bounds, capturable),
      };
      events::install();
      Ok(window)
    })
  }

  fn create_drawable(
    &self,
    gpu: &screenwide_gpu::Gpu,
    window: WindowId,
  ) -> Result<Box<dyn RegionDrawable>, String> {
    let surface = dispatch2::run_on_main(move |_| {
      windows::with(window, |overlay| {
        let layer = NonNull::from(&*overlay.layer).cast::<std::ffi::c_void>();
        // Safety: on the main thread, on a live layer the registry keeps.
        unsafe { Surface::on_metal_layer(gpu, layer) }
      })
      .unwrap_or_else(|| Err("The region overlay window is gone".to_owned()))
      .map(MetalDrawable)
    })?;
    Ok(Box::new(surface))
  }

  fn destroy_window(&self, window: WindowId) {
    on_main(move |_| {
      timers::kill_all(window);
      windows::destroy(window);
      events::uninstall_if_unused();
    });
  }

  fn set_timer(&self, window: WindowId, timer: usize, interval_ms: u32) {
    on_main(move |_| timers::set(window, timer, interval_ms));
  }

  fn kill_timer(&self, window: WindowId, timer: usize) {
    on_main(move |_| timers::kill(window, timer));
  }

  fn refresh_cursor_later(&self, window: WindowId) {
    dispatch2::DispatchQueue::main().exec_async(move || {
      engine::cursor_request(window);
    });
  }

  fn focus(&self, window: WindowId) -> bool {
    on_main(move |_| {
      windows::with(window, |overlay| {
        if let Some(host) = overlay.view.window() {
          host.makeKeyWindow();
        }
      });
    });
    true
  }

  fn set_cursor(&self, shape: CursorShape) {
    on_main(move |_| {
      if let Some(cursor) = cursor(shape) {
        cursor.set();
      }
    });
  }

  fn release_pointer(&self) {
    on_main(|_| events::release_pointer());
  }

  fn cursor_position(&self) -> Option<ScreenPoint> {
    Some(geometry::screen_point(NSEvent::mouseLocation()))
  }

  fn light_mode(&self) -> bool {
    LIGHT.load(Ordering::Relaxed)
  }

  fn show(&self, window: WindowId, visible: bool) {
    on_main(move |_| windows::show(window, visible));
  }

  fn place_peer(&self, window: WindowId, bounds: Rect) {
    on_main(move |_| windows::place_peer(window, bounds));
  }

  fn raise(&self, window: WindowId, _size: (u32, u32)) {
    on_main(move |_| windows::raise(window));
  }

  fn scale(&self, window: WindowId) -> f64 {
    geometry::placement(window.0).map_or(1.0, |placement| placement.scale)
  }

  fn client_size(&self, window: WindowId) -> (u32, u32) {
    geometry::placement(window.0).map_or((0, 0), |placement| {
      (
        placement.frame.size.width.round().max(0.0) as u32,
        placement.frame.size.height.round().max(0.0) as u32,
      )
    })
  }

  fn screen_rect(&self, window: WindowId) -> Option<Rect> {
    geometry::placement(window.0).map(|placement| placement.frame)
  }

  fn screen_to_client(&self, window: WindowId, point: ScreenPoint) -> (f64, f64) {
    let origin = geometry::placement(window.0).map_or((0.0, 0.0), |placement| {
      (placement.frame.origin.x, placement.frame.origin.y)
    });
    (f64::from(point.x) - origin.0, f64::from(point.y) - origin.1)
  }

  /// The anchor's pointer is decided by the engine's hit test; a peer lets
  /// the pointer through to other apps' windows while it passes.
  fn set_pointer_passthrough(&self, window: WindowId, is_root: bool, passthrough: bool) -> bool {
    if !is_root {
      on_main(move |_| {
        windows::with(window, |overlay| {
          if let Some(panel) = &overlay.panel {
            panel.setIgnoresMouseEvents(passthrough);
          }
        });
      });
    }
    true
  }

  fn set_capture_affinity(&self, window: WindowId, capturable: bool) -> bool {
    on_main(move |_| {
      windows::with(window, |overlay| {
        if let Some(panel) = &overlay.panel {
          panel.setSharingType(windows::sharing(capturable));
        }
      });
    });
    true
  }

  fn text_context(
    &self,
    font_size: f64,
    raster_scale: f64,
    mono: bool,
  ) -> Option<Box<dyn TextRaster>> {
    text::CoreTextRaster::new(font_size, raster_scale, mono).map(|text| Box::new(text) as _)
  }

  fn capture_keyboard(&self, target: WindowId, _owner: KeyboardOwner) -> Result<(), String> {
    on_main(move |_| events::KEYBOARD.with(|keyboard| keyboard.set(Some(target))));
    Ok(())
  }

  fn release_keyboard(&self, _owner: KeyboardOwner) {
    on_main(|_| events::KEYBOARD.with(|keyboard| keyboard.set(None)));
  }

  fn on_owning_thread(&self, _window: &WebviewWindow) -> bool {
    MainThreadMarker::new().is_some()
  }

  fn acquire_quick_screenshot_cursor(&self, app: &tauri::AppHandle) -> Result<(), String> {
    crate::osc::cursor_macos::acquire_quick_screenshot(app)
  }

  fn release_quick_screenshot_cursor(&self, app: &tauri::AppHandle) -> Result<(), String> {
    crate::osc::cursor_macos::release_quick_screenshot(app)
  }
}
