// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "macos")]
#[path = "platform/panels_macos.rs"]
mod panels_macos;
#[cfg(target_os = "windows")]
#[path = "platform/windows_api.rs"]
mod windows_api;
#[cfg(target_os = "windows")]
pub use windows_api::{
  hide, initialize_annotate_toolbar, initialize_editor, initialize_recording_bar,
  initialize_recording_dock, initialize_recording_source_selector, initialize_region_selector,
  initialize_standalone_listbox, initialize_tooltip, is_visible, prepare_to_show,
  raise_without_activation, restore_recording_level, set_capture_affinity, set_normal_level,
  set_opacity, set_owner, set_pointer_passthrough, show,
};

#[cfg(target_os = "macos")]
use panels_macos::*;

#[cfg(target_os = "macos")]
use objc2_foundation::{NSPoint, NSRect, NSSize};

use tauri::{LogicalPosition, LogicalSize, WebviewWindow};

#[cfg(target_os = "macos")]
#[path = "platform/capture_exclusion_macos.rs"]
mod capture_exclusion_macos;
#[cfg(target_os = "windows")]
#[path = "platform/composition.rs"]
mod composition;
#[path = "platform/glide_preview.rs"]
mod glide_preview;
#[cfg(target_os = "macos")]
pub use capture_exclusion_macos::exclude_from_capture;
#[cfg(target_os = "macos")]
mod presentation_macos;
#[cfg(target_os = "macos")]
pub use presentation_macos::{
  hide, raise_without_activation, restore_nonactivating_overlay, show, show_interactive_overlay,
};

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use glide_preview::fade_out as fade_glide_preview;
pub use glide_preview::{initialize_glide_preview, show_glide};

#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri::Manager;

#[cfg(target_os = "macos")]
use tauri_nspanel::{
  tauri_panel, CollectionBehavior, ManagerExt as PanelManagerExt, PanelHandle, PanelLevel,
  StyleMask, WebviewWindowExt,
};

/// Applies the complete panel frame in one AppKit operation. Keeping position
/// and size atomic prevents WindowServer from presenting an intermediate frame
/// while an above-anchored panel changes height.
#[cfg(target_os = "macos")]
pub fn set_frame(
  window: &WebviewWindow,
  position: LogicalPosition<f64>,
  size: LogicalSize<f64>,
) -> tauri::Result<()> {
  let panel = ensure_recording_panel(window)?;
  let main_display_height = CGDisplay::main().pixels_high() as f64;
  let frame = NSRect::new(
    NSPoint::new(position.x, main_display_height - position.y - size.height),
    NSSize::new(size.width, size.height),
  );
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || {
    panel.as_panel().setFrame_display(frame, true);
  })
}

#[cfg(not(target_os = "macos"))]
pub fn set_frame(
  window: &WebviewWindow,
  position: LogicalPosition<f64>,
  size: LogicalSize<f64>,
) -> tauri::Result<()> {
  window.set_size(size)?;
  window.set_position(position)
}

/// The editor window is an ordinary focusable window, so it gets none of the
/// panel treatment - only the capture exclusion, so that taking a screenshot
/// while it is open never pictures it. On macOS every window this process owns
/// is already excluded by owning-process, so there is nothing to do.
#[cfg(target_os = "macos")]
pub fn initialize_editor(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

/// The tooltip's panel. It floats just above ordinary windows, so it clears
/// the editor it describes without reaching the recording overlays, and it
/// refuses key status outright: a tooltip appearing must never take the
/// keyboard away from the control the pointer is over.
#[cfg(target_os = "macos")]
pub fn initialize_tooltip(window: &WebviewWindow) -> tauri::Result<()> {
  configure_panel::<RecordingDockPanel>(window, PanelLevel::Floating.value() as i32)
}

/// The annotate toolbar's panel. It sits one level above the overlay's hosts,
/// so nothing can be drawn over the controls, and refuses key status outright:
/// the anchor host owns the keyboard, which is what keeps the keys that draw
/// working while the toolbar is used.
#[cfg(target_os = "macos")]
pub fn initialize_annotate_toolbar(window: &WebviewWindow) -> tauri::Result<()> {
  configure_panel::<RecordingDockPanel>(
    window,
    crate::capture_overlays::FOREGROUND_LEVEL as i32 + 1,
  )
}

#[cfg(target_os = "macos")]
mod panel_properties_macos;
#[cfg(target_os = "macos")]
pub use panel_properties_macos::{
  release_key_focus, restore_recording_level, set_above_capture_overlays, set_normal_level,
  set_opacity,
};

/// Every window this app floats over the desktop is an overlay: always on top,
/// and off the taskbar. Its capture affinity follows the user's persistent
/// "record Screenwide windows" preference.
#[cfg(target_os = "windows")]
fn initialize_overlay(window: &WebviewWindow) -> tauri::Result<()> {
  window.set_always_on_top(true)?;
  window.set_skip_taskbar(true)?;
  disable_show_transitions(window)?;
  initialize_capture_affinity(window)
}

#[cfg(target_os = "windows")]
pub(crate) fn initialize_capture_overlay(window: &WebviewWindow) -> tauri::Result<()> {
  // Dynamic capture hosts may call `set_level` again during a screenshot
  // handoff. Reapply the idempotent native policy without the startup-only
  // hide choreography used by predefined windows.
  window.set_always_on_top(true)?;
  window.set_skip_taskbar(true)?;
  disable_show_transitions(window)?;
  let record_screenwide_windows =
    crate::settings::current(window.app_handle()).record_screenwide_windows;
  set_capture_affinity(window, record_screenwide_windows)
}

/// DWM plays a scale-and-fade transition on `ShowWindow` for top-level
/// windows; the AppKit recording panels order in and out instantly. Overlays
/// must match the panels, most visibly the monitor-sized region selector.
#[cfg(target_os = "windows")]
fn disable_show_transitions(window: &WebviewWindow) -> tauri::Result<()> {
  use windows::{
    core::BOOL,
    Win32::{
      Foundation::HWND,
      Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED},
    },
  };

  let hwnd = HWND(window.hwnd()?.0);
  let disabled = BOOL(1);
  unsafe {
    DwmSetWindowAttribute(
      hwnd,
      DWMWA_TRANSITIONS_FORCEDISABLED,
      (&raw const disabled).cast(),
      std::mem::size_of::<BOOL>() as u32,
    )
  }
  .map_err(std::io::Error::other)?;
  Ok(())
}

/// Asks DWM to round the window's corners, as it does for every framed
/// window on Windows 11 but not for an undecorated one. The floating panels
/// (recording bar and dock, pop-up lists, tooltips) are Fluent flyouts, which
/// carry the 8px overlay corner; the DWM corner is the only one that clips
/// the window's own material and shadow.
#[cfg(target_os = "windows")]
pub(crate) fn round_corners(window: &WebviewWindow) -> tauri::Result<()> {
  use windows::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND},
  };

  let hwnd = HWND(window.hwnd()?.0);
  let preference = DWMWCP_ROUND;
  unsafe {
    DwmSetWindowAttribute(
      hwnd,
      DWMWA_WINDOW_CORNER_PREFERENCE,
      (&raw const preference).cast(),
      std::mem::size_of_val(&preference) as u32,
    )
  }
  .map_err(std::io::Error::other)?;
  Ok(())
}

#[cfg(target_os = "macos")]
pub fn prepare_to_show(window: &WebviewWindow) -> tauri::Result<()> {
  enable_inactive_webview_hover(window)?;
  super::webview_visibility::show_webview(window)
}

#[cfg(target_os = "windows")]
fn initialize_capture_affinity(window: &WebviewWindow) -> tauri::Result<()> {
  let record_screenwide_windows =
    crate::settings::current(window.app_handle()).record_screenwide_windows;
  // Tauri can transiently report configured-hidden windows as visible while
  // WebView2 is creating their native surfaces. Changing display affinity in
  // that interval orders those unpainted surfaces onscreen. Hide on both sides
  // of the native call so startup never exposes a blank window shell.
  window.hide()?;
  set_capture_affinity(window, record_screenwide_windows)?;
  window.hide()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn initialize_recording_bar(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn initialize_recording_source_selector(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn initialize_region_selector(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn initialize_standalone_listbox(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn initialize_recording_dock(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn initialize_tooltip(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn initialize_editor(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn hide(window: &WebviewWindow) -> tauri::Result<()> {
  window.hide()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn show(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  set_opacity(window, opacity)?;
  window.show()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn prepare_to_show(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn set_opacity(_window: &WebviewWindow, _opacity: f64) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn restore_recording_level(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn raise_without_activation(_window: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}
