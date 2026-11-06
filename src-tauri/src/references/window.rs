// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Where a reference stands and the window it stands in.

use screenwide_capture::Region;
use screenwide_osc::geometry::Size;
use tauri::{AppHandle, LogicalSize, Position, WebviewUrl, WindowEvent};

#[cfg(target_os = "macos")]
use super::macos::attach;
#[cfg(target_os = "windows")]
use super::windows::attach;

use crate::screenshots::ScreenshotTarget;

/// Where a reference stands: over the area it was taken from, at that area's
/// size in points, which its window keeps on a display of another scale.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Placement {
  /// Where it stands as it opens: where it was taken, or where a dropped
  /// picture was let go. None centres it.
  pub origin: Option<Position>,
  /// Whether `origin` is where the still was taken, which Back to origin
  /// returns it to. A dropped picture has nowhere to return to.
  pub returns: bool,
  pub size: LogicalSize<f64>,
}

/// Where a reference of a still `pixels` in size, taken of `target` at
/// `scale` pixels to the point, stands.
pub(crate) fn placement(
  app: &AppHandle,
  target: ScreenshotTarget,
  (width, height): (u32, u32),
  scale: f64,
) -> Placement {
  let (monitor_id, region) = match target {
    ScreenshotTarget::Region { monitor_id, region }
    | ScreenshotTarget::DesktopRegion { monitor_id, region } => (monitor_id, region),
    ScreenshotTarget::Screen { .. } | ScreenshotTarget::Window { .. } => {
      return Placement {
        origin: None,
        returns: false,
        size: LogicalSize::new(f64::from(width) / scale, f64::from(height) / scale),
      }
    }
  };
  let origin = crate::monitor_topology::snapshot(app)
    .ok()
    .and_then(|displays| {
      displays
        .into_iter()
        .find(|display| display.id == monitor_id)
    })
    .map(|display| origin_on(&display, region));
  Placement {
    returns: origin.is_some(),
    origin,
    size: LogicalSize::new(region.size.width, region.size.height),
  }
}

/// `region`'s corner on the desktop. macOS lays its displays out in points,
/// as a window's logical position is given.
#[cfg(target_os = "macos")]
fn origin_on(display: &crate::monitor_topology::Display, region: Region) -> Position {
  Position::Logical(tauri::LogicalPosition::new(
    display.layout_position.0 + region.position.x,
    display.layout_position.1 + region.position.y,
  ))
}

/// `region`'s corner on the desktop. Windows lays its displays out in
/// physical pixels, each at its own scale, so the region's offset in points
/// is taken at its display's.
#[cfg(not(target_os = "macos"))]
fn origin_on(display: &crate::monitor_topology::Display, region: Region) -> Position {
  let scale = display.native.scale_factor();
  let corner = display.native.position();
  Position::Physical(tauri::PhysicalPosition::new(
    corner.x + (region.position.x * scale).round() as i32,
    corner.y + (region.position.y * scale).round() as i32,
  ))
}

/// Opens the window for the reference `label` showing `texture` at
/// `placement`, focused so its keys answer at once. The window is placed
/// round the picture as its native view attaches (`references::engine`).
/// Its picture is let go of as the window goes.
pub(super) fn build(
  app: &AppHandle,
  label: &str,
  placement: Placement,
  texture: wgpu::Texture,
) -> Result<(), String> {
  let window = crate::app_windows::webview_window(app, label, WebviewUrl::App("/reference".into()))
    .title(screenwide_i18n::t!("window-titles-reference"))
    .accept_first_mouse(true)
    .always_on_top(true)
    .decorations(false)
    .inner_size(placement.size.width, placement.size.height)
    .resizable(false)
    .shadow(false)
    .skip_taskbar(true)
    .transparent(true)
    .visible(false)
    .build()
    .map_err(|error| error.to_string())?;
  let forgotten = label.to_owned();
  window.on_window_event(move |event| {
    if matches!(event, WindowEvent::Destroyed) {
      super::forget(&forgotten);
    }
  });
  let size = Size {
    width: placement.size.width,
    height: placement.size.height,
  };
  let attaching = window.clone();
  super::engine::on_main(app, move || {
    attach(&attaching, &texture, size, placement.origin)
  })??;
  #[cfg(target_os = "windows")]
  crate::app_windows::apply_capture_policy(app).map_err(|error| error.to_string())?;
  window.show().map_err(|error| error.to_string())?;
  window.set_focus().map_err(|error| error.to_string())
}
