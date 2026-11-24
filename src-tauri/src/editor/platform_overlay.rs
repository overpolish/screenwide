// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lays a short-video app's interface, and the phone it is previewed on,
//! over the recording editor's picture for the format in hand. Drawn by the
//! workspace's chrome, it never reaches the preview's frames or an export.

use screenwide_platform_overlays::{OverlayRequest, PhoneView, PlatformOverlay};
use serde::Deserialize;
use tauri::{AppHandle, Manager};

use super::EditorKind;

/// The canvas the overlay was chosen for, in output pixels.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct OverlayCanvas {
  width: u32,
  height: u32,
}

// Async so it runs off the main thread, as the preview's layout does: there
// the chrome's drawable presents at once, where on the main thread it waits
// for the next transaction, which only the next press or scrub committed, so
// the overlay showed one change behind.
#[tauri::command]
pub async fn set_editor_platform_overlay(
  app: AppHandle,
  platform: Option<PlatformOverlay>,
  phone: Option<PhoneView>,
  canvas: OverlayCanvas,
) -> Result<(), String> {
  let request = OverlayRequest { platform, phone };
  // A tall canvas is the one a phone and an app are previewed over, so its
  // fonts are read while the preview is still off.
  if canvas.height > canvas.width || !request.is_empty() {
    screenwide_platform_overlays::warm();
  }
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    let window = app
      .get_webview_window(EditorKind::Recording.window_label().as_str())
      .ok_or_else(|| "The recording editor is not open".to_owned())?;
    super::preview_platform::RecordingPreviewSurface::from_window(&window)?
      .set_platform_overlay(request, (canvas.width, canvas.height));
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let _ = (app, request, canvas);
  Ok(())
}
