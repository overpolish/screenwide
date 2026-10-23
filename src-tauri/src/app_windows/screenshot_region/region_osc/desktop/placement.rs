// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the host window sits: over the anchor display, in desktop mode.

use tauri::WebviewWindow;

use super::MonitorProbe;
use screenwide_osc::geometry::Rect;

/// The desktop compositor owns the host window's placement, exactly as the
/// macOS `rebuild_surfaces` sets the parent frame to the anchor screen:
/// `show_region_selector` deliberately skips sizing in desktop mode.
pub(super) fn place_over_anchor(window: &WebviewWindow, anchor: MonitorProbe) {
  let target = Rect::from_xywh(
    anchor.origin.x,
    anchor.origin.y,
    anchor.size.width,
    anchor.size.height,
  );
  if host_rect(window) == Some(target) {
    return;
  }
  // macOS places windows in points, and converts a physical position through
  // the window's current scale rather than the anchor's: moved from a 2x
  // display onto a 1x one, the window would land at half the anchor's size.
  // Points from the anchor's own scale place it exactly. Windows places
  // windows in physical pixels.
  #[cfg(target_os = "macos")]
  let placed = {
    let scale = anchor.scale.max(0.1);
    window
      .set_position(tauri::LogicalPosition::new(
        anchor.origin.x / scale,
        anchor.origin.y / scale,
      ))
      .and_then(|()| {
        window.set_size(tauri::LogicalSize::new(
          anchor.size.width / scale,
          anchor.size.height / scale,
        ))
      })
  };
  #[cfg(not(target_os = "macos"))]
  let placed = window
    .set_position(tauri::PhysicalPosition::new(
      anchor.origin.x as i32,
      anchor.origin.y as i32,
    ))
    .and_then(|()| {
      window.set_size(tauri::PhysicalSize::new(
        anchor.size.width as u32,
        anchor.size.height as u32,
      ))
    });
  if placed.is_err() {
    eprintln!("The region OSC could not cover the anchor monitor");
  }
}

pub(super) fn host_rect(window: &WebviewWindow) -> Option<Rect> {
  let position = window.outer_position().ok()?;
  let size = window.outer_size().ok()?;
  Some(Rect::from_xywh(
    f64::from(position.x),
    f64::from(position.y),
    f64::from(size.width),
    f64::from(size.height),
  ))
}
