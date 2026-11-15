// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Quick Screenshot command: one still, copied and previewed, or kept on
//! screen as a reference.

use chrono::Local;
use tauri::{image::Image, AppHandle};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::CapturedImage;
use super::{capture, capture_file_stem, ScreenshotDestination, ScreenshotTarget};

#[tauri::command]
pub async fn capture_still(
  app: AppHandle,
  target: ScreenshotTarget,
  show_cursor: bool,
  destination: ScreenshotDestination,
) -> Result<(), String> {
  take_still(app, target, show_cursor, destination).await
}

/// The shutter every still shares: the Quick Screenshot command's and Delayed
/// Screenshot's.
pub(crate) async fn take_still(
  app: AppHandle,
  target: ScreenshotTarget,
  show_cursor: bool,
  destination: ScreenshotDestination,
) -> Result<(), String> {
  if !crate::recording::is_idle(&app) {
    return Err("A screenshot cannot be taken while a recording is active".to_owned());
  }
  crate::editor::reserve_screenshot_workspace(&app)?;
  // The preview of the last shot would be in this one wherever Screenwide's
  // windows are recorded, and this one replaces it anyway.
  super::copied::withdraw(&app);
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
  // A reference is laid over the interface it shows, where a pointer drawn
  // into it would only stand in the way.
  let show_cursor = show_cursor && !matches!(destination, ScreenshotDestination::Reference);
  let image = match capture(&app, target, show_cursor, include_ruler).await {
    Ok(image) => image,
    Err(error) => {
      eprintln!("Screenshot capture failed for {target:?}: {error}");
      crate::editor::release_screenshot_workspace(&app);
      return Err(error);
    }
  };
  let annotations = crate::annotate::screenshot_annotations(target, &image);

  if matches!(destination, ScreenshotDestination::Reference) {
    let opened = keep_as_reference(&app, target, image, annotations).await;
    crate::editor::release_screenshot_workspace(&app);
    let _ = crate::app_windows::hide_recording_ui(app.clone());
    return opened;
  }

  // The clipboard has no layers, so the annotations go into the pixels. The
  // editor gets them as a layer of their own, should the preview open it.
  let scale = super::capture_scale(target);
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let copied = super::annotation_bake::bake_annotations(&image, annotations.clone(), scale);
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let copied: Result<CapturedImage, String> = Ok(image.clone());
  let copied = copied.and_then(|copied| {
    app
      .clipboard()
      .write_image(&Image::new(&copied.rgba, copied.width, copied.height))
      .map_err(|error| error.to_string())
      .map(|()| copied)
  });
  crate::editor::release_screenshot_workspace(&app);
  let copied = copied?;
  let _ = crate::app_windows::hide_recording_ui(app.clone());
  super::copied::show(
    &app,
    super::copied::Shot {
      annotations,
      copied,
      file_stem: capture_file_stem(Local::now().naive_local()),
      image,
      scale,
    },
  );
  Ok(())
}

/// Keeps the still on screen as a reference where it was taken. The window
/// shows it with the live annotations drawn in, as a copied still has them;
/// the editor still gets them as a layer of their own.
async fn keep_as_reference(
  app: &AppHandle,
  target: ScreenshotTarget,
  image: CapturedImage,
  annotations: Vec<screenwide_annotations::Annotation>,
) -> Result<(), String> {
  let scale = super::capture_scale(target);
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let shown = if annotations.is_empty() {
    None
  } else {
    Some(super::annotation_bake::bake_annotations(
      &image,
      annotations.clone(),
      scale,
    )?)
  };
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let shown = None;
  let placement = crate::references::placement(app, target, (image.width, image.height), scale);
  crate::references::open(
    app,
    crate::references::ReferencePicture {
      original: image,
      shown,
      annotations,
      scale,
    },
    placement,
  )
  .await
}
