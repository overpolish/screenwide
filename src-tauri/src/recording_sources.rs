// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "recording_sources/application_metadata.rs"]
mod application_metadata;
#[path = "recording_sources/monitor_thumbnails.rs"]
mod monitor_thumbnails;
#[path = "recording_sources/window_enumeration.rs"]
mod window_enumeration;
use application_metadata::application_details;
pub(crate) use application_metadata::application_icon_cache_dir;

use monitor_thumbnails::capture_monitor_thumbnails;

use window_enumeration::enumerate_windows;
use window_enumeration::write_thumbnail;

use std::{
  collections::{HashMap, HashSet},
  path::{Path, PathBuf},
};

use image::{DynamicImage, RgbaImage};
use rayon::prelude::*;
use serde::Serialize;
use tauri::{AppHandle, Manager};

mod platform;

/// Reachable outside the pickers so Glide resolves its target's icon through
/// the very same extraction, rather than growing a second copy of it.
pub(crate) use platform::app_icon;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDetails {
  id: u32,
  name: String,
  layout_position: Position,
  layout_size: Size,
  position: Position,
  physical_position: Position,
  physical_size: Size,
  size: Size,
  scale_factor: f32,
  is_primary: bool,
  is_builtin: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Position {
  x: i32,
  y: i32,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Size {
  width: u32,
  height: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorThumbnail {
  id: u32,
  path: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowDetails {
  id: u32,
  pid: u32,
  app_name: String,
  title: String,
  position: Position,
  size: Size,
  app_icon_path: Option<PathBuf>,
  thumbnail_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationDetails {
  id: String,
  label: String,
  icon_path: Option<PathBuf>,
  process_ids: Vec<u32>,
}

/// Whether a remembered window can still be recorded, with its app's icon
/// resolved afresh. The remembered icon path points into the temp directory,
/// which macOS purges of files nobody has read for a few days.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedWindowStatus {
  available: bool,
  app_icon_path: Option<PathBuf>,
}

#[tauri::command]
pub fn list_monitors(app: AppHandle) -> Result<Vec<MonitorDetails>, String> {
  crate::monitor_topology::snapshot(&app)?
    .into_iter()
    .map(|display| {
      let scale_factor = display.native.scale_factor();
      let physical_position = display.native.position();
      let physical_size = display.native.size();
      let logical_position = physical_position.to_logical::<f64>(scale_factor);
      let logical_size = physical_size.to_logical::<f64>(scale_factor);

      Ok(MonitorDetails {
        id: display.id,
        name: display
          .capture
          .friendly_name()
          .or_else(|_| display.capture.name())
          .map_err(|error| error.to_string())?,
        layout_position: Position {
          x: display.layout_position.0.round() as i32,
          y: display.layout_position.1.round() as i32,
        },
        layout_size: Size {
          width: display.layout_size.0.round() as u32,
          height: display.layout_size.1.round() as u32,
        },
        position: Position {
          x: logical_position.x.round() as i32,
          y: logical_position.y.round() as i32,
        },
        physical_position: Position {
          x: physical_position.x,
          y: physical_position.y,
        },
        physical_size: Size {
          width: physical_size.width,
          height: physical_size.height,
        },
        size: Size {
          width: logical_size.width.round() as u32,
          height: logical_size.height.round() as u32,
        },
        scale_factor: scale_factor as f32,
        is_primary: display
          .capture
          .is_primary()
          .map_err(|error| error.to_string())?,
        is_builtin: display
          .capture
          .is_builtin()
          .map_err(|error| error.to_string())?,
      })
    })
    .collect()
}

#[tauri::command]
pub async fn list_windows(app: AppHandle) -> Result<Vec<WindowDetails>, String> {
  let cache_dir = app
    .path()
    .temp_dir()
    .map_err(|error| error.to_string())?
    .join("Screenwide")
    .join("window-selector");
  let icon_dir = application_icon_cache_dir(&app)?;
  tauri::async_runtime::spawn_blocking(move || enumerate_windows(&cache_dir, &icon_dir))
    .await
    .map_err(|error| error.to_string())?
}

/// A still of every attached display for the Screen segment and the display
/// chooser. The capture and PNG encoding are the slow parts, so they run off
/// the main thread; the display list comes straight from xcap rather than
/// `monitor_topology::snapshot`, which pairs against Tauri's window API and
/// would have to hop back to the main thread to do it.
#[tauri::command]
pub async fn list_monitor_thumbnails(app: AppHandle) -> Result<Vec<MonitorThumbnail>, String> {
  let cache_dir = app
    .path()
    .temp_dir()
    .map_err(|error| error.to_string())?
    .join("Screenwide")
    .join("monitor-selector");
  tauri::async_runtime::spawn_blocking(move || capture_monitor_thumbnails(&cache_dir))
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn selected_window_status(
  app: AppHandle,
  id: u32,
  pid: u32,
) -> Result<SelectedWindowStatus, String> {
  let icon_dir = application_icon_cache_dir(&app)?;
  tauri::async_runtime::spawn_blocking(move || {
    let selectable_window_ids = platform::selectable_window_ids();
    let windows = xcap::Window::all().map_err(|error| error.to_string())?;

    let available = windows.into_iter().any(|window| {
      window.id().ok() == Some(id)
        && window.pid().ok() == Some(pid)
        && selectable_window_ids
          .as_ref()
          .is_none_or(|window_ids| window_ids.contains(&id))
        && window.title().is_ok_and(|title| !title.trim().is_empty())
        && window.width().is_ok_and(|width| width > 0)
        && window.height().is_ok_and(|height| height > 0)
        && !window.is_minimized().unwrap_or(true)
    });
    // The icon belongs to the process, so it resolves for as long as the app
    // is running, whether or not this particular window still is.
    Ok(SelectedWindowStatus {
      available,
      app_icon_path: platform::app_icon(&icon_dir, pid),
    })
  })
  .await
  .map_err(|error| error.to_string())?
}

#[tauri::command]
#[cfg(target_os = "macos")]
pub async fn list_applications(app: AppHandle) -> Result<Vec<ApplicationDetails>, String> {
  let cache_dir = application_icon_cache_dir(&app)?;
  let applications = platform::audio_applications().await?;
  tauri::async_runtime::spawn_blocking(move || {
    enumerate_audio_applications(&cache_dir, applications)
  })
  .await
  .map_err(|error| error.to_string())?
}

#[cfg(target_os = "macos")]
fn enumerate_audio_applications(
  cache_dir: &Path,
  candidates: Vec<platform::AudioApplication>,
) -> Result<Vec<ApplicationDetails>, String> {
  let mut applications = HashMap::<String, (String, Option<PathBuf>, HashSet<u32>)>::new();

  for candidate in candidates {
    let application = applications.entry(candidate.id).or_insert_with(|| {
      (
        candidate.label,
        platform::app_icon(cache_dir, candidate.pid),
        HashSet::new(),
      )
    });
    application.2.insert(candidate.pid);
  }

  application_details(applications)
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub async fn list_applications(app: AppHandle) -> Result<Vec<ApplicationDetails>, String> {
  let cache_dir = application_icon_cache_dir(&app)?;
  tauri::async_runtime::spawn_blocking(move || enumerate_applications(&cache_dir))
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(not(target_os = "macos"))]
fn enumerate_applications(cache_dir: &Path) -> Result<Vec<ApplicationDetails>, String> {
  let current_pid = std::process::id();
  let mut applications = HashMap::<String, (String, Option<PathBuf>, HashSet<u32>)>::new();

  for window in xcap::Window::all().map_err(|error| error.to_string())? {
    let Ok(pid) = window.pid() else { continue };
    if pid == current_pid {
      continue;
    }
    let Some(id) = platform::app_identity(pid) else {
      continue;
    };
    let Ok(label) = window.app_name() else {
      continue;
    };
    if label.trim().is_empty() {
      continue;
    }

    let application = applications.entry(id).or_insert_with(|| {
      (
        label.trim().to_string(),
        platform::app_icon(cache_dir, pid),
        HashSet::new(),
      )
    });
    application.2.insert(pid);
  }

  application_details(applications)
}
