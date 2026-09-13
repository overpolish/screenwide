// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

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
        let image = monitor.capture_image().ok()?;
        // A square box lets the longer edge set the scale, so the still keeps
        // the display's aspect ratio whichever way it is oriented.
        let path = write_thumbnail(image, 320, 320, cache_dir, &format!("monitor-{id}.png"))?;
        Some(MonitorThumbnail { id, path })
      })
      .collect(),
  )
}
