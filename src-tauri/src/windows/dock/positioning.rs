// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn recording_dock_offset_path(app: &AppHandle) -> tauri::Result<PathBuf> {
  Ok(
    app
      .path()
      .app_config_dir()?
      .join(RECORDING_DOCK_POSITION_FILE),
  )
}

pub(super) fn load_recording_dock_offset(app: &AppHandle) {
  let offset = recording_dock_offset_path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|contents| serde_json::from_slice::<RecordingDockOffset>(&contents).ok());

  *RECORDING_DOCK_OFFSET
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = offset;
}

pub(super) fn store_recording_dock_offset(
  app: &AppHandle,
  offset: RecordingDockOffset,
) -> tauri::Result<()> {
  *RECORDING_DOCK_OFFSET
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(offset);

  let path = recording_dock_offset_path(app)?;
  if let Some(directory) = path.parent() {
    std::fs::create_dir_all(directory)?;
  }
  let contents = serde_json::to_vec_pretty(&offset).map_err(std::io::Error::other)?;
  std::fs::write(path, contents)?;

  Ok(())
}

/// The pill follows the recording bar rather than the recorded screen: it is
/// excluded from capture, so it never has to sit on the target monitor, and
/// following the bar puts it where the user is already looking.
pub(super) fn recording_dock_monitor(app: &AppHandle) -> tauri::Result<Option<Monitor>> {
  if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
    // Geometry rather than `current_monitor`, because the bar is already
    // hidden by the time the pill is shown.
    if let Some(monitor) = monitor_with_most_overlap(app, &bar)? {
      return Ok(Some(monitor));
    }
  }

  app.primary_monitor()
}

/// Places the pill inside a work area: at its saved offset when it has one,
/// otherwise top-centre with a small gap. Always clamped so it stays wholly
/// inside, which is what makes a saved offset survive a move to a smaller
/// monitor.
pub(super) fn recording_dock_local_position(
  work_area_size: PhysicalSize<u32>,
  dock_size: PhysicalSize<u32>,
  scale: f64,
  offset: Option<RecordingDockOffset>,
) -> (i32, i32) {
  let max_x = f64::from(work_area_size.width.saturating_sub(dock_size.width));
  let max_y = f64::from(work_area_size.height.saturating_sub(dock_size.height));
  let (x, y) = match offset {
    // Offsets are stored in logical pixels, so a pill dropped 200pt from the
    // corner of a Retina display lands 200pt from the corner of a 1x one.
    Some(offset) => (offset.x * scale, offset.y * scale),
    None => (max_x / 2.0, RECORDING_DOCK_TOP_GAP * scale),
  };

  (
    x.clamp(0.0, max_x).round() as i32,
    y.clamp(0.0, max_y).round() as i32,
  )
}

pub(super) fn recording_dock_position(
  monitor: &Monitor,
  dock_size: PhysicalSize<u32>,
  offset: Option<RecordingDockOffset>,
) -> PhysicalPosition<i32> {
  let scale = monitor.scale_factor();
  let work_area = monitor.work_area();
  let (x, y) = recording_dock_local_position(work_area.size, dock_size, scale, offset);

  PhysicalPosition {
    x: work_area.position.x + x,
    y: work_area.position.y + y,
  }
}

/// The pill's position expressed against the work area it was dropped on.
pub(super) fn recording_dock_offset(
  app: &AppHandle,
  dock: &WebviewWindow,
) -> tauri::Result<Option<RecordingDockOffset>> {
  let Some(monitor) = monitor_with_most_overlap(app, dock)? else {
    return Ok(None);
  };
  let scale = monitor.scale_factor();
  let work_area = monitor.work_area();
  let dock_position = dock.outer_position()?;
  let dock_size = dock.outer_size()?;
  let max_x = f64::from(work_area.size.width.saturating_sub(dock_size.width));
  let max_y = f64::from(work_area.size.height.saturating_sub(dock_size.height));

  Ok(Some(RecordingDockOffset {
    x: f64::from(dock_position.x - work_area.position.x).clamp(0.0, max_x) / scale,
    y: f64::from(dock_position.y - work_area.position.y).clamp(0.0, max_y) / scale,
  }))
}
