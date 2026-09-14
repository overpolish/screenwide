// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[cfg(target_os = "windows")]
#[path = "monitor_thumbnails/windows.rs"]
mod windows;

fn thumbnail_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
  let longest = width.max(height);
  if width == 0 || height == 0 {
    return None;
  }
  let edge = longest.min(320);
  let scale = |value: u32| {
    ((u64::from(value) * u64::from(edge) + u64::from(longest) / 2) / u64::from(longest)).max(1)
      as u32
  };
  Some((scale(width), scale(height)))
}

#[cfg(target_os = "macos")]
fn capture(monitor: &xcap::Monitor) -> Option<RgbaImage> {
  let (width, height) = thumbnail_dimensions(monitor.width().ok()?, monitor.height().ok()?)?;
  let image =
    crate::screenshots::capture_monitor_thumbnail(monitor.id().ok()?, width, height).ok()?;
  RgbaImage::from_raw(image.width, image.height, image.rgba)
}

pub(super) fn capture_monitor_thumbnails(
  cache_dir: &Path,
) -> Result<Vec<MonitorThumbnail>, String> {
  std::fs::create_dir_all(cache_dir).map_err(|error| error.to_string())?;

  let monitors = xcap::Monitor::all().map_err(|error| error.to_string())?;
  // Windows monitor handles are thread-bound. This whole operation already
  // runs off the UI thread; enumerate and capture them on that same worker.
  #[cfg(target_os = "windows")]
  let monitors = monitors.into_iter();
  #[cfg(not(target_os = "windows"))]
  let monitors = monitors.into_par_iter();

  Ok(
    monitors
      .filter_map(|monitor| {
        let id = monitor.id().ok()?;
        #[cfg(target_os = "macos")]
        let image = capture(&monitor)?;
        #[cfg(target_os = "windows")]
        let image = windows::capture(&monitor).ok()?;
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let image = monitor.capture_image().ok()?;
        // A square box lets the longer edge set the scale, so the still keeps
        // the display's aspect ratio whichever way it is oriented.
        let path = write_thumbnail(image, 320, 320, cache_dir, &format!("monitor-{id}.png"))?;
        Some(MonitorThumbnail { id, path })
      })
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::thumbnail_dimensions;

  #[test]
  fn thumbnail_dimensions_fit_display_orientations_without_upscaling() {
    assert_eq!(thumbnail_dimensions(3840, 2160), Some((320, 180)));
    assert_eq!(thumbnail_dimensions(2160, 3840), Some((180, 320)));
    assert_eq!(thumbnail_dimensions(5120, 1440), Some((320, 90)));
    assert_eq!(thumbnail_dimensions(2560, 2560), Some((320, 320)));
    assert_eq!(thumbnail_dimensions(200, 100), Some((200, 100)));
    assert_eq!(thumbnail_dimensions(1, u32::MAX), Some((1, 320)));
    assert_eq!(thumbnail_dimensions(0, 1080), None);
    assert_eq!(thumbnail_dimensions(1920, 0), None);
  }
}
