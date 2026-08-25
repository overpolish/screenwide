// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod clipboard;
pub(crate) mod encoding;
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod mesh;
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod mesh_gpu;
mod naming;
mod output;
#[cfg(test)]
pub(crate) use output::tests::settings as test_output_settings;
mod placement;
#[cfg(target_os = "macos")]
mod platform;
#[cfg(target_os = "windows")]
mod platform_windows;
mod recenter;
pub(crate) mod scrolling;
mod source_crop;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use chrono::Local;
use serde::Deserialize;
use tauri::{image::Image, AppHandle};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::recording::Region;

pub(crate) use crate::capture_geometry::physical_capture_rect;
#[cfg(test)]
pub(crate) use crate::capture_geometry::CaptureRect;
pub(crate) use clipboard::open_in_export as open_clipboard_in_export;
pub use encoding::encode_png;
#[cfg(not(target_os = "macos"))]
pub use encoding::rounded_corners;
#[cfg(target_os = "windows")]
pub(crate) use mesh::validate_mesh;
#[cfg(test)]
pub(crate) use mesh::MeshGradientPoint;
pub use naming::{capture_file_stem, screenshot_directory, unique_path};
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use output::compose_screenshot;
#[cfg(target_os = "windows")]
pub(crate) use output::output_dimensions;
pub(crate) use output::parse_hex_colour;
pub use output::ScreenshotOutputSettings;
pub(crate) use placement::output_placement;
#[cfg(target_os = "macos")]
pub(crate) use platform::{
  alpha_composite, compose_output_layers, native_canvas, NativeCanvas, StillOverlay,
};
#[cfg(target_os = "windows")]
pub(crate) use recenter::{colour_f32, foreground_bounds_f32, optional_colour_f32};
pub(crate) use source_crop::NormalizedSourceRect;
/// A captured still: straight (non-premultiplied) RGBA8, packed rows, top down.
/// That is what both the clipboard and the PNG encoder want.
#[derive(Clone)]
pub struct CapturedImage {
  pub rgba: Vec<u8>,
  pub width: u32,
  pub height: u32,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn validate_output_settings(
  source_width: u32,
  source_height: u32,
  settings: &ScreenshotOutputSettings,
) -> Result<(), String> {
  output_placement(source_width, source_height, settings)?;
  parse_hex_colour(&settings.background_color)?;
  if let Some(colour) = settings.recenter_inset_color.as_deref() {
    parse_hex_colour(colour)?;
  }
  if !settings.radius_percent.is_finite()
    || !(0.0..=50.0).contains(&settings.radius_percent)
    || !settings.background_radius_percent.is_finite()
    || !(0.0..=50.0).contains(&settings.background_radius_percent)
  {
    return Err("The output corner radius is not valid".to_owned());
  }
  match settings.background_type.as_str() {
    "solid" => Ok(()),
    "mesh" => mesh::validate_mesh(
      &settings.mesh_colors,
      &settings.mesh_points,
      settings.mesh_warp_percent,
    ),
    _ => Err("The output background type is not valid".to_owned()),
  }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "kind"
)]
pub enum ScreenshotTarget {
  Screen { monitor_id: u32 },
  Window { window_id: u32 },
  Region { monitor_id: u32, region: Region },
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScreenshotDestination {
  Export,
  #[default]
  Clipboard,
  Both,
}

/// A whole-monitor still with Screenwide's own windows left out, whatever the
/// "Record Screenwide's windows" setting says: the region overlay reads this
/// for its magnifier while it is itself on screen.
#[cfg(target_os = "macos")]
pub(crate) fn capture_monitor_without_own_windows_blocking(
  monitor_id: u32,
) -> Result<CapturedImage, String> {
  platform::capture_blocking(ScreenshotTarget::Screen { monitor_id }, false, false)
}

pub(crate) async fn capture(
  app: &AppHandle,
  target: ScreenshotTarget,
  show_cursor: bool,
  include_ruler: bool,
) -> Result<CapturedImage, String> {
  // Read as the shutter fires, the same way a recording reads it as it starts.
  // A preserved ruler is intentionally part of this shot. The screenshot
  // region and recording controls have already faded out before the shutter.
  let include_own_windows =
    include_ruler || crate::settings::current(app).record_screenwide_windows;

  capture_content(target, include_own_windows, show_cursor).await
}

/// A still with Screenwide's own windows always left out, whatever the "Record
/// Screenwide's windows" setting says: the scrolling capture floats its own
/// progress overlay over the very region it is photographing, frame after
/// frame, so that overlay must never reach the sensor.
pub(crate) async fn capture_excluding_own_windows(
  target: ScreenshotTarget,
) -> Result<CapturedImage, String> {
  capture_content(target, false, false).await
}

async fn capture_content(
  target: ScreenshotTarget,
  include_own_windows: bool,
  show_cursor: bool,
) -> Result<CapturedImage, String> {
  #[cfg(target_os = "macos")]
  {
    tauri::async_runtime::spawn_blocking(move || {
      platform::capture_blocking(target, include_own_windows, show_cursor)
    })
    .await
    .map_err(|error| error.to_string())?
  }

  #[cfg(target_os = "windows")]
  {
    let _ = include_own_windows;
    tauri::async_runtime::spawn_blocking(move || platform_windows::capture(target, show_cursor))
      .await
      .map_err(|error| error.to_string())?
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = (include_own_windows, target, show_cursor);
    Err("Screenshots are not available on this platform".to_owned())
  }
}

/// Freezes a whole monitor before OCR surfaces appear. Screenwide's own
/// windows deliberately remain in the image so text can be recognized there.
pub(crate) async fn capture_overlay_snapshot(monitor_id: u32) -> Result<CapturedImage, String> {
  #[cfg(target_os = "macos")]
  {
    tauri::async_runtime::spawn_blocking(move || {
      platform::capture_blocking(ScreenshotTarget::Screen { monitor_id }, true, false)
    })
    .await
    .map_err(|error| error.to_string())?
  }

  #[cfg(target_os = "windows")]
  {
    tauri::async_runtime::spawn_blocking(move || {
      platform_windows::capture(ScreenshotTarget::Screen { monitor_id }, false)
    })
    .await
    .map_err(|error| error.to_string())?
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = monitor_id;
    Err("Text recognition is not available on this platform".to_owned())
  }
}

pub(crate) async fn capture_text_recognition_snapshot(
  monitor_id: u32,
) -> Result<CapturedImage, String> {
  capture_overlay_snapshot(monitor_id).await
}

/// Captures a still and either copies it or saves it, returning the path it was
/// written to when it went to disk.
#[tauri::command]
pub async fn capture_still(
  app: AppHandle,
  target: ScreenshotTarget,
  show_cursor: bool,
  destination: ScreenshotDestination,
) -> Result<Option<PathBuf>, String> {
  if !crate::recording::is_idle(&app) {
    return Err("A screenshot cannot be taken while a recording is active".to_owned());
  }
  crate::exports::reserve_screenshot_workspace(&app)?;
  let include_ruler = crate::ruler::is_active(&app);
  crate::capture_overlays::dismiss_except(
    &app,
    include_ruler.then_some(crate::capture_overlays::CaptureOverlay::Ruler),
  );
  let image = match capture(&app, target, show_cursor, include_ruler).await {
    Ok(image) => image,
    Err(error) => {
      crate::exports::release_screenshot_workspace(&app);
      return Err(error);
    }
  };

  if matches!(
    destination,
    ScreenshotDestination::Clipboard | ScreenshotDestination::Both
  ) {
    // The clipboard takes the raw pixels, so there is nothing to encode.
    if let Err(error) = app
      .clipboard()
      .write_image(&Image::new(&image.rgba, image.width, image.height))
      .map_err(|error| error.to_string())
    {
      crate::exports::release_screenshot_workspace(&app);
      return Err(error);
    }
    if matches!(destination, ScreenshotDestination::Clipboard) {
      crate::exports::release_screenshot_workspace(&app);
      let _ = crate::windows::hide_recording_ui(app.clone());
      return Ok(None);
    }
  }

  // With the clipboard off, the export window takes over: the user names the
  // file and picks where it goes, so nothing is written here.
  if let Err(error) =
    crate::exports::present_screenshot(&app, image, capture_file_stem(Local::now().naive_local()))
  {
    crate::exports::release_screenshot_workspace(&app);
    return Err(error);
  }

  Ok(None)
}
