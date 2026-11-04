// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The system's pointers for a page that draws one over its own pictures,
//! such as a setting's preview, so it is the pointer people know rather than
//! a drawing of one: the arrow, and the crosshair the tools that measure or
//! select the screen show.

use screenwide_records::cursor::CursorStyle;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// Which of the system's pointers a page asks for.
#[derive(Clone, Copy, Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SystemCursorKind {
  Arrow,
  Crosshair,
}

impl SystemCursorKind {
  fn style(self) -> CursorStyle {
    match self {
      Self::Arrow => CursorStyle::Arrow,
      Self::Crosshair => CursorStyle::Crosshair,
    }
  }

  fn file_name(self) -> &'static str {
    match self {
      Self::Arrow => "system-arrow.png",
      Self::Crosshair => "system-crosshair.png",
    }
  }
}

/// The pointer written to a file the webview reads through the asset
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
pub async fn render_system_cursor(
  app: AppHandle,
  kind: SystemCursorKind,
) -> Result<Option<SystemCursor>, String> {
  let directory = crate::screenshots::thumbnail::cache_directory(&app)?;
  tauri::async_runtime::spawn_blocking(move || {
    let Some(cursor) = screenwide_cursor_effects::system_cursor(kind.style()) else {
      return Ok(None);
    };
    // Written each launch: the system's artwork follows its pointer size
    // and accessibility settings, which may have changed since.
    let path = directory.join(kind.file_name());
    let png = screenwide_output::encode_png(&screenwide_imaging::CapturedImage {
      width: cursor.image.width(),
      height: cursor.image.height(),
      rgba: cursor.image.as_raw().clone(),
    })?;
    std::fs::write(&path, png)
      .map_err(|error| format!("The pointer could not be written: {error}"))?;
    Ok(Some(SystemCursor {
      path: path.to_string_lossy().into_owned(),
      width: cursor.points.0,
      height: cursor.points.1,
      hotspot_x: cursor.hotspot.0,
      hotspot_y: cursor.hotspot.1,
    }))
  })
  .await
  .map_err(|error| error.to_string())?
}
