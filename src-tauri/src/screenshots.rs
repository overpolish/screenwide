// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod clipboard;
pub(crate) mod delayed;
pub(crate) mod desktop;
#[cfg(target_os = "macos")]
pub(crate) mod macos;
mod naming;
#[cfg(target_os = "macos")]
use self::macos::platform;
#[cfg(target_os = "macos")]
pub(crate) use self::macos::platform::capture_monitor_thumbnail;
#[cfg(test)]
mod output_tests;
#[cfg(target_os = "windows")]
mod platform_windows;
#[cfg(test)]
mod recenter_tests;
pub(crate) mod scrolling;
pub(crate) mod still_command;
#[cfg(test)]
mod tests;
pub(crate) mod thumbnail;

/// A still composed on the macOS compositor as a screenshot is, for tests.
/// Elsewhere `screenwide_output::compose_screenshot` composes on the CPU.
#[cfg(all(target_os = "macos", test))]
fn compose_screenshot(
  image: &CapturedImage,
  settings: &screenwide_output::ScreenshotOutputSettings,
) -> Result<CapturedImage, String> {
  screenwide_compositor::compositor::still_overlay::compose_output_layers(
    image, settings, 0.0, true, None, None, None, None, false, false,
  )
}
#[cfg(all(not(target_os = "macos"), test))]
use screenwide_output::compose_screenshot;

use serde::Deserialize;
use tauri::AppHandle;

use screenwide_capture::Region;

pub(crate) use clipboard::open_in_export as open_clipboard_in_export;
pub use naming::{capture_file_stem, screenshot_directory, unique_path};
use screenwide_imaging::CapturedImage;

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
  DesktopRegion { monitor_id: u32, region: Region },
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScreenshotDestination {
  #[serde(alias = "export")]
  Editor,
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

/// The display scale `target` is captured at: how many of the still's pixels
/// one logical point spans, which its redactions are sized by. A window takes
/// the scale of the monitor it is on. One where the display cannot be read.
pub(crate) fn capture_scale(target: ScreenshotTarget) -> f64 {
  let scale = match target {
    ScreenshotTarget::Screen { monitor_id }
    | ScreenshotTarget::Region { monitor_id, .. }
    | ScreenshotTarget::DesktopRegion { monitor_id, .. } => xcap::Monitor::all()
      .ok()
      .and_then(|monitors| {
        monitors
          .into_iter()
          .find(|monitor| monitor.id().ok() == Some(monitor_id))
      })
      .and_then(|monitor| monitor.scale_factor().ok()),
    ScreenshotTarget::Window { window_id } => xcap::Window::all()
      .ok()
      .and_then(|windows| {
        windows
          .into_iter()
          .find(|window| window.id().ok() == Some(window_id))
      })
      .and_then(|window| window.current_monitor().ok())
      .and_then(|monitor| monitor.scale_factor().ok()),
  };
  scale
    .map(f64::from)
    .filter(|scale| scale.is_finite() && *scale > 0.0)
    .unwrap_or(1.0)
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
