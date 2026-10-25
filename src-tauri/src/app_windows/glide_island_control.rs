// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub fn initialize_glide_island(app: &AppHandle) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  platform::initialize_glide_window(&window)
}

pub fn hide_glide_island(app: &AppHandle) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  #[cfg(target_os = "windows")]
  window.set_ignore_cursor_events(true)?;
  platform::hide(&window)
}

pub fn defer_hide_glide_island(app: &AppHandle) {
  let main_app = app.clone();
  let busy = crate::glide::BusyLease::acquire();
  let _ = app.run_on_main_thread(move || {
    let _busy = busy;
    let _ = hide_glide_island(&main_app);
  });
}

/// Fades the preview out instead of hiding it outright, then runs `completion`
/// once it is gone. Committed gestures dismiss this way; a cancelled one still
/// takes the instant `hide_glide_island` path.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn fade_glide_island(
  app: &AppHandle,
  completion: Box<dyn FnOnce() + Send>,
) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  platform::fade_glide_window(&window, completion)
}

/// Centres the preview on the anchor the gesture began at, keeping the island
/// it holds on that anchor's screen; the transparent shadow room round it may
/// reach past the edge. The island may cover the menu bar: a gesture on a title
/// bar just under it would otherwise find the island pushed down off its
/// anchor. The window is still hidden here, because the reveal waits for a
/// destination.
///
/// Tao applies a new position asynchronously, on the main queue, so the
/// window's frame read back here would still be the last session's. The
/// placement is worked out from the anchor alone and set once.
#[cfg(target_os = "macos")]
pub(crate) fn position_glide_island(app: &AppHandle, x: f64, y: f64) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  let size = window
    .outer_size()?
    .to_logical::<f64>(window.scale_factor()?);
  let size = (size.width, size.height);
  let centred = (x - size.0 / 2.0, y - size.1 / 2.0);

  let monitors = app.available_monitors()?;
  let screen = monitors
    .iter()
    .map(|monitor| {
      let scale = monitor.scale_factor();
      let origin = monitor.position().to_logical::<f64>(scale);
      let extent = monitor.size().to_logical::<f64>(scale);
      ((origin.x, origin.y), (extent.width, extent.height))
    })
    .find(|((left, top), (width, height))| {
      x >= *left && x < left + width && y >= *top && y < top + height
    });
  let (left, top) = match screen {
    Some((origin, extent)) => {
      crate::glide::contained_origin(centred, size, crate::glide::SHADOW_ROOM, origin, extent)
    }
    None => centred,
  };
  window.set_position(tauri::LogicalPosition::new(left, top))
}

pub fn show_glide_island(app: &AppHandle, blocks_hover: bool) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::Glide.as_str())
    .ok_or(tauri::Error::WindowNotFound)?;
  platform::show_glide(&window, 1.0, blocks_hover)
}
