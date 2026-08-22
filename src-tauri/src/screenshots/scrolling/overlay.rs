// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, LogicalPosition, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::screenshots::ScreenshotTarget;

pub(super) const LABEL: &str = "scrolling-capture-overlay";
const WIDTH: f64 = 260.0;
const HEIGHT: f64 = 200.0;

fn centred_origin(centre_x: f64, centre_y: f64) -> LogicalPosition<f64> {
  LogicalPosition::new(centre_x - WIDTH / 2.0, centre_y - HEIGHT / 2.0)
}

/// Where the overlay sits: centred on the region being captured.
///
/// A region is expressed in logical points relative to its monitor on both
/// platforms — the physical maths in `scroll_geometry` is what the pointer APIs
/// want, not what a window position wants — so converting the monitor origin to
/// logical points is the whole conversion.
fn position(app: &AppHandle, target: ScreenshotTarget) -> Result<LogicalPosition<f64>, String> {
  let ScreenshotTarget::Region { monitor_id, region } = target else {
    return Err("Scrolling capture requires a region".to_owned());
  };
  let (_, scale, monitor) = crate::capture_overlays::monitor_layout(app)?
    .into_iter()
    .find(|(candidate, _, _)| *candidate == monitor_id)
    .ok_or_else(|| "The selected monitor is no longer available".to_owned())?;
  let origin = monitor.position().to_logical::<f64>(scale);

  Ok(centred_origin(
    origin.x + region.position.x + region.size.width / 2.0,
    origin.y + region.position.y + region.size.height / 2.0,
  ))
}

/// `cancellable` rides in on the URL rather than an event: the first progress
/// event is emitted before this webview has finished loading its listener, so
/// anything sent that early is simply missed, and whether Escape was claimed is
/// fixed for the whole capture anyway.
pub(super) fn show(
  app: &AppHandle,
  target: ScreenshotTarget,
  cancellable: bool,
) -> Result<(), String> {
  close(app);
  let origin = position(app, target)?;
  let window = WebviewWindowBuilder::new(
    app,
    LABEL,
    WebviewUrl::App(
      format!(
        "/scrolling-capture-overlay?cancellable={}",
        u8::from(cancellable)
      )
      .into(),
    ),
  )
  .always_on_top(true)
  .decorations(false)
  .focused(false)
  .inner_size(WIDTH, HEIGHT)
  .position(origin.x, origin.y)
  .resizable(false)
  .shadow(false)
  .skip_taskbar(true)
  .transparent(true)
  .visible(false)
  .build()
  .map_err(|error| error.to_string())?;

  // Never focused: the capture scrolls whatever the user was reading, and
  // taking key status away from it would change what is on screen mid-capture.
  crate::windows::show(&window, false).map_err(|error| error.to_string())?;
  // Both invariants are asserted after showing, because `platform::show` turns
  // cursor events back on every time it runs and re-applies the persistent
  // capture-affinity preference.
  //
  // Click-through: the pointer is parked at the centre of the region driving
  // the scroll, which is exactly where this window sits. A hit-testing window
  // there would swallow every scroll event instead of the page beneath it.
  window
    .set_ignore_cursor_events(true)
    .map_err(|error| error.to_string())?;
  crate::windows::exclude_from_capture(&window).map_err(|error| error.to_string())?;

  Ok(())
}

pub(super) fn close(app: &AppHandle) {
  let Some(window) = app.get_webview_window(LABEL) else {
    return;
  };
  #[cfg(target_os = "windows")]
  let _ = crate::windows::conceal_disposable_overlay(&window);
  let _ = window.close();
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;
