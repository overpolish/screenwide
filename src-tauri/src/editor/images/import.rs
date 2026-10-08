// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Bringing a picture in for an image: from a file chosen, or from the
//! clipboard.
//!
//! A clipboard can hold a picture several ways at once, so it is read in the
//! order that keeps the most of it. Copied files come first: Finder also
//! puts the file's icon on the clipboard as a picture, and that icon is not
//! what was copied. Then the picture itself, in the formats browsers,
//! screenshots and image apps put there.

use std::path::PathBuf;

use tauri::{AppHandle, WebviewWindow};

use super::store::import_file;
use crate::editor::annotations::image::ImageArt;

/// The picture formats an image can be made from: what the `image` crate
/// reads. HEIC is not among them; Photos copies one as a TIFF or PNG.
pub(crate) const PICTURE_EXTENSIONS: &[&str] =
  &["png", "jpg", "jpeg", "webp", "gif", "tiff", "tif", "bmp"];

/// The picture on the clipboard as an image can carry it, or `None` when
/// there is no picture there. Copied files that are not pictures are not
/// one either.
fn clipboard_image() -> Result<Option<ImageArt>, String> {
  let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
  if let Ok(files) = clipboard.get().file_list() {
    if !files.is_empty() {
      return Ok(files.iter().find_map(|path| import_file(path).ok()));
    }
  }
  clipboard_picture(&mut clipboard)
}

/// The picture data on the macOS pasteboard, in the first format it holds
/// that keeps the most: a PNG keeps its transparency, and the format an app
/// copied the picture in is the picture itself. The TIFF a browser adds
/// beside it is a conversion, read only where nothing else is offered.
#[cfg(target_os = "macos")]
fn clipboard_picture(_: &mut arboard::Clipboard) -> Result<Option<ImageArt>, String> {
  use objc2_app_kit::NSPasteboard;
  use objc2_foundation::NSString;

  const TYPES: &[&str] = &[
    "public.png",
    "com.compuserve.gif",
    "org.webmproject.webp",
    "public.jpeg",
    "public.tiff",
  ];
  let pasteboard = NSPasteboard::generalPasteboard();
  for kind in TYPES {
    if let Some(data) = pasteboard.dataForType(&NSString::from_str(kind)) {
      return super::store::import_bytes(&data.to_vec()).map(Some);
    }
  }
  Ok(None)
}

/// The picture on the Windows clipboard: its registered PNG, which keeps
/// transparency, or else its device-independent bitmap.
#[cfg(not(target_os = "macos"))]
fn clipboard_picture(clipboard: &mut arboard::Clipboard) -> Result<Option<ImageArt>, String> {
  let Ok(picture) = clipboard.get_image() else {
    return Ok(None);
  };
  let (width, height) = (picture.width as u32, picture.height as u32);
  let pixels = image::RgbaImage::from_raw(width, height, picture.bytes.into_owned())
    .ok_or("The clipboard's picture could not be read")?;
  super::store::import(image::DynamicImage::ImageRgba8(pixels)).map(Some)
}

/// The picture on the clipboard, kept for an image, or nothing when the
/// clipboard holds none.
#[tauri::command]
pub async fn read_clipboard_image() -> Result<Option<ImageArt>, String> {
  tauri::async_runtime::spawn_blocking(clipboard_image)
    .await
    .map_err(|error| error.to_string())?
}

/// Asks for a picture file and keeps it for an image, or nothing when the
/// picker is dismissed.
#[tauri::command]
pub async fn browse_annotation_image(
  app: AppHandle,
  window: WebviewWindow,
) -> Result<Option<ImageArt>, String> {
  tauri::async_runtime::spawn_blocking(move || {
    use tauri_plugin_dialog::DialogExt;

    // Parented to the asking window, as the background picker is, so the
    // sheet does not attach to some hidden window instead.
    let picked: Option<PathBuf> = app
      .dialog()
      .file()
      .set_title(crate::i18n::t!("dialog-choose-image"))
      .add_filter(
        crate::i18n::t!("dialog-pictures-filter"),
        PICTURE_EXTENSIONS,
      )
      .set_parent(&window)
      .blocking_pick_file()
      .and_then(|path| path.into_path().ok());
    picked.map(|path| import_file(&path)).transpose()
  })
  .await
  .map_err(|error| error.to_string())?
}
