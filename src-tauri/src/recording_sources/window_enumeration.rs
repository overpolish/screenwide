// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Thumbnails go to the picker's own `cache_dir`; app icons go to the shared
/// `icon_dir`, which every other surface reads and re-extracts from.
pub(super) fn enumerate_windows(
  cache_dir: &Path,
  icon_dir: &Path,
) -> Result<Vec<WindowDetails>, String> {
  std::fs::create_dir_all(cache_dir).map_err(|error| error.to_string())?;
  let current_pid = std::process::id();
  let windows = xcap::Window::all().map_err(|error| error.to_string())?;
  let selectable_window_ids = platform::selectable_window_ids();

  let mut details = windows
    .into_par_iter()
    .filter_map(|window| {
      let id = window.id().ok()?;
      let pid = window.pid().ok()?;
      let app_name = window.app_name().ok()?;
      let title = window.title().ok()?;
      let width = window.width().ok()?;
      let height = window.height().ok()?;

      if pid == current_pid
        || selectable_window_ids
          .as_ref()
          .is_some_and(|window_ids| !window_ids.contains(&id))
        || title.trim().is_empty()
        || width == 0
        || height == 0
        || window.is_minimized().unwrap_or(true)
      {
        return None;
      }

      // A window without a capturable preview is not a usable recording
      // source. Filter it out just as we do minimized windows above.
      let thumbnail_path = create_thumbnail(&window, cache_dir, id)?;
      let app_icon_path = platform::app_icon(icon_dir, pid);

      Some(WindowDetails {
        id,
        pid,
        app_name,
        title,
        position: Position {
          x: window.x().ok()?,
          y: window.y().ok()?,
        },
        size: Size { width, height },
        app_icon_path,
        thumbnail_path: Some(thumbnail_path),
      })
    })
    .collect::<Vec<_>>();

  details.sort_by_cached_key(|window| {
    (
      window.app_name.to_lowercase(),
      window.title.to_lowercase(),
      window.id,
    )
  });

  Ok(details)
}

pub(super) fn create_thumbnail(
  window: &xcap::Window,
  cache_dir: &Path,
  id: u32,
) -> Option<PathBuf> {
  let image = window.capture_image().ok()?;
  write_thumbnail(image, 320, 180, cache_dir, &format!("window-{id}.png"))
}

/// Scales a capture into the given box and writes it to the picker cache, so
/// window and display previews share one encoder and one naming scheme.
pub(super) fn write_thumbnail(
  image: RgbaImage,
  width: u32,
  height: u32,
  cache_dir: &Path,
  file_name: &str,
) -> Option<PathBuf> {
  let path = cache_dir.join(file_name);
  DynamicImage::ImageRgba8(image)
    .thumbnail(width, height)
    .save(&path)
    .ok()?;
  Some(path)
}
