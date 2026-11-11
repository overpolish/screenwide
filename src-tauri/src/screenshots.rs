// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod annotation_bake;
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
pub(crate) mod quick_island;
#[cfg(test)]
mod recenter_tests;
pub(crate) mod scrolling;
pub(crate) mod still_command;
#[cfg(test)]
mod tests;
pub(crate) mod thumbnail;

/// A still composed on the shared compositor as a screenshot is, for tests.
#[cfg(all(any(target_os = "macos", target_os = "windows"), test))]
fn compose_screenshot(
  image: &CapturedImage,
  settings: &screenwide_output::ScreenshotOutputSettings,
) -> Result<CapturedImage, String> {
  screenwide_compositor::compositor::still_overlay::compose_output_layers(
    image, settings, 0.0, true, None, None, None, None, false, false,
  )
}
#[cfg(all(not(any(target_os = "macos", target_os = "windows")), test))]
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
  /// Kept on screen at its true size where it was taken (`references`).
  Reference,
}

/// A whole-monitor still with Screenwide's own windows left out, whatever the
/// "Record Screenwide's windows" setting says: the region overlay reads this
/// for its magnifier while it is itself on screen.
#[cfg(target_os = "macos")]
pub(crate) fn capture_monitor_without_own_windows_blocking(
  monitor_id: u32,
) -> Result<CapturedImage, String> {
  platform::capture_blocking(
    ScreenshotTarget::Screen { monitor_id },
    platform::LeftOut {
      chosen: false,
      include_own_windows: false,
    },
    false,
  )
}

pub(crate) async fn capture(
  app: &AppHandle,
  target: ScreenshotTarget,
  show_cursor: bool,
  include_ruler: bool,
) -> Result<CapturedImage, String> {
  // Read as the shutter fires, the same way a recording reads it as it starts.
  // A preserved ruler is intentionally part of this shot. The screenshot
  // region and recording controls have already faded out before the shutter;
  // Quick Screenshot's island steps out here if the shot would take it.
  let include_own_windows =
    include_ruler || crate::settings::current(app).record_screenwide_windows;
  quick_island::before_shutter(app, include_own_windows);

  capture_content(target, include_own_windows, true, show_cursor).await
}

/// A still with Screenwide's own windows always left out, whatever the "Record
/// Screenwide's windows" setting says: the scrolling capture keeps the island
/// out while it says what the capture is doing, frame after frame, and it may
/// stand over the very region being photographed.
pub(crate) async fn capture_excluding_own_windows(
  target: ScreenshotTarget,
) -> Result<CapturedImage, String> {
  capture_content(target, false, true, false).await
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

/// `chosen` leaves out what the person keeps out of recordings and
/// screenshots in Settings, which only macOS can leave out of a still.
async fn capture_content(
  target: ScreenshotTarget,
  include_own_windows: bool,
  chosen: bool,
  show_cursor: bool,
) -> Result<CapturedImage, String> {
  #[cfg(target_os = "macos")]
  {
    let left_out = platform::LeftOut {
      chosen,
      include_own_windows,
    };
    tauri::async_runtime::spawn_blocking(move || {
      platform::capture_blocking(target, left_out, show_cursor)
    })
    .await
    .map_err(|error| error.to_string())?
  }

  #[cfg(target_os = "windows")]
  {
    let _ = (include_own_windows, chosen);
    tauri::async_runtime::spawn_blocking(move || platform_windows::capture(target, show_cursor))
      .await
      .map_err(|error| error.to_string())?
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = (include_own_windows, chosen, target, show_cursor);
    Err("Screenshots are not available on this platform".to_owned())
  }
}

/// What a frozen desktop keeps of Screenwide's own windows.
#[derive(Clone, Copy)]
pub(crate) enum FrozenDesktop {
  /// Under the Ruler and Text Recognition, which measure and read whatever
  /// is on screen, Screenwide's windows included.
  WithOwnWindows,
  /// Behind window picking: Screenwide's windows stay out where the
  /// platform allows, so the island is not blurred in under itself.
  WithoutOwnWindows,
}

/// Notes that a tray menu item was just clicked: Windows draws the item
/// fading away for a moment after the menu closes, and a freeze it starts
/// waits for it to go.
#[cfg(target_os = "windows")]
pub(crate) fn menu_item_clicked() {
  platform_windows::menu_fade::clicked();
}

/// Freezes every monitor in `monitor_ids` at once, so the wait is the
/// slowest monitor's rather than every monitor's in turn. Answers in the
/// order asked, never with the pointer in.
pub(crate) async fn freeze_monitors(
  monitor_ids: &[u32],
  desktop: FrozenDesktop,
) -> Vec<Result<CapturedImage, String>> {
  #[cfg(target_os = "macos")]
  {
    let left_out = platform::LeftOut {
      chosen: false,
      include_own_windows: matches!(desktop, FrozenDesktop::WithOwnWindows),
    };
    let captures = monitor_ids
      .iter()
      .map(|&monitor_id| {
        tauri::async_runtime::spawn_blocking(move || {
          platform::capture_blocking(ScreenshotTarget::Screen { monitor_id }, left_out, false)
        })
      })
      .collect::<Vec<_>>();
    let mut stills = Vec::with_capacity(captures.len());
    for capture in captures {
      stills.push(
        capture
          .await
          .map_err(|error| error.to_string())
          .and_then(|still| still),
      );
    }
    stills
  }

  // Windows cannot leave Screenwide's windows out of a still.
  #[cfg(target_os = "windows")]
  {
    let _ = desktop;
    let ids = monitor_ids.to_vec();
    tauri::async_runtime::spawn_blocking(move || platform_windows::monitors::capture(&ids))
      .await
      .unwrap_or_else(|error| monitor_ids.iter().map(|_| Err(error.to_string())).collect())
  }

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = desktop;
    monitor_ids
      .iter()
      .map(|_| Err("Freezing the desktop is not available on this platform".to_owned()))
      .collect()
  }
}
