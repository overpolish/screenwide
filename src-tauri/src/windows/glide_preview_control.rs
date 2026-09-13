// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub fn initialize_glide_preview(app: &AppHandle) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  platform::initialize_glide_preview(&window)
}

pub fn hide_glide_preview(app: &AppHandle) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  #[cfg(target_os = "windows")]
  window.set_ignore_cursor_events(true)?;
  platform::hide(&window)
}

pub fn defer_hide_glide_preview(app: &AppHandle) {
  let main_app = app.clone();
  let busy = crate::glide::BusyLease::acquire();
  let _ = app.run_on_main_thread(move || {
    let _busy = busy;
    let _ = hide_glide_preview(&main_app);
  });
}

/// Fades the preview out instead of hiding it outright, then runs `completion`
/// once it is gone. Committed gestures dismiss this way; a cancelled one still
/// takes the instant `hide_glide_preview` path.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn fade_glide_preview(
  app: &AppHandle,
  completion: Box<dyn FnOnce() + Send>,
) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  platform::fade_glide_preview(&window, completion)
}

/// Centres the preview on the anchor the gesture began at, then keeps it wholly
/// inside that monitor's work area so it cannot land under the menu bar, the
/// notch or the Dock. The window is still hidden here, because the reveal waits
/// for a destination, so the two-step placement cannot flicker.
#[cfg(target_os = "macos")]
pub(crate) fn position_glide_preview(app: &AppHandle, x: f64, y: f64) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  let logical_size = window
    .outer_size()?
    .to_logical::<f64>(window.scale_factor()?);
  window.set_position(tauri::LogicalPosition::new(
    x - logical_size.width / 2.0,
    y - logical_size.height / 2.0,
  ))?;

  // Geometry rather than `current_monitor`, because the window is hidden here.
  let monitor = match monitor_with_most_overlap(app, &window)? {
    Some(monitor) => Some(monitor),
    None => app.primary_monitor()?,
  };
  let Some(monitor) = monitor else {
    return Ok(());
  };
  let position = window.outer_position()?;
  let size = window.outer_size()?;
  let work_area = monitor.work_area();
  let max_x = work_area.position.x + work_area.size.width.saturating_sub(size.width) as i32;
  let max_y = work_area.position.y + work_area.size.height.saturating_sub(size.height) as i32;
  let contained = PhysicalPosition::new(
    position
      .x
      .clamp(work_area.position.x, max_x.max(work_area.position.x)),
    position
      .y
      .clamp(work_area.position.y, max_y.max(work_area.position.y)),
  );

  if contained != position {
    window.set_position(contained)?;
  }

  Ok(())
}

pub fn show_glide_preview(app: &AppHandle, blocks_hover: bool) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  platform::show_glide(&window, 1.0, blocks_hover)
}
