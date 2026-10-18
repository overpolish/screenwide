// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the region overlay asks of the platform it runs on: its windows and
//! the drawables in them, timers, the pointer and keyboard, and the type its
//! labels are set in. The scene, the desktop peers, the OCR chrome, the
//! Ruler and every input decision are the engine's, the same everywhere.

use screenwide_gpu::surface::Frame;
use screenwide_osc::geometry::Rect;
use tauri::WebviewWindow;

use super::input::CursorShape;

/// A platform window the overlay draws in or takes input through: an HWND on
/// Windows, an `NSView` or `NSPanel` on macOS. Only the host gives it
/// meaning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct WindowId(pub(crate) isize);

/// The overlay that holds the keyboard while it is up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyboardOwner {
  Ruler,
  TextRecognition,
}

/// A point in physical screen pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenPoint {
  pub x: i32,
  pub y: i32,
}

/// The drawable a surface renders its frames into.
pub(crate) trait RegionDrawable: Send + Sync {
  fn resize(&mut self, gpu: &screenwide_gpu::Gpu, size: (u32, u32));
  fn acquire(&self, gpu: &screenwide_gpu::Gpu) -> Result<Frame, String>;
}

/// Sets text in the overlay's label faces: Inter for chrome, Roboto Mono for
/// readouts, at one font size and raster scale.
pub(crate) trait TextRaster {
  /// A string's cell extent in raster pixels.
  fn measure(&self, text: &str) -> Option<(i32, i32)>;
  /// Draws `runs` - text and a raster-pixel origin - and returns their
  /// coverage, one byte a pixel, at `output` pixels.
  fn coverage(&self, output: (i32, i32), runs: &[(String, i32, i32)]) -> Option<Vec<u8>>;
}

pub(crate) trait RegionHost: Send + Sync + 'static {
  /// The window a webview draws in, which the anchor overlay covers.
  fn webview_window(&self, window: &WebviewWindow) -> Option<WindowId>;
  /// Makes the anchor overlay over `host` when `peer` is `None`, or a peer
  /// covering `bounds` on another display, capturable or kept out of
  /// captures. Runs on the thread that owns `webview`'s windows.
  fn create_window(
    &self,
    webview: &WebviewWindow,
    host: WindowId,
    peer: Option<(Rect, bool)>,
  ) -> Result<WindowId, String>;
  fn create_drawable(
    &self,
    gpu: &screenwide_gpu::Gpu,
    window: WindowId,
  ) -> Result<Box<dyn RegionDrawable>, String>;
  fn destroy_window(&self, window: WindowId);

  /// Fires `timer` on `window` every `interval_ms` until killed, through the
  /// engine's `on_timer`.
  fn set_timer(&self, window: WindowId, timer: usize, interval_ms: u32);
  fn kill_timer(&self, window: WindowId, timer: usize);
  /// Asks for the cursor to be set again from the window's own thread.
  fn refresh_cursor_later(&self, window: WindowId);

  /// Gives the keyboard to `window`. Answers whether it took it.
  fn focus(&self, window: WindowId) -> bool;
  fn set_cursor(&self, shape: CursorShape);
  fn release_pointer(&self);
  fn cursor_position(&self) -> Option<ScreenPoint>;
  fn light_mode(&self) -> bool;

  fn show(&self, window: WindowId, visible: bool);
  /// Places a peer over `bounds`, physical pixels, above every other window.
  fn place_peer(&self, window: WindowId, bounds: Rect);
  /// Raises the anchor overlay above the webview it covers, sized to `size`
  /// physical pixels.
  fn raise(&self, window: WindowId, size: (u32, u32));
  /// Physical pixels per logical point for `window`.
  fn scale(&self, window: WindowId) -> f64;
  /// The client area in physical pixels.
  fn client_size(&self, window: WindowId) -> (u32, u32);
  /// The window's frame on screen, physical pixels.
  fn screen_rect(&self, window: WindowId) -> Option<Rect>;
  fn screen_to_client(&self, window: WindowId, point: ScreenPoint) -> (f64, f64);
  fn set_pointer_passthrough(&self, window: WindowId, is_root: bool, passthrough: bool) -> bool;
  fn set_capture_affinity(&self, window: WindowId, capturable: bool) -> bool;
  /// A raster context for one face at `font_size` points and `raster_scale`.
  fn text_context(
    &self,
    font_size: f64,
    raster_scale: f64,
    mono: bool,
  ) -> Option<Box<dyn TextRaster>>;

  /// Routes the keyboard to `target` for `owner`'s overlay, whatever has
  /// focus, until released.
  fn capture_keyboard(&self, target: WindowId, owner: KeyboardOwner) -> Result<(), String>;
  fn release_keyboard(&self, owner: KeyboardOwner);
  /// Whether the caller is on the thread that owns `window`'s views, where
  /// waiting on that thread would deadlock.
  fn on_owning_thread(&self, window: &WebviewWindow) -> bool;
  /// Holds the cursor to the overlay for a Quick Screenshot, where the
  /// platform would otherwise let windows beneath set it.
  fn acquire_quick_screenshot_cursor(&self, app: &tauri::AppHandle) -> Result<(), String>;
  fn release_quick_screenshot_cursor(&self, app: &tauri::AppHandle) -> Result<(), String>;
}

#[cfg(target_os = "windows")]
pub(crate) fn host() -> &'static dyn RegionHost {
  &crate::app_windows::screenshot_region::native_osc_windows::WindowsRegionHost
}

#[cfg(target_os = "macos")]
pub(crate) fn host() -> &'static dyn RegionHost {
  &crate::app_windows::screenshot_region::native_osc_macos::MacRegionHost
}
