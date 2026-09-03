// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
  desktop_capture::{self, DesktopDisplay, OutputLimits},
  screenshots::{
    desktop as still_composition, physical_capture_rect, CapturedImage, ScreenshotTarget,
  },
};

fn monitor(monitor_id: u32) -> Result<xcap::Monitor, String> {
  xcap::Monitor::all()
    .map_err(|error| error.to_string())?
    .into_iter()
    .find(|monitor| monitor.id().ok() == Some(monitor_id))
    .ok_or_else(|| "The selected monitor is no longer available".to_owned())
}

fn captured(image: image::RgbaImage) -> CapturedImage {
  CapturedImage {
    width: image.width(),
    height: image.height(),
    rgba: image.into_raw(),
  }
}

fn logical_display(
  id: u32,
  x: i32,
  y: i32,
  width: u32,
  height: u32,
  scale: f64,
) -> Result<DesktopDisplay, String> {
  if !scale.is_finite() || scale <= 0.0 {
    return Err("Windows returned an invalid monitor scale".to_owned());
  }
  Ok(DesktopDisplay {
    id,
    x: f64::from(x) / scale,
    y: f64::from(y) / scale,
    width: f64::from(width) / scale,
    height: f64::from(height) / scale,
    scale,
  })
}

fn desktop_layout(monitors: &[xcap::Monitor]) -> Result<Vec<DesktopDisplay>, String> {
  monitors
    .iter()
    .map(|monitor| {
      let scale = f64::from(monitor.scale_factor().map_err(|error| error.to_string())?);
      // xcap exposes Windows' virtual-screen geometry in device pixels while
      // the native Region controller emits anchor-local logical points.
      // Dividing every display by its own scale matches the OSC desktop plane.
      logical_display(
        monitor.id().map_err(|error| error.to_string())?,
        monitor.x().map_err(|error| error.to_string())?,
        monitor.y().map_err(|error| error.to_string())?,
        monitor.width().map_err(|error| error.to_string())?,
        monitor.height().map_err(|error| error.to_string())?,
        scale,
      )
    })
    .collect()
}

fn capture_desktop_region(
  anchor_id: u32,
  region: crate::recording::Region,
) -> Result<CapturedImage, String> {
  let monitors = xcap::Monitor::all().map_err(|error| error.to_string())?;
  let displays = desktop_layout(&monitors)?;
  let plan = desktop_capture::plan(&displays, anchor_id, region, OutputLimits::UNBOUNDED)?;
  let mut pieces = Vec::with_capacity(plan.pieces.len());
  for piece in plan.pieces.iter().copied() {
    let monitor = monitors
      .iter()
      .find(|monitor| monitor.id().ok() == Some(piece.display_id))
      .ok_or_else(|| "A composed display is no longer available".to_owned())?;
    let rect = piece.source_pixels;
    let image = monitor
      .capture_region(rect.x, rect.y, rect.width, rect.height)
      .map(captured)
      .map_err(|error| error.to_string())?;
    pieces.push((piece, image));
  }
  still_composition::compose(&plan, pieces)
}

pub fn capture(target: ScreenshotTarget, show_cursor: bool) -> Result<CapturedImage, String> {
  // Windows Graphics Capture cannot draw the pointer into a frame, so the
  // bar's "Show cursor" toggle has no effect on stills here. macOS honours it
  // through ScreenCaptureKit. This divergence is a platform limit, not a bug,
  // and is why the toggle is not disabled on Windows: it still governs
  // recordings.
  let _ = show_cursor;

  match target {
    ScreenshotTarget::Screen { monitor_id } => monitor(monitor_id)?
      .capture_image()
      .map(captured)
      .map_err(|error| error.to_string()),
    ScreenshotTarget::Region { monitor_id, region } => {
      let monitor = monitor(monitor_id)?;
      let scale = f64::from(monitor.scale_factor().map_err(|error| error.to_string())?);
      // xcap reports Windows monitors in device pixels already, unlike macOS
      // where it reports points, so these need no scaling of their own.
      let width = monitor.width().map_err(|error| error.to_string())?;
      let height = monitor.height().map_err(|error| error.to_string())?;
      let rect = physical_capture_rect(region, scale, width, height)
        .ok_or_else(|| "The selected region is not on the monitor".to_owned())?;

      monitor
        .capture_region(rect.x, rect.y, rect.width, rect.height)
        .map(captured)
        .map_err(|error| error.to_string())
    }
    ScreenshotTarget::DesktopRegion { monitor_id, region } => {
      capture_desktop_region(monitor_id, region)
    }
    ScreenshotTarget::Window { window_id } => xcap::Window::all()
      .map_err(|error| error.to_string())?
      .into_iter()
      .find(|window| window.id().ok() == Some(window_id))
      .ok_or_else(|| "The selected window is no longer available".to_owned())?
      .capture_image()
      .map(captured)
      .map_err(|error| error.to_string()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn windows_physical_monitors_match_the_region_controllers_logical_plane() {
    assert_eq!(
      logical_display(7, -1920, 0, 3840, 2160, 2.0).unwrap(),
      DesktopDisplay {
        id: 7,
        x: -960.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
        scale: 2.0,
      }
    );
    assert!(logical_display(7, 0, 0, 1920, 1080, 0.0).is_err());
  }
}
