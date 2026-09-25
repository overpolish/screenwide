// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Quick Screenshot command: one still, copied, opened in the editor, or
//! both.

use std::path::PathBuf;

use chrono::Local;
use tauri::{image::Image, AppHandle};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use super::CapturedImage;
use super::{capture, capture_file_stem, ScreenshotDestination, ScreenshotTarget};

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
  crate::editor::reserve_screenshot_workspace(&app)?;
  let include_ruler = crate::ruler::is_active(&app);
  // The annotations are kept, not shot: their windows are excluded from the
  // capture, and what the still takes with it is the annotations themselves.
  crate::capture_overlays::dismiss_except(
    &app,
    if include_ruler {
      &[
        crate::capture_overlays::CaptureOverlay::Annotate,
        crate::capture_overlays::CaptureOverlay::Ruler,
      ]
    } else {
      &[crate::capture_overlays::CaptureOverlay::Annotate]
    },
  );
  let image = match capture(&app, target, show_cursor, include_ruler).await {
    Ok(image) => image,
    Err(error) => {
      eprintln!("Screenshot capture failed for {target:?}: {error}");
      crate::editor::release_screenshot_workspace(&app);
      return Err(error);
    }
  };
  let annotations = crate::annotate::screenshot_annotations(target, &image);

  if matches!(
    destination,
    ScreenshotDestination::Clipboard | ScreenshotDestination::Both
  ) {
    // The clipboard has no layers, so the annotations go into the pixels.
    #[cfg(target_os = "macos")]
    let copied = super::annotation_bake::bake_annotations(&image, annotations.clone());
    #[cfg(target_os = "windows")]
    let copied = crate::annotate::bake_annotations(&image, &annotations);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let copied: Result<CapturedImage, String> = Ok(image.clone());
    if let Err(error) = copied.and_then(|copied| {
      app
        .clipboard()
        .write_image(&Image::new(&copied.rgba, copied.width, copied.height))
        .map_err(|error| error.to_string())
    }) {
      crate::editor::release_screenshot_workspace(&app);
      return Err(error);
    }
    if matches!(destination, ScreenshotDestination::Clipboard) {
      crate::editor::release_screenshot_workspace(&app);
      let _ = crate::windows::hide_recording_ui(app.clone());
      return Ok(None);
    }
  }

  // With the clipboard off, the editor window takes over: the user names the
  // file and picks where it goes, so nothing is written here. The annotations
  // arrive as a layer of their own, editable like any the user draws there.
  if let Err(error) = crate::editor::present_screenshot(
    &app,
    image,
    super::capture_scale(target),
    annotations,
    capture_file_stem(Local::now().naive_local()),
  ) {
    crate::editor::release_screenshot_workspace(&app);
    return Err(error);
  }

  Ok(None)
}
