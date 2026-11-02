// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system's arrow for a page that draws a pointer over its own pictures,
//! such as a setting's preview, so it is the pointer people know rather than
//! a drawing of one.

use serde::Serialize;
use tauri::AppHandle;

/// The arrow written to a file the webview reads through the asset
/// protocol, with the box it is drawn in and its hotspot, in points.
#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SystemCursor {
  pub path: String,
  pub width: f64,
  pub height: f64,
  pub hotspot_x: f64,
  pub hotspot_y: f64,
}

/// `None` where the system's artwork could not be loaded.
#[tauri::command]
pub async fn render_system_cursor(app: AppHandle) -> Result<Option<SystemCursor>, String> {
  let directory = crate::screenshots::thumbnail::cache_directory(&app)?;
  tauri::async_runtime::spawn_blocking(move || {
    let Some(arrow) = screenwide_cursor_effects::system_arrow() else {
      return Ok(None);
    };
    // Written each launch: the system's artwork follows its pointer size
    // and accessibility settings, which may have changed since.
    let path = directory.join("system-arrow.png");
    let png = screenwide_output::encode_png(&screenwide_imaging::CapturedImage {
      width: arrow.image.width(),
      height: arrow.image.height(),
      rgba: arrow.image.as_raw().clone(),
    })?;
    std::fs::write(&path, png)
      .map_err(|error| format!("The pointer could not be written: {error}"))?;
    Ok(Some(SystemCursor {
      path: path.to_string_lossy().into_owned(),
      width: arrow.points.0,
      height: arrow.points.1,
      hotspot_x: arrow.hotspot.0,
      hotspot_y: arrow.hotspot.1,
    }))
  })
  .await
  .map_err(|error| error.to_string())?
}
