// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making room beside an editor for the tool panel that hangs off it.
//!
//! Opening can widen the editor and shift it horizontally within its display's
//! work area. If the enlarged window cannot fit, leave its frame alone and let
//! the panel overlap at 100% zoom.

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

use super::geometry::monitor_with_most_overlap;

/// Below a pixel is a rounding difference between what was asked for and what
/// the window manager settled on, not a resize by the user.
const WIDTH_TOLERANCE: f64 = 1.0;

/// Minimum horizontal movement needed for growth, in one physical coordinate
/// space. None means even repositioning cannot fit the enlarged window.
fn growth_position(x: f64, width: f64, delta: f64, area_left: f64, area_width: f64) -> Option<f64> {
  let enlarged_width = width + delta;
  if enlarged_width > area_width {
    return None;
  }
  Some(x.clamp(area_left, area_left + area_width - enlarged_width))
}

/// Grow once, shifting only as far as needed inside the current display.
#[tauri::command]
pub async fn grow_editor_for_panel(
  app: AppHandle,
  label: String,
  delta: f64,
) -> tauri::Result<bool> {
  if !delta.is_finite() || delta <= 0.0 {
    return Ok(false);
  }
  let Some(window) = app.get_webview_window(&label) else {
    return Ok(false);
  };
  if window.is_fullscreen()? || window.is_maximized()? {
    return Ok(false);
  }
  let scale = window.scale_factor()?;
  let position = window.outer_position()?;
  let size = window.outer_size()?;
  let Some(monitor) = monitor_with_most_overlap(&app, &window)? else {
    return Ok(false);
  };
  let work_area = monitor.work_area();
  let Some(x) = growth_position(
    f64::from(position.x),
    f64::from(size.width),
    delta * scale,
    f64::from(work_area.position.x),
    f64::from(work_area.size.width),
  ) else {
    return Ok(false);
  };
  // set_size takes the inner size; containment above uses the outer frame so
  // decorations are accounted for, including on mixed-DPI Windows displays.
  let inner = window.inner_size()?.to_logical::<f64>(scale);
  if x.round() as i32 != position.x {
    window.set_position(PhysicalPosition::new(x.round() as i32, position.y))?;
  }
  window.set_size(LogicalSize::new(inner.width + delta, inner.height))?;
  Ok(true)
}

/// Takes the gutter back when the panel closes, but only from a window still
/// exactly the width the growth produced: a resize by the user in between is
/// the size they chose, and nothing here may overrule it.
#[tauri::command]
pub fn shrink_editor_after_panel(
  app: AppHandle,
  label: String,
  delta: f64,
  expected_width: f64,
) -> tauri::Result<()> {
  if !delta.is_finite() || delta <= 0.0 || !expected_width.is_finite() {
    return Ok(());
  }
  let Some(window) = app.get_webview_window(&label) else {
    return Ok(());
  };
  if window.is_fullscreen()? || window.is_maximized()? {
    return Ok(());
  }
  let scale = window.scale_factor()?;
  let size = window.outer_size()?.to_logical::<f64>(scale);
  if (size.width - expected_width).abs() > WIDTH_TOLERANCE {
    return Ok(());
  }
  window.set_size(LogicalSize::new((size.width - delta).max(1.0), size.height))?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::growth_position;

  #[test]
  fn room_on_the_right_needs_no_move() {
    assert_eq!(
      growth_position(100.0, 900.0, 332.0, 0.0, 1440.0),
      Some(100.0)
    );
  }

  #[test]
  fn centred_window_moves_only_as_far_left_as_needed() {
    assert_eq!(
      growth_position(270.0, 900.0, 332.0, 0.0, 1440.0),
      Some(208.0)
    );
  }

  #[test]
  fn oversized_growth_keeps_the_existing_frame() {
    assert_eq!(growth_position(100.0, 1300.0, 332.0, 0.0, 1440.0), None);
  }

  #[test]
  fn exact_fit_moves_to_the_work_area_left_edge() {
    assert_eq!(
      growth_position(300.0, 1108.0, 332.0, 0.0, 1440.0),
      Some(0.0)
    );
  }

  #[test]
  fn negative_display_origin_and_work_area_inset_are_respected() {
    assert_eq!(
      growth_position(-1000.0, 900.0, 332.0, -1400.0, 1400.0),
      Some(-1232.0)
    );
    assert_eq!(growth_position(0.0, 900.0, 332.0, 80.0, 1360.0), Some(80.0));
  }

  #[test]
  fn growth_uses_the_window_scale_in_physical_coordinates() {
    let delta = 332.0 * 1.5;
    assert_eq!(
      growth_position(405.0, 1350.0, delta, 0.0, 2160.0),
      Some(312.0)
    );
  }
}
